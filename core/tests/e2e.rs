use sha2::{Digest, Sha256};
use zescrow_core::{Asset, Condition, Escrow, EscrowError, EscrowState, Party, Result};

fn assert_err<T, E>(res: Result<T>, expected: E)
where
    E: std::fmt::Debug + PartialEq<E>,
    EscrowError: Into<E> + PartialEq<E>,
{
    match res {
        Err(e) => assert_eq!(e.into(), expected),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[test]
fn escrow_lifecycle() {
    // Funded -> released (no condition)
    let mut escrow = Escrow {
        asset: Asset::Fungible {
            id: "test-token".to_string(),
            amount: 10,
        },
        recipient: Party {
            identity_hash: "bob".into(),
        },
        sender: Party {
            identity_hash: "alice".into(),
        },
        condition: None,
        created_block: 0,
        state: EscrowState::Funded,
    };
    assert_eq!(escrow.execute().unwrap(), EscrowState::Released);
    assert_eq!(escrow.state, EscrowState::Released);

    // Executing again should result in invalid state
    assert_err(escrow.execute(), EscrowError::InvalidState);

    // Fund with failing condition
    let mut bad_escrow = Escrow {
        condition: Some(Condition::Preimage {
            hash: [0u8; 32],
            preimage: vec![10],
        }),
        ..escrow.clone()
    };
    bad_escrow.state = EscrowState::Funded;
    assert_err(bad_escrow.execute(), EscrowError::ConditionViolation);
}

#[test]
fn preimage_condition() {
    let preimage = b"secret".to_vec();
    let hash = Sha256::digest(&preimage).into();
    let cond = Condition::Preimage { hash, preimage };
    assert!(cond.verify().is_ok());

    // invalid preimage
    let cond = Condition::Preimage {
        hash,
        preimage: b"wrong-secret".to_vec(),
    };
    assert_err(cond.verify(), EscrowError::ConditionViolation);
}

#[test]
fn ed25519_condition() {
    use ed25519_dalek::ed25519::signature::rand_core::OsRng;
    use ed25519_dalek::{Signer, SigningKey};

    let mut csprng = OsRng;
    let sk: SigningKey = SigningKey::generate(&mut csprng);

    let message = b"zkEscrow".to_vec();
    let signature = sk.sign(&message).to_bytes().to_vec();
    let public_key = sk.verifying_key().to_bytes();

    let cond = Condition::Ed25519 {
        public_key: public_key.clone(),
        signature: signature.clone(),
        message: message.clone(),
    };
    assert!(cond.verify().is_ok());

    // tampered sig
    let mut signature = signature;
    signature[0] ^= 0xFF;
    let cond = Condition::Ed25519 {
        public_key,
        signature,
        message,
    };
    assert_err(cond.verify(), EscrowError::ConditionViolation);
}

#[test]
fn secp256k1_condition() {
    use k256::ecdsa::signature::Signer;
    use k256::ecdsa::{Signature, SigningKey};
    use k256::elliptic_curve::rand_core::OsRng;

    let sk = SigningKey::random(&mut OsRng);
    let vk = sk.verifying_key();
    let message = b"zkEscrow".to_vec();
    let signature: Signature = sk.sign(&message);

    let sig_bytes = signature.to_der().as_bytes().to_vec();
    let pk_bytes = vk.to_encoded_point(false).as_bytes().to_vec();

    let cond = Condition::Secp256k1 {
        public_key: pk_bytes.clone(),
        signature: sig_bytes.clone(),
        message,
    };
    assert!(cond.verify().is_ok());

    // tampered message
    let cond = Condition::Secp256k1 {
        public_key: pk_bytes,
        signature: sig_bytes,
        message: b"tampered".to_vec(),
    };
    assert_err(cond.verify(), EscrowError::ConditionViolation);
}

#[test]
fn threshold_condition() {
    // two trivial subconditions: one succeeds, one fails
    let hash = Sha256::digest(b"zkEscrow").into();
    let correct = Condition::Preimage {
        hash,
        preimage: b"zkEscrow".to_vec(),
    };
    let wrong = Condition::Preimage {
        hash,
        preimage: b"wrong-preimage".to_vec(),
    };

    // threshold == 1 should pass
    let t = Condition::Threshold {
        threshold: 1,
        subconditions: vec![correct.clone(), wrong.clone()],
    };
    assert!(t.verify().is_ok());

    // threshold == 2 should fail
    let t = Condition::Threshold {
        threshold: 2,
        subconditions: vec![correct, wrong],
    };
    assert_err(t.verify(), EscrowError::ConditionViolation);
}

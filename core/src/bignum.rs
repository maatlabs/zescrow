use bincode::de::{BorrowDecoder, Decoder};
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{BorrowDecode, Decode, Encode};
use num_bigint::BigUint;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use crate::biguint_serde;

/// Wrapper around BigUint so we can implement bincode traits.
#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "json", serde(transparent))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct BigNumber(#[cfg_attr(feature = "json", serde(with = "biguint_serde"))] pub BigUint);

impl BigNumber {
    /// Convert Self directly from an `u64`.
    pub fn from_u64(n: u64) -> Self {
        Self::from(BigUint::from(n))
    }
}

impl From<BigUint> for BigNumber {
    fn from(v: BigUint) -> Self {
        BigNumber(v)
    }
}

impl From<BigNumber> for BigUint {
    fn from(v: BigNumber) -> BigUint {
        v.0
    }
}

impl Encode for BigNumber {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> std::result::Result<(), EncodeError> {
        let s = self.0.to_str_radix(10);
        s.encode(encoder)
    }
}

impl<Context> Decode<Context> for BigNumber {
    fn decode<D: Decoder>(decoder: &mut D) -> std::result::Result<Self, DecodeError> {
        let s = String::decode(decoder)?;
        BigUint::parse_bytes(s.as_bytes(), 10)
            .map(BigNumber)
            .ok_or_else(|| DecodeError::OtherString("BigUint parse error".into()))
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BigNumber {
    fn borrow_decode<D: BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, DecodeError> {
        Self::decode(decoder)
    }
}

impl std::fmt::Display for BigNumber {
    /// Print the inner BigUint as a decimal string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_str_radix(10))
    }
}

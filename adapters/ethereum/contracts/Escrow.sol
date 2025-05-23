// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Address.sol";

/// @dev Interface to the on-chain verifier contract.
interface IVerifier {
    /// @notice Return true iff `proof` verifies.
    function verify(bytes calldata proof) external view returns (bool);
}

/// @title Zescrow Escrow Contract
/// @notice Holds funds until either a time-lock expires or a condition is met
contract Escrow is ReentrancyGuard {
    using Address for address payable;

    /// @notice The party who funded the escrow
    address public immutable sender;

    /// @notice The party to receive funds on successful completion
    address public immutable recipient;

    /// @notice Earliest **block number** when escrow can be finished
    uint256 public immutable finishAfter;

    /// @notice Earliest **block number** when escrow can be cancelled
    uint256 public immutable cancelAfter;

    /// @notice Whether this escrow requires an on-chain proof verifcation
    bool public immutable hasConditions;

    /// @notice Address of the on-chain verifier contract (if `hasConditions`)
    address public immutable verifier;

    /// @notice Remaining amount locked in escrow
    uint256 public amount;

    event Created(
        address indexed sender,
        address indexed recipient,
        uint256 amount,
        bool hasConditions
    );
    event Released(address indexed recipient, uint256 amount);
    event Cancelled(address indexed sender, uint256 amount);

    /// @param _recipient Intended beneficiary of escrowed funds
    /// @param _finishAfter Block number after which release is allowed (if no ZK conditions)
    /// @param _cancelAfter Block number after which refund is allowed
    /// @param _hasConditions If true, must submit a proof instead of waiting for `_finishAfter`
    /// @param _verifier Address of the verifier contract
    constructor(
        address _recipient,
        uint256 _finishAfter,
        uint256 _cancelAfter,
        bool _hasConditions,
        address _verifier
    ) payable {
        require(_recipient != address(0), "Zescrow: invalid recipient");
        require(
            _finishAfter > block.number,
            "Zescrow: finishAfter block must be future"
        );
        require(
            _cancelAfter > _finishAfter,
            "Zescrow: cancelAfter block must follow finishAfter block"
        );
        if (_hasConditions) {
            require(_verifier != address(0), "Zescrow: verifier required");
        }

        sender = msg.sender;
        recipient = _recipient;
        finishAfter = _finishAfter;
        cancelAfter = _cancelAfter;
        hasConditions = _hasConditions;
        verifier = _verifier;
        amount = msg.value;

        emit Created(sender, recipient, amount, hasConditions);
    }

    /// @notice Release escrowed funds to recipient if block number is reachead
    /// and/or ZK conditions met
    /// @param proof The ZK proof data (empty if `hasConditions == false`)
    function finishEscrow(bytes calldata proof) external nonReentrant {
        require(block.number >= finishAfter, "Zescrow: too early to finish");
        require(amount > 0, "Zescrow: nothing to release");

        if (hasConditions) {
            require(proof.length > 0, "Zescrow: proof required");
            require(
                IVerifier(verifier).verify(proof),
                "Zescrow: proof verification failed"
            );
        }

        uint256 payout = amount;
        amount = 0;
        emit Released(recipient, payout);
        payable(recipient).sendValue(payout);
    }

    /// @notice Cancel the escrow and refund the `sender` after `cancelAfter`
    function cancelEscrow() external nonReentrant {
        require(msg.sender == sender, "Zescrow: only sender can cancel");
        require(block.number >= cancelAfter, "Zescrow: too early to cancel");

        uint256 refund = amount;
        amount = 0;
        emit Cancelled(sender, refund);
        payable(sender).sendValue(refund);
    }
}

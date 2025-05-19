// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Address.sol";

/// @dev Interface to RISC Zero on-chain verifier.
interface IRisc0Verifier {
    /// @notice Return true iff `proof` verifies.
    function verify(bytes calldata proof) external view returns (bool);
}

/// @title Zescrow Escrow Contract
/// @notice Holds funds until either a time-lock expires or a condition is met
contract Escrow is ReentrancyGuard {
    using Address for address payable;

    address public immutable sender;
    address public immutable recipient;
    uint256 public immutable finishAfter;
    uint256 public immutable cancelAfter;
    bool public immutable hasConditions;
    address public immutable verifier;

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
    /// @param _finishAfter UNIX timestamp after which release is allowed (if no ZK conditions)
    /// @param _cancelAfter UNIX timestamp after which refund is allowed
    /// @param _hasConditions If true, must submit a proof instead of waiting for `_finishAfter`
    /// @param _verifier Address of the deployed RISC Zero on‐chain verifier contract
    constructor(
        address _recipient,
        uint256 _finishAfter,
        uint256 _cancelAfter,
        bool _hasConditions,
        address _verifier
    ) payable {
        require(msg.value > 0, "Must deposit non-zero amount in escrow");

        sender = msg.sender;
        recipient = _recipient;
        amount = msg.value;
        finishAfter = _finishAfter;
        cancelAfter = _cancelAfter;
        hasConditions = _hasConditions;
        verifier = _verifier;

        emit Created(sender, recipient, amount, hasConditions);
    }

    /// @notice Release funds to beneficiary if time-lock passed and/or ZK conditions met
    /// @param proof The RISC Zero proof bytes (empty if `hasConditions == false`)
    function finishEscrow(bytes calldata proof) external nonReentrant {
        if (hasConditions) {
            require(proof.length > 0, "Proof required");
            require(
                IRisc0Verifier(verifier).verify(proof),
                "Proof verification failed"
            );
        } else {
            require(block.timestamp >= finishAfter, "Too early to finish");
        }

        uint256 payout = amount;
        amount = 0;
        emit Released(recipient, payout);
        payable(recipient).sendValue(payout);
    }

    /// @notice Refund the `sender` after `cancelAfter`
    function cancelEscrow() external nonReentrant {
        require(msg.sender == sender, "Only sender can cancel");
        require(block.timestamp >= cancelAfter, "Too early to cancel");

        uint256 refund = amount;
        amount = 0;
        emit Cancelled(sender, refund);
        payable(sender).sendValue(refund);
    }
}

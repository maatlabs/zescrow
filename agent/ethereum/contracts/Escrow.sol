// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/// @title Zescrow Escrow Manager
/// @notice Holds funds until a time-lock expires or explicit cancellation
contract Escrow is ReentrancyGuard {
    /// @dev Represents a single escrow's state
    struct EscrowDB {
        address sender; // depositor
        address recipient; // beneficiary
        uint256 amount; // locked ETH
        uint256 finishAfter; // unlock block (0 = immediate)
        uint256 cancelAfter; // refund block (0 = disabled)
        bool settled; // prevents reuse
    }

    /// @dev Auto-incrementing escrow ID; we start at 1 on creation
    uint256 private _nextEscrowId = 0;

    /// @dev Maps escrow IDs to their state data
    mapping(uint256 => EscrowDB) private _escrows;

    error InvalidRecipient(); // recipient must be non-zero
    error InsufficientValue(); // msg.value > 0
    error TimeLockUnset(); // neither finishAfter nor cancelAfter set
    error InvalidTimeOrder(); // finishAfter >= cancelAfter
    error EscrowNotExists(); // no escrow for given ID
    error OnlyRecipient(); // finishEscrow caller mismatch
    error OnlySender(); // cancelEscrow caller mismatch
    error AlreadySettled(); // escrow already released (finished) or cancelled (refunded)
    error TooEarlyToFinish(); // block.number < finishAfter
    error TooEarlyToCancel(); // block.number < cancelAfter
    error CancelDisabled(); // cancelAfter == 0
    error TransferFailed(); // low-level payable call (transfer) returned false

    event EscrowCreated(
        uint256 indexed escrowId,
        address indexed sender,
        address indexed recipient,
        uint256 amount,
        uint256 finishAfter,
        uint256 cancelAfter
    );
    event EscrowFinished(
        uint256 indexed escrowId,
        address indexed recipient,
        uint256 amount
    );
    event EscrowCancelled(
        uint256 indexed escrowId,
        address indexed sender,
        uint256 amount
    );

    /// @notice Create a new escrow
    /// - Must set at least one of `finishAfter` or `cancelAfter`
    /// - If both set, `finishAfter < cancelAfter`
    /// @param recipient The address to receive funds upon release
    /// @param finishAfter Absolute block number after which finish/release is allowed
    /// @param cancelAfter Absolute block number after which cancel/refund is allowed
    /// @return escrowId A unique identifier for the new escrow
    function createEscrow(
        address recipient,
        uint256 finishAfter,
        uint256 cancelAfter
    ) external payable returns (uint256 escrowId) {
        if (recipient == address(0)) revert InvalidRecipient();
        if (msg.value == 0) revert InsufficientValue();
        if (finishAfter == 0 && cancelAfter == 0) revert TimeLockUnset();
        if (finishAfter != 0 && cancelAfter != 0 && finishAfter >= cancelAfter)
            revert InvalidTimeOrder();

        escrowId = ++_nextEscrowId;

        _escrows[escrowId] = EscrowDB({
            sender: msg.sender,
            recipient: recipient,
            amount: msg.value,
            finishAfter: finishAfter,
            cancelAfter: cancelAfter,
            settled: false
        });

        emit EscrowCreated(
            escrowId,
            msg.sender,
            recipient,
            msg.value,
            finishAfter,
            cancelAfter
        );
    }

    /// @notice Release an existing escrow (callable only by recipient)
    /// @param escrowId The ID of the escrow to finish/complete
    function finishEscrow(uint256 escrowId) external nonReentrant {
        EscrowDB storage escrow = _escrows[escrowId];
        if (escrow.sender == address(0)) revert EscrowNotExists();
        if (msg.sender != escrow.recipient) revert OnlyRecipient();
        if (escrow.settled) revert AlreadySettled();
        if (escrow.finishAfter != 0 && block.number < escrow.finishAfter)
            revert TooEarlyToFinish();

        escrow.settled = true;
        uint256 payout = escrow.amount;
        escrow.amount = 0;
        emit EscrowFinished(escrowId, escrow.recipient, payout);

        (bool success, ) = payable(escrow.recipient).call{value: payout}("");
        if (!success) revert TransferFailed();
    }

    /// @notice Cancel and refund an existing escrow (callable only by sender)
    /// @param escrowId The ID of the escrow to cancel/refund
    function cancelEscrow(uint256 escrowId) external nonReentrant {
        EscrowDB storage escrow = _escrows[escrowId];
        if (escrow.sender == address(0)) revert EscrowNotExists();
        if (msg.sender != escrow.sender) revert OnlySender();
        if (escrow.settled) revert AlreadySettled();
        if (escrow.cancelAfter == 0) revert CancelDisabled();
        if (block.number < escrow.cancelAfter) revert TooEarlyToCancel();

        escrow.settled = true;
        uint256 refund = escrow.amount;
        escrow.amount = 0;
        emit EscrowCancelled(escrowId, escrow.sender, refund);

        (bool success, ) = payable(escrow.sender).call{value: refund}("");
        if (!success) revert TransferFailed();
    }

    /// @notice Retrieve an existing escrow's data
    /// @param escrowId Identifier of the escrow
    /// @return The `EscrowDB` struct for that ID
    function getEscrow(
        uint256 escrowId
    ) external view returns (EscrowDB memory) {
        EscrowDB storage escrow = _escrows[escrowId];
        if (escrow.sender == address(0)) revert EscrowNotExists();
        return escrow;
    }
}

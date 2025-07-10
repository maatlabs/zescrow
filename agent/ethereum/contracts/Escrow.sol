// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Address.sol";

/// @title Zescrow Escrow Contract
/// @notice Holds funds until a time-lock expires or explicit cancellation
contract Escrow is ReentrancyGuard {
    using Address for address payable;

    /// @notice The party who funded the escrow
    address public immutable sender;

    /// @notice The party to receive funds on successful completion
    address public immutable recipient;

    /// @notice Earliest block when escrow can be released
    uint256 public immutable finishAfter;

    /// @notice Earliest block when escrow can be cancelled
    uint256 public immutable cancelAfter;

    /// @notice Address of the factory that deployed this escrow
    address public immutable factory;

    /// @notice Remaining amount locked in escrow
    uint256 public amount;

    bool private settled;

    error InvalidSender();
    error InvalidRecipient();
    error MustSpecifyPath();
    error InvalidTimeOrder();
    error AmountZero();
    error TooEarlyToFinish();
    error TooEarlyToCancel();
    error CancelNotAllowed();
    error AlreadySettled();
    error Unauthorized();

    event Created(
        address indexed sender,
        address indexed recipient,
        uint256 amount
    );
    event Released(address indexed recipient, uint256 amount);
    event Cancelled(address indexed sender, uint256 amount);

    /// @param _sender Depositor of the escrowed funds
    /// @param _recipient Intended beneficiary of escrow
    /// @param _finishAfter Block when release is allowed (0 = immediate)
    /// @param _cancelAfter Block when refund is allowed (0 = disabled)
    constructor(
        address _sender,
        address _recipient,
        uint256 _finishAfter,
        uint256 _cancelAfter
    ) payable {
        if (_sender == address(0)) revert InvalidSender();
        if (_recipient == address(0)) revert InvalidRecipient();

        if (_finishAfter == 0 && _cancelAfter == 0) revert MustSpecifyPath();
        if (_finishAfter != 0 && _finishAfter <= block.number)
            revert InvalidTimeOrder();

        if (_cancelAfter != 0) {
            uint256 base = _finishAfter != 0 ? _finishAfter : block.number;
            if (_cancelAfter <= base) revert InvalidTimeOrder();
        }
        if (msg.value == 0) revert AmountZero();

        sender = _sender;
        recipient = _recipient;
        finishAfter = _finishAfter;
        cancelAfter = _cancelAfter;
        factory = msg.sender;
        amount = msg.value;

        emit Created(sender, recipient, amount);
    }

    /// @notice Release escrowed funds to `recipient`
    function finishEscrow() external nonReentrant {
        if (settled) revert AlreadySettled();
        if (finishAfter != 0 && block.number < finishAfter)
            revert TooEarlyToFinish();

        settled = true;
        uint256 payout = amount;
        amount = 0;
        emit Released(recipient, payout);
        payable(recipient).sendValue(payout);
    }

    /// @notice Cancel the escrow and refund the `sender`
    function cancelEscrow() external nonReentrant {
        if (settled) revert AlreadySettled();
        if (msg.sender != sender && msg.sender != factory)
            revert Unauthorized();

        if (cancelAfter == 0) revert CancelNotAllowed();
        if (block.number < cancelAfter) revert TooEarlyToCancel();

        settled = true;
        uint256 refund = amount;
        amount = 0;
        emit Cancelled(sender, refund);
        payable(sender).sendValue(refund);
    }
}

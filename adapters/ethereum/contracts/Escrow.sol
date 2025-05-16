// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Address.sol";

/// @title Escrow Contract
/// @notice Holds funds until either a time-lock expires or a condition is met
contract Escrow is ReentrancyGuard {
    using Address for address payable;

    address public immutable sender;
    address public immutable recipient;
    uint256 public amount;
    uint256 public immutable finishAfter;
    uint256 public immutable cancelAfter;
    bytes32 public immutable conditionHash;

    event Created(
        address indexed sender,
        address indexed recipient,
        uint256 amount
    );
    event Released(address indexed recipient, uint256 amount);
    event Cancelled(address indexed sender, uint256 amount);

    /// @param _recipient Intended beneficiary of escrowed funds
    /// @param _finishAfter Earliest claim time (UNIX timestamp)
    /// @param _cancelAfter Earliest reclaim/refund time (UNIX timestamp)
    /// @param _conditionHash Optional SHA-256 hash for preimage release
    constructor(
        address _recipient,
        uint256 _finishAfter,
        uint256 _cancelAfter,
        bytes32 _conditionHash
    ) payable {
        require(_recipient != address(0), "Recipient cannot be zero address");
        require(msg.value > 0, "Must deposit non-zero amount in escrow");

        sender = msg.sender;
        recipient = _recipient;
        amount = msg.value;
        finishAfter = _finishAfter;
        cancelAfter = _cancelAfter;
        conditionHash = _conditionHash;

        emit Created(sender, recipient, amount);
    }

    /// @notice Release funds to beneficiary if time-lock passed or condition met
    /// @param preimage Optional preimage to satisfy `conditionHash`
    function release(bytes calldata preimage) external nonReentrant {
        require(msg.sender == sender, "Only sender can release");
        require(block.timestamp >= finishAfter, "Not yet claimable");
        if (conditionHash != bytes32(0)) {
            require(keccak256(preimage) == conditionHash, "Condition not met");
        }

        uint256 payout = amount;
        amount = 0;
        emit Released(recipient, payout);
        payable(sender).sendValue(payout);
    }

    /// @notice Reclaim funds if cancel time-lock passed
    function cancel() external nonReentrant {
        require(msg.sender == sender, "Only sender can cancel");
        require(block.timestamp >= cancelAfter, "Not yet refundable");

        uint256 refund = amount;
        amount = 0;
        emit Cancelled(sender, refund);
        payable(sender).sendValue(refund);
    }
}

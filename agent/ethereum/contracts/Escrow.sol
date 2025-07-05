// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Address.sol";

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

    /// @notice Address of the factory that deployed this escrow
    address public immutable factory;

    /// @notice Remaining amount locked in escrow
    uint256 public amount;

    event Created(
        address indexed sender,
        address indexed recipient,
        uint256 amount
    );
    event Released(address indexed recipient, uint256 amount);
    event Cancelled(address indexed sender, uint256 amount);

    /// @param _depositor Party creating the escrow
    /// @param _recipient Intended beneficiary of escrowed funds
    /// @param _finishAfter Block number after which release is allowed (if no ZK conditions)
    /// @param _cancelAfter Block number after which refund is allowed
    constructor(
        address _depositor,
        address _recipient,
        uint256 _finishAfter,
        uint256 _cancelAfter
    ) payable {
        require(_depositor != address(0), "Zescrow: invalid depositor");
        require(_recipient != address(0), "Zescrow: invalid recipient");
        require(
            _finishAfter > block.number,
            "Zescrow: finishAfter block must be future"
        );
        require(
            _cancelAfter > _finishAfter,
            "Zescrow: cancelAfter block must follow finishAfter block"
        );

        sender = _depositor;
        recipient = _recipient;
        finishAfter = _finishAfter;
        cancelAfter = _cancelAfter;
        factory = msg.sender;
        amount = msg.value;

        emit Created(sender, recipient, amount);
    }

    /// @notice Release escrowed funds to recipient if block number is reachead
    function finishEscrow() external nonReentrant {
        require(block.number >= finishAfter, "Zescrow: too early to finish");
        require(amount > 0, "Zescrow: nothing to release");

        uint256 payout = amount;
        amount = 0;
        emit Released(recipient, payout);
        payable(recipient).sendValue(payout);
    }

    /// @notice Cancel the escrow and refund the `sender` after `cancelAfter`
    function cancelEscrow() external nonReentrant {
        require(
            msg.sender == sender || msg.sender == factory,
            "Zescrow: only sender can cancel"
        );
        require(block.number >= cancelAfter, "Zescrow: too early to cancel");

        uint256 refund = amount;
        amount = 0;
        emit Cancelled(sender, refund);
        payable(sender).sendValue(refund);
    }
}

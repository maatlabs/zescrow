// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "./Escrow.sol";

/// @title Zescrow Escrow Factory
/// @notice Deploy and manage multiple `Escrow` instances
contract EscrowFactory {
    /// @notice Emitted when a new escrow is created
    event EscrowCreated(
        address indexed creator,
        address indexed escrowAddress,
        address indexed recipient,
        uint256 amount,
        uint256 finishAfter,
        uint256 cancelAfter
    );

    /// @notice Indicates whether an address is a valid escrow instance
    mapping(address => bool) public isEscrow;

    /// @notice Maps each escrow instance to its creator
    mapping(address => address) public escrowToCreator;

    /// @notice Lists escrows deployed by each creator
    mapping(address => address[]) private escrowsByCreator;

    error InvalidRecipient();
    error InvalidTimeOrder();
    error MustSpecifyPath();
    error NotAnEscrow();
    error Unauthorized();
    error AmountZero();
    error TransferFailed();

    /// @notice Deploys a new escrow contract:
    /// - Must set at least one of `finishAfter` or `cancelAfter`
    /// - If both set, `finishAfter < cancelAfter`
    /// @param recipient Beneficiary of the escrow
    /// @param finishAfter Block when release is allowed (0 = immediate)
    /// @param cancelAfter Block when refund is allowed (0 = disabled)
    /// @return escrowAddress Address of the newly created escrow contract
    function createEscrow(
        address recipient,
        uint256 finishAfter,
        uint256 cancelAfter
    ) external payable returns (address escrowAddress) {
        if (recipient == address(0)) revert InvalidRecipient();
        if (finishAfter == 0 && cancelAfter == 0) revert MustSpecifyPath();
        if (finishAfter != 0 && cancelAfter != 0 && finishAfter >= cancelAfter)
            revert InvalidTimeOrder();
        if (msg.value == 0) revert AmountZero();

        // Deploy child escrow, forwarding ETH
        Escrow escrow = new Escrow{value: msg.value}(
            msg.sender,
            recipient,
            finishAfter,
            cancelAfter
        );
        escrowAddress = address(escrow);

        // Register escrow
        isEscrow[escrowAddress] = true;
        escrowToCreator[escrowAddress] = msg.sender;
        escrowsByCreator[msg.sender].push(escrowAddress);

        emit EscrowCreated(
            msg.sender,
            escrowAddress,
            recipient,
            msg.value,
            finishAfter,
            cancelAfter
        );
    }

    /// @notice Finalizes the escrow, releasing funds if time-lock conditions are met
    /// @param escrowAddress Address of the `Escrow` contract
    function finishEscrow(address escrowAddress) external {
        if (!isEscrow[escrowAddress]) revert NotAnEscrow();
        Escrow(escrowAddress).finishEscrow();
    }

    /// @notice Cancels the escrow, refunding the creator if time-lock conditions are met
    /// @param escrowAddress Address of the `Escrow` contract
    function cancelEscrow(address escrowAddress) external {
        if (!isEscrow[escrowAddress]) revert NotAnEscrow();
        if (msg.sender != escrowToCreator[escrowAddress]) revert Unauthorized();
        Escrow(escrowAddress).cancelEscrow();
    }

    /// @notice Returns all escrows created by a given address
    /// @param creator The depositor whose escrows to fetch
    /// @return Array of escrow contract addresses
    function getEscrowsByCreator(
        address creator
    ) external view returns (address[] memory) {
        return escrowsByCreator[creator];
    }
}

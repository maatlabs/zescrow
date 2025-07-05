// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "./Escrow.sol";
import "@openzeppelin/contracts/utils/Address.sol";

/// @title Zescrow Escrow Factory
/// @notice Deploy and manage multiple `Escrow` instances
contract EscrowFactory {
    using Address for address payable;

    /// @notice Emitted when a new escrow is created
    event EscrowCreated(
        address indexed creator,
        address indexed escrowAddress,
        address indexed recipient,
        uint256 amount,
        uint256 finishAfter,
        uint256 cancelAfter
    );

    /// @dev Tracks deployed escrow instances
    mapping(address => bool) public isEscrow;

    /// @dev Lists escrows per creator
    mapping(address => address[]) private escrowsByCreator;

    /// @notice Create a new escrow instance
    /// @param recipient Address to receive funds upon successful completion
    /// @param finishAfter Block number after which escrow can be finished
    /// @param cancelAfter Block number after which escrow can be cancelled
    /// @return escrowAddress Address of the newly minted escrow contract
    function createEscrow(
        address recipient,
        uint256 finishAfter,
        uint256 cancelAfter
    ) external payable returns (address escrowAddress) {
        require(recipient != address(0), "Zescrow factory: invalid recipient");
        require(msg.value > 0, "Zescrow factory: must fund escrow");
        require(
            finishAfter > block.number,
            "Zescrow factory: finishAfter must be future"
        );
        require(
            cancelAfter > finishAfter,
            "Zescrow factory: cancelAfter must follow finishAfter"
        );

        Escrow escrow = new Escrow{value: msg.value}(
            msg.sender,
            recipient,
            finishAfter,
            cancelAfter
        );
        escrowAddress = address(escrow);
        isEscrow[escrowAddress] = true;
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

    /// @notice Finish escrow at given address
    /// @param escrowAddress Address of the `Escrow` contract
    function finishEscrow(address escrowAddress) external {
        require(isEscrow[escrowAddress], "Zescrow factory: not a valid escrow");
        Escrow(escrowAddress).finishEscrow();
    }

    /// @notice Cancel escrow at given address
    /// @param escrowAddress Address of the `Escrow` contract
    function cancelEscrow(address escrowAddress) external {
        require(isEscrow[escrowAddress], "Zescrow factory: not a valid escrow");
        Escrow(escrowAddress).cancelEscrow();
    }

    /// @notice Retrieve all escrows created by a specific address
    /// @param creator Address of the escrow creator
    /// @return List of escrow contract addresses
    function getEscrowsByCreator(
        address creator
    ) external view returns (address[] memory) {
        return escrowsByCreator[creator];
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "./Escrow.sol";

contract MockVerifier is IVerifier {
    /// @notice Always return true
    function verify(bytes calldata) external pure override returns (bool) {
        return true;
    }
}

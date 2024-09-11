// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import {IL1DAValidator, L1DAValidatorOutput} from "l1-contracts/state-transition/chain-interfaces/IL1DAValidator.sol";
import {IAvailBridge} from "./interfaces/IAvailBridge.sol";
import {AvailAttestation} from "./lib/AvailAttestation.sol";

/// @notice The DA validator intended to be used in ZK chains.
/// @dev Accepts the necessary data to verify the DA inclusion.
contract AvailDAValidator is IL1DAValidator, AvailAttestation {
    error InvalidOperatorDAInput();

    constructor(address _bridge) AvailAttestation(IAvailBridge(_bridge)) {}

    /// @inheritdoc IL1DAValidator
    function checkDA(
        uint256, // _chainId
        bytes32 l2DAValidatorOutputHash,
        bytes calldata operatorDAInput,
        uint256 // _maxBlobsSupported
    ) external view returns (L1DAValidatorOutput memory output) {
        output.stateDiffHash = bytes32(_operatorDAInput[:32]);
        IAvailBridge.MerkleProofInput memory input = abi.decode(_operatorDAInput, (IAvailBridge.MerkleProofInput));
        if (input.leaf != l2DAValidatorOutputHash) {
            revert InvalidOperatorDAInput();
        }
        _attest(input);
    }
}

// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import {IL2DAValidator} from "./interfaces/IL2DAValidator.sol";

/// Celestia DA validator. It will publish inclusion data that would allow to verify the inclusion.
contract AvailL2DAValidator is IL2DAValidator {
    function validatePubdata(
        // The rolling hash of the user L2->L1 logs.
        bytes32,
        // The root hash of the user L2->L1 logs.
        bytes32,
        // The chained hash of the L2->L1 messages
        bytes32,
        // The chained hash of uncompressed bytecodes sent to L1
        bytes32,
        // Operator data, that is related to the DA itself
        bytes calldata _totalL2ToL1PubdataAndStateDiffs
    ) external pure returns (bytes32 outputHash) {
        // The Merkle path is required to verify the proof inclusion. The `outputHash` is used as a leaf in the Merkle tree.
        outputHash = keccak256(_totalL2ToL1PubdataAndStateDiffs);
    }
}

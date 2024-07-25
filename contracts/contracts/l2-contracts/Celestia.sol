// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import {IL2DAValidator} from "./interfaces/IL2DAValidator.sol";
import {BinaryMerkleMultiproof} from "@blobstreamMain/lib/tree/binary/BinaryMerkleMultiproof.sol";
import {NamespaceMerkleMultiproof} from "@blobstreamMain/lib/tree/namespace/NamespaceMerkleMultiproof.sol";

struct BlobInclusionProof {
    // the blob (the pubdata)
    bytes blob;
    // The multiproof for the row roots into the data root
    BinaryMerkleMultiproof row_inclusion_range_proof;
    // The proofs for the shares into the row roots .
    NamespaceMerkleMultiproof[] share_to_row_root_proofs;
    // The data root of the block containing the blob
    bytes32 dataRoot;
    // The height of the block containing the blob
    uint256 height;
}

/// Celestia DA validator. It will publish inclusion data that would allow to verify the inclusion.
contract CelestiaL2DAValidator is IL2DAValidator {
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
        // outputHash = keccak256(_totalL2ToL1PubdataAndStateDiffs);
        BlobInclusionProof memory proof = abi.decode(_totalL2ToL1PubdataAndStateDiffs, (BlobInclusionProof));


    }
}

// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import {IL2DAValidator} from "./interfaces/IL2DAValidator.sol";
import "../../lib/blobstream-contracts.git/src/lib/tree/binary/BinaryMerkleMultiproof.sol";
import "../../lib/blobstream-contracts.git/src/lib/tree/binary/BinaryMerkletree.sol";
import "../../lib/blobstream-contracts.git/src/lib/tree/namespace/NamespaceMerkletree.sol";
import {DAVerifier} from "../../lib/blobstream-contracts.git/src/lib/verifier/DAVerifier.sol";

struct BlobInclusionProof {
    // the blob (the pubdata)
    bytes[] blob;
    // The multiproof for the row roots into the data root
    BinaryMerkleMultiproof row_inclusion_range_proof;
    // The proofs for the shares into the row roots .
    NamespaceMerkleMultiproof[] share_to_row_root_proofs;
    // The row roots of the rows spanned by the blob
    NamespaceNode[] rowRoots;
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
        
        // I hard-coded the namespace, because it doesn't really matter for L2s like ZKStack
        // However, it should be customizeable
        // So I need to make it a config option
        Namespace memory ns = Namespace(0x00, 0x00000000000000000000000000000000000000000000000102030405);
        BlobInclusionProof memory payload = abi.decode(_totalL2ToL1PubdataAndStateDiffs, (BlobInclusionProof));

        // We have a NamespaceMerkleMultiproof for each row spanned by the blob
        // Here, we verify each of them
        uint start = 0;
        for (uint i = 0; i < payload.share_to_row_root_proofs.length; i++) {
            NamespaceMerkleMultiproof memory proof = payload.share_to_row_root_proofs[i];
            uint end = start + proof.endKey - proof.beginKey;

            // Solidity doesn't let you slice arrays in memory
            // We might want to optimize this, and avoid so much copying
            (bytes[] memory slice, DAVerifier.ErrorCodes err) = DAVerifier.slice(payload.blob, start, end);
            require(err == DAVerifier.ErrorCodes.NoError, "DAVerifier: slice error");
            require(NamespaceMerkleTree.verifyMulti(
                payload.rowRoots[i],
                proof,
                ns,
                slice
            ), "Share to row root NMT multiproof failed");
            start = end;
        }
        
        // For each row spanned by the blob, we have a row root
        // The data root is the merkle root of the row + column roots
        // Finally, we verify the multiproof for the row roots into the data root
        require(BinaryMerkleTree.verifyMultiNamespaced(
            payload.dataRoot,
            payload.row_inclusion_range_proof,
            payload.rowRoots
        ), "Row root to data root multiproof verification failed");

        outputHash = payload.dataRoot;

    }
}

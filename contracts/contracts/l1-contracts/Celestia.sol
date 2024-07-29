// SPDX-License-Identifier: MIT

pragma solidity 0.8.24;

// solhint-disable gas-custom-errors, reason-string

import "@blobstream/DataRootTuple.sol";
import "@blobstream/lib/tree/binary/BinaryMerkleTree.sol";

import {IL1DAValidator, L1DAValidatorOutput} from "l1-contracts/state-transition/chain-interfaces/IL1DAValidator.sol";
import {BlobstreamX} from "blobstreamx/BlobstreamX.sol";

/// @notice The DA validator intended to be used in ZK chains.
/// @dev Accepts the necessary data to verify the DA inclusion.
contract CelestiaDAValidator is IL1DAValidator {
    BlobstreamX public blobstreamX;

    constructor(address _blobstreamX) {
        blobstreamX = BlobstreamX(_blobstreamX);
    }

    /// @inheritdoc IL1DAValidator
    function checkDA(
        uint256, // _chainId
        bytes32 _l2DAValidatorOutputHash,
        bytes calldata _operatorDAInput,
        uint256 // _maxBlobsSupported
    ) external view returns (L1DAValidatorOutput memory output) {
        // - First 32 bytes are the hash of the uncompressed state diff.
        // - Then, there is a 32-byte inclusion proof nonce
        // - Then, there is a 32-byte data root's block height
        // - Then, there is a 32-byte MerkleTree's leaf key
        // - Then, there is a 32-byte MerkleTree's number of leaves
        // - Then, there is a 1-byte amount of the Merkle proof's side nodes
        // - Then, are side nodes of the Merkle proof, 32-bytes each
        require(_operatorDAInput.length >= 161, "Operator DA input is too small");
        require(_operatorDAInput.length % 32 == 1, "Operator DA input must have a remainder of 1 when divided by 32");
        output.stateDiffHash = bytes32(_operatorDAInput[: 32]);

        uint256 proofNonce = uint256(bytes32(_operatorDAInput[32 : 64]));
        DataRootTuple memory dataRootTuple = DataRootTuple(
            uint256(bytes32(_operatorDAInput[64 : 96])), // blockHeight
            _l2DAValidatorOutputHash // using the outputHash as a dataRoot
        );

        BinaryMerkleProof memory proof;
        proof.key = uint256(bytes32(_operatorDAInput[96 : 128]));
        proof.numLeaves = uint256(bytes32(_operatorDAInput[128 : 160]));

        uint256 ptr = 161;
        uint256 sideNodesProvided = uint256(uint8(_operatorDAInput[160]));
        bytes32[] memory sideNodes = new bytes32[](sideNodesProvided);
        for (uint256 i = 0; i < sideNodesProvided; i++) {
            sideNodes[i] = bytes32(_operatorDAInput[ptr : ptr + 32]);
            ptr += 32;
        }

        proof.sideNodes = sideNodes;
        bool isValid = blobstreamX.verifyAttestation(proofNonce, dataRootTuple, proof);

        require(isValid, "BlobstreamX attestation failed");
    }
}

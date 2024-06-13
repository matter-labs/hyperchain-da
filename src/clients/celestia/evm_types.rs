use alloy_sol_macro::sol;
use nmt_rs::{
    NamespaceProof,
    NamespaceId,
    NamespacedHash,
    NamespacedSha2Hasher,
    simple_merkle::proof::Proof,
};

use crate::types::DAError;

const CELESTIA_NS_ID_SIZE: usize = 29;

sol! {
    struct BinaryMerkleProof {
        // List of side nodes to verify and calculate tree.
        bytes32[] sideNodes;
        // The key of the leaf to verify.
        uint256 key;
        // The number of leaves in the tree
        uint256 numLeaves;
    }

    struct NamespaceMerkleMultiproof {
        // The beginning key of the leaves to verify.
        uint256 beginKey;
        // The ending key of the leaves to verify.
        uint256 endKey;
        // List of side nodes to verify and calculate tree.
        NamespaceNode[] sideNodes;
    }

    struct NamespaceNode {
        // Minimum namespace.
        Namespace min;
        // Maximum namespace.
        Namespace max;
        // Node value.
        bytes32 digest;
    }

    struct Namespace {
        // The namespace version.
        bytes1 version;
        // The namespace ID.
        bytes28 id;
    }
}

impl TryFrom<NamespaceId<CELESTIA_NS_ID_SIZE>> for Namespace {
    type Error = DAError;
    fn try_from(namespace_id: NamespaceId<CELESTIA_NS_ID_SIZE>) -> Result<Self, Self::Error> {
        Ok(Self {
            version: namespace_id.0[0].into(),
            id: namespace_id.0[1..].try_into()
                .map_err(|_| DAError {
                    error: anyhow::anyhow!("failed to convert namespace id to array"),
                    is_transient: false,
                })?,
        })
    }
}

impl TryFrom<NamespacedHash<CELESTIA_NS_ID_SIZE>> for NamespaceNode {
    type Error = DAError;
    fn try_from(namespaced_hash: NamespacedHash<CELESTIA_NS_ID_SIZE>) -> Result<Self, Self::Error> {
        Ok(Self {
            min: Namespace::try_from(namespaced_hash.min_namespace())?,
            max: Namespace::try_from(namespaced_hash.max_namespace())?,
            digest: namespaced_hash.hash().into(),
        })
    }
}

impl TryFrom<Proof<NamespacedSha2Hasher<CELESTIA_NS_ID_SIZE>>> for NamespaceMerkleMultiproof {
    type Error = DAError;
    fn try_from(proof: Proof<NamespacedSha2Hasher<CELESTIA_NS_ID_SIZE>>) -> Result<Self, Self::Error> {
        Ok(Self {
            beginKey: proof.range.start.try_into()
                .map_err(|_| DAError {
                    error: anyhow::anyhow!("failed to convert start key to u256"),
                    is_transient: false,
                })?,
            endKey: proof.range.end.try_into()
                .map_err(|_| DAError {
                    error: anyhow::anyhow!("failed to convert end key to u256"),
                    is_transient: false,
                })?,
            sideNodes: proof.siblings.iter()
                .map(|node| NamespaceNode::try_from(node.clone()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
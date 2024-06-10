use alloy_sol_macro::sol;
use nmt_rs::{
    NamespaceProof,
    NamespaceId,
    NamespacedHash,
};

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

impl From<NamespaceId<CELESTIA_NS_ID_SIZE> for Namespace {
    fn from(namespace_id: NamespaceId<CELESTIA_NS_ID_SIZE>) -> Self {
        Self {
            version: namespace_id.version,
            id: namespace_id.id,
        }
    }

}

impl From<NamespacedHash<CELESTIA_NS_ID_SIZE>> for NamespaceNode {
    fn from(hash: NamespacedHash<CELESTIA_NS_ID_SIZE>) -> Self {
        Self {
            min: hash.min_namespace(),
            max: hash.max_namespace(),
            digest: hash.hash,
        }
    }

}

impl From<NamespaceProof> for NamespaceMerkleMultiproof {
    fn from(proof: NamespaceProof) -> Self {
        match proof {
            NamespaceProof::AbsenceProof { .. } => {
                panic!("cannot convert absence proof to multiproof");
            }
            NamespaceProof::PresenceProof { proof, .. } => {
                let sideNodes = proof
                    .siblings
                    .iter()
                    .map(|sibling| NamespaceNode {
                        min: sibling.min.clone(),
                        max: sibling.max.clone(),
                        digest: sibling.digest.clone(),
                    })
                    .collect();
                Self {
                    beginKey: proof.begin_key,
                    endKey: proof.end_key,
                    sideNodes,
                }
            }
        }
    }
}
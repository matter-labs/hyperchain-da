use alloy_sol_macro::sol;
use nmt_rs::{
    NamespaceProof,
    NamespaceId,
    NamespacedHash,
    NamespacedSha2Hasher,
    TmSha2Hasher,
    simple_merkle::proof::Proof,
};


use crate::types::DAError;

use super::InclusionDataPayload;

const CELESTIA_NS_ID_SIZE: usize = 29;

sol! {
    
    struct BlobInclusionProof {
        // The range proof for the row inclusion.
        BinaryMerkleMultiproof row_inclusion_range_proof;
        // The proofs for the share to row root.
        NamespaceMerkleMultiproof[] share_to_row_root_proofs;
    }
    
    struct BinaryMerkleMultiproof {
        // List of side nodes to verify and calculate tree.
        bytes32[] sideNodes;
        // The beginning key of the leaves to verify.
        uint256 beginKey;
        // The ending key of the leaves to verify.
        uint256 endKey;
    }

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

impl TryFrom<InclusionDataPayload> for BlobInclusionProof {
    type Error = DAError;
    fn try_from(payload: InclusionDataPayload) -> Result<Self, Self::Error> {
        let proofs: Result<Vec<Proof<NamespacedSha2Hasher<CELESTIA_NS_ID_SIZE>>>, Self::Error> = payload.share_to_row_root_proofs
            .iter()
            .map(|proof| match proof.clone().into_inner() {
                NamespaceProof::PresenceProof{proof, ..} => Ok(proof.clone()),
                NamespaceProof::AbsenceProof { .. } => Err(DAError {
                    error: anyhow::anyhow!("absence proof not supported"),
                    is_transient: false,
                }),
            })
            .collect();
        Ok(BlobInclusionProof {
            row_inclusion_range_proof: payload.row_inclusion_range_proof.try_into()?,
            share_to_row_root_proofs: proofs?.iter()
                .map(|proof| proof.clone().try_into())
                .collect::<Result<Vec<_>, _>>()?,
        })
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

impl TryFrom<Proof<TmSha2Hasher>> for BinaryMerkleMultiproof {
    type Error = DAError;
    fn try_from(proof: Proof<TmSha2Hasher>) -> Result<Self, Self::Error> {
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
                .map(|node| node.into())
                .collect(),
        })
    }
}

impl TryFrom<NamespaceProof<NamespacedSha2Hasher<CELESTIA_NS_ID_SIZE>, CELESTIA_NS_ID_SIZE>> for NamespaceMerkleMultiproof {
    type Error = DAError;
    fn try_from(proof: NamespaceProof<NamespacedSha2Hasher<CELESTIA_NS_ID_SIZE>, CELESTIA_NS_ID_SIZE>) -> Result<Self, Self::Error> {
        match proof {
            NamespaceProof::PresenceProof{proof, ..} => Ok(Self::try_from(proof)?),
            NamespaceProof::AbsenceProof { .. } => Err(DAError {
                error: anyhow::anyhow!("absence proof not supported"),
                is_transient: false,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cmp::max, fs::File};
    use alloy_sol_macro::sol;
    use alloy_sol_types::{SolValue, sol_data::Bytes};
    use celestia_types::{
        hash::Hash, nmt::{Namespace, NamespaceProof, NamespacedHashExt}, Blob
    };
    use celestia_types::ExtendedHeader;
    use serde_json;
    use nmt_rs::{
        simple_merkle::{db::MemDb, proof::Proof, tree::MerkleTree},
        NamespaceId,
        NamespaceProof as NmtNamespaceProof, 
        NamespacedSha2Hasher,
        TmSha2Hasher
    };

    use super::{NamespaceMerkleMultiproof, NamespaceNode, Namespace as NamespaceEVM, CELESTIA_NS_ID_SIZE};

    // This is just to generate hex bytes for the EVM.
    // the code on the other side is here https://github.com/S1nus/blobstream-contracts/blob/connor/binary-mulitproofs/src/lib/tree/namespace/test/NamespaceMerkleMultiproof.t.sol#L92
    #[test]
    fn proof_to_evm() {

        let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
        let my_namespace_id: NamespaceId<29> = my_namespace.into();

        let proofs_file = File::open("proofs.json").unwrap();
        let proofs: Vec<NamespaceProof> = serde_json::from_reader(proofs_file).unwrap();
        let nmt_proofs: Vec<NmtNamespaceProof<NamespacedSha2Hasher<CELESTIA_NS_ID_SIZE>, CELESTIA_NS_ID_SIZE>> = proofs.iter().map(|p| p.clone().into_inner()).collect();

        let blob_bytes = std::fs::read("blob.dat").unwrap();
        let mut blob = Blob::new(my_namespace, blob_bytes.clone()).unwrap();
        blob.index = Some(8);
        let blob_size: u64 = max(1, blob.data.len() as u64 / 512);
        let shares = blob.to_shares().expect("Failed to split blob to shares");
        let share_values: Vec<[u8; 512]> = shares.iter().map(|share| share.data).collect();
        let share_values_slices: Vec<&[u8]> = share_values.iter().map(|share| &share[..]).collect();
        let shares_evm_bytes = share_values_slices[..(proofs[0].end_idx()-proofs[0].start_idx()) as usize].abi_encode();
        println!("shares evm bytes: {}", shares_evm_bytes.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());

        let namespace: NamespaceEVM = my_namespace_id.try_into().unwrap();
        let namespace_as_hex = namespace.abi_encode().iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        println!("namespace: {}", namespace_as_hex);

        let header_bytes = std::fs::read("header.dat").unwrap();
        let dah = ExtendedHeader::decode_and_validate(&header_bytes).unwrap();
        let eds_row_roots = &dah.dah.row_roots();
        let eds_column_roots = &dah.dah.column_roots();
        let data_tree_leaves: Vec<_> = eds_row_roots
            .iter()
            .chain(eds_column_roots.iter())
            .map(|root| root.to_array())
            .collect();
        let row_root_0 = eds_row_roots[0].clone();
        let row_root_evm: NamespaceNode = row_root_0.try_into().unwrap();
        let row_root_evm_as_hex = row_root_evm.abi_encode().iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
        println!("row_root_hex: {}", row_root_evm_as_hex);

        let proof0 = nmt_proofs[0].clone();
        let evm_proof = NamespaceMerkleMultiproof::try_from(proof0).expect("failed rip");
        let hex_string: String = evm_proof.abi_encode().iter().map(|byte| format!("{:02x}", byte)).collect();
        println!("{}", hex_string);

        let eds_size: u64 = eds_row_roots.len().try_into().unwrap();
        // original data square (ODS) size
        let ods_size = eds_size / 2;
        let first_row_index: u64 = blob.index.unwrap().div_ceil(eds_size) - 1;
        let ods_index = blob.index.unwrap() - (first_row_index * ods_size);
        let last_row_index: u64 = (ods_index + blob_size).div_ceil(ods_size) - 1;

        // "Data root" is the merkle root of the EDS row and column roots
        let hasher = TmSha2Hasher {}; // Tendermint Sha2 hasher
        let mut tree: MerkleTree<MemDb<[u8; 32]>, TmSha2Hasher> = MerkleTree::with_hasher(hasher);
        for leaf in data_tree_leaves.clone() {
            tree.push_raw_leaf(&leaf);
        }
        let rp = tree.build_range_proof(first_row_index as usize..last_row_index as usize + 1);
        let rp_evm: super::BinaryMerkleMultiproof = rp.try_into().unwrap();
        // Ensure that the data root is the merkle root of the EDS row and column roots
        assert_eq!(dah.dah.hash(), Hash::Sha256(tree.root()));
        println!("binary multiproof:");
        println!("{}", rp_evm.abi_encode().iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
        println!("data root: {}", tree.root().iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
        let rp_leaves = data_tree_leaves[first_row_index as usize..last_row_index as usize + 1].to_vec();
        let rp_leaves_abi: Vec<&[u8]> = rp_leaves
            .iter()
            .map(|x| &x[..])
            .collect();
        println!("leaves: {:?}",rp_leaves_abi.abi_encode().iter().map(|byte| format!("{:02x}", byte)).collect::<String>());

    }

}
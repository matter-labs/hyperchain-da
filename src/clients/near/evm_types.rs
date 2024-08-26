use alloy::sol;
use near_jsonrpc_client::methods::light_client_proof::RpcLightClientExecutionProofResponse;
use near_primitives::{
    merkle::{Direction as NearDirection, MerklePathItem as NearMerklePathItem},
    views::{
        BlockHeaderInnerLiteView as NearBlockHeaderInnerLiteView,
        ExecutionOutcomeWithIdView as NearExecutionOutcomeWithIdView,
        LightClientBlockLiteView as NearLightClientBlockLiteView,
    },
};

sol! {
    #[derive(Debug)]
    struct BlobInclusionProof {
        ExecutionOutcomeWithIdView outcomeProof;
        MerklePathItem[] outcomeRootProof;
        LightClientBlockLiteView blockHeaderLite;
        MerklePathItem[] blockProof;
    }

    #[derive(Debug)]
    struct ExecutionOutcomeWithIdView {
        MerklePathItem[] proof;
        bytes32 blockHash;
        bytes32 id;
    }

    #[derive(Debug)]
    struct MerklePathItem {
        bytes32 hash;
        Direction direction;
    }

    #[derive(Debug)]
    enum Direction {
        Left,
        Right,
    }

    #[derive(Debug)]
    struct LightClientBlockLiteView {
        bytes32 prevBlockHash;
        bytes32 innerRestHash;
        BlockHeaderInnerLiteView innerLite;
    }

    #[derive(Debug)]
    struct BlockHeaderInnerLiteView {
        uint64 height;
        bytes32 epochId;
        bytes32 nextEpochId;
        bytes32 prevStateRoot;
        bytes32 outcomeRoot;
        uint64 timestamp;
        bytes32 nextBpHash;
        bytes32 blockMerkleRoot;
    }
}

impl TryFrom<NearMerklePathItem> for MerklePathItem {
    type Error = anyhow::Error;

    fn try_from(value: NearMerklePathItem) -> Result<Self, Self::Error> {
        Ok(MerklePathItem {
            hash: value.hash.0.into(),
            direction: value.direction.try_into()?,
        })
    }
}

impl TryFrom<NearDirection> for Direction {
    type Error = anyhow::Error;

    fn try_from(value: NearDirection) -> Result<Self, Self::Error> {
        Ok(match value {
            NearDirection::Left => Direction::Left,
            NearDirection::Right => Direction::Right,
        })
    }
}

impl TryFrom<NearExecutionOutcomeWithIdView> for ExecutionOutcomeWithIdView {
    type Error = anyhow::Error;

    fn try_from(value: NearExecutionOutcomeWithIdView) -> Result<Self, Self::Error> {
        Ok(ExecutionOutcomeWithIdView {
            proof: value
                .proof
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            blockHash: value.block_hash.0.into(),
            id: value.id.0.into(),
        })
    }
}

impl TryFrom<RpcLightClientExecutionProofResponse> for BlobInclusionProof {
    type Error = anyhow::Error;

    fn try_from(value: RpcLightClientExecutionProofResponse) -> Result<Self, Self::Error> {
        Ok(BlobInclusionProof {
            outcomeProof: value.outcome_proof.try_into()?,
            outcomeRootProof: value
                .outcome_root_proof
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            blockHeaderLite: value.block_header_lite.try_into()?,
            blockProof: value
                .block_proof
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl TryFrom<NearLightClientBlockLiteView> for LightClientBlockLiteView {
    type Error = anyhow::Error;

    fn try_from(value: NearLightClientBlockLiteView) -> Result<Self, Self::Error> {
        Ok(LightClientBlockLiteView {
            prevBlockHash: value.prev_block_hash.0.into(),
            innerRestHash: value.inner_rest_hash.0.into(),
            innerLite: value.inner_lite.try_into()?,
        })
    }
}

impl TryFrom<NearBlockHeaderInnerLiteView> for BlockHeaderInnerLiteView {
    type Error = anyhow::Error;

    fn try_from(value: NearBlockHeaderInnerLiteView) -> Result<Self, Self::Error> {
        Ok(BlockHeaderInnerLiteView {
            height: value.height,
            epochId: value.epoch_id.0.into(),
            nextEpochId: value.next_epoch_id.0.into(),
            prevStateRoot: value.prev_state_root.0.into(),
            outcomeRoot: value.outcome_root.0.into(),
            timestamp: value.timestamp,
            nextBpHash: value.next_bp_hash.0.into(),
            blockMerkleRoot: value.block_merkle_root.0.into(),
        })
    }
}

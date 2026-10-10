use crate::{consensus::test_consensus::TestConsensus, model::services::reachability::ReachabilityService};
use kaspa_consensus_core::{
    BlockHashSet,
    api::ConsensusApi,
    block::{Block, BlockTemplate, MutableBlock, TemplateBuildMode, TemplateTransactionSelector},
    blockhash,
    blockstatus::BlockStatus,
    coinbase::MinerData,
    config::{ConfigBuilder, params::MAINNET_PARAMS},
    constants::BLOCK_VERSION,
    tx::{ScriptPublicKey, ScriptVec, Transaction},
};
use kaspa_database::{prelude::ConnBuilder, utils::get_kaspa_tempdir};
use kaspa_hashes::Hash;
use std::{collections::VecDeque, thread::JoinHandle};

struct OnetimeTxSelector {
    txs: Option<Vec<Transaction>>,
}

impl OnetimeTxSelector {
    fn new(txs: Vec<Transaction>) -> Self {
        Self { txs: Some(txs) }
    }
}

impl TemplateTransactionSelector for OnetimeTxSelector {
    fn select_transactions(&mut self) -> Vec<Transaction> {
        self.txs.take().unwrap()
    }

    fn reject_selection(&mut self, _tx_id: kaspa_consensus_core::tx::TransactionId) {
        unimplemented!()
    }

    fn is_successful(&self) -> bool {
        true
    }
}

struct TestContext {
    consensus: TestConsensus,
    join_handles: Vec<JoinHandle<()>>,
    miner_data: MinerData,
    simulated_time: u64,
    current_templates: VecDeque<BlockTemplate>,
    current_tips: BlockHashSet,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.consensus.shutdown(std::mem::take(&mut self.join_handles));
    }
}

impl TestContext {
    fn new(consensus: TestConsensus) -> Self {
        let join_handles = consensus.init();
        let genesis_hash = consensus.params().genesis.hash;
        let simulated_time = consensus.params().genesis.timestamp;
        Self {
            consensus,
            join_handles,
            miner_data: new_miner_data(),
            simulated_time,
            current_templates: Default::default(),
            current_tips: BlockHashSet::from_iter([genesis_hash]),
        }
    }

    pub fn build_block_template_row(&mut self, nonces: impl Iterator<Item = usize>) -> &mut Self {
        for nonce in nonces {
            self.simulated_time += self.consensus.params().target_time_per_block();
            self.current_templates.push_back(self.build_block_template(nonce as u64, self.simulated_time));
        }
        self
    }

    pub fn assert_row_parents(&mut self) -> &mut Self {
        for t in self.current_templates.iter() {
            assert_eq!(self.current_tips, BlockHashSet::from_iter(t.block.header.direct_parents().iter().copied()));
        }
        self
    }

    pub async fn validate_and_insert_row(&mut self) -> &mut Self {
        self.current_tips.clear();
        while let Some(t) = self.current_templates.pop_front() {
            self.current_tips.insert(t.block.header.hash);
            self.validate_and_insert_block(t.block.to_immutable()).await;
        }
        self
    }

    pub async fn build_and_insert_disqualified_chain(&mut self, mut parents: Vec<Hash>, len: usize) -> Hash {
        // The chain will be disqualified since build_block_with_parents builds utxo-invalid blocks
        for _ in 0..len {
            self.simulated_time += self.consensus.params().target_time_per_block();
            let b = self.build_block_with_parents(parents, 0, self.simulated_time);
            parents = vec![b.header.hash];
            self.validate_and_insert_block(b.to_immutable()).await;
        }
        parents[0]
    }

    pub fn build_block_template(&self, nonce: u64, timestamp: u64) -> BlockTemplate {
        let mut t = self
            .consensus
            .build_block_template(
                self.miner_data.clone(),
                Box::new(OnetimeTxSelector::new(Default::default())),
                TemplateBuildMode::Standard,
            )
            .unwrap();
        t.block.header.timestamp = timestamp;
        t.block.header.nonce = nonce;
        t.block.header.finalize();
        t
    }

    pub fn build_block_with_parents(&self, parents: Vec<Hash>, nonce: u64, timestamp: u64) -> MutableBlock {
        let mut b = self.consensus.build_block_with_parents_and_transactions(blockhash::NONE, parents, Default::default());
        b.header.timestamp = timestamp;
        b.header.nonce = nonce;
        b.header.finalize(); // This overrides the NONE hash we passed earlier with the actual hash
        b
    }

    pub async fn validate_and_insert_block(&mut self, block: Block) -> &mut Self {
        let status = self.consensus.validate_and_insert_block(block).virtual_state_task.await.unwrap();
        assert!(status.has_block_body());
        self
    }

    pub fn assert_tips(&mut self) -> &mut Self {
        assert_eq!(BlockHashSet::from_iter(self.consensus.get_tips().into_iter()), self.current_tips);
        self
    }

    pub fn assert_tips_num(&mut self, expected_num: usize) -> &mut Self {
        assert_eq!(BlockHashSet::from_iter(self.consensus.get_tips().into_iter()).len(), expected_num);
        self
    }

    pub fn assert_virtual_parents_subset(&mut self) -> &mut Self {
        assert!(self.consensus.get_virtual_parents().is_subset(&self.current_tips));
        self
    }

    pub fn assert_valid_utxo_tip(&mut self) -> &mut Self {
        // Assert that at least one body tip was resolved with valid UTXO
        assert!(self.consensus.body_tips().iter().copied().any(|h| self.consensus.block_status(h) == BlockStatus::StatusUTXOValid));
        self
    }
}

#[tokio::test]
async fn template_mining_sanity_test() {
    let config = ConfigBuilder::new(MAINNET_PARAMS).skip_proof_of_work().build();
    let mut ctx = TestContext::new(TestConsensus::new(&config));
    let rounds = 10;
    let width = 3;
    for _ in 0..rounds {
        ctx.build_block_template_row(0..width)
            .assert_row_parents()
            .validate_and_insert_row()
            .await
            .assert_tips()
            .assert_virtual_parents_subset()
            .assert_valid_utxo_tip();
    }
}

#[tokio::test]
async fn block_template_uses_current_block_version() {
    let config = ConfigBuilder::new(MAINNET_PARAMS).skip_proof_of_work().build();
    let consensus = TestConsensus::new(&config);
    let join_handles = consensus.init();
    let miner_data = new_miner_data();

    let template = consensus
        .build_block_template(miner_data, Box::new(OnetimeTxSelector::new(Default::default())), TemplateBuildMode::Standard)
        .unwrap();
    assert_eq!(template.block.header.version, BLOCK_VERSION);

    consensus.shutdown(join_handles);
}

#[tokio::test]
async fn antichain_merge_test() {
    let config = ConfigBuilder::new(MAINNET_PARAMS)
        .skip_proof_of_work()
        .edit_consensus_params(|p| {
            p.max_block_parents = 4;
            p.mergeset_size_limit = 10;
        })
        .build();

    let mut ctx = TestContext::new(TestConsensus::new(&config));

    // Build a large 32-wide antichain
    ctx.build_block_template_row(0..32)
        .validate_and_insert_row()
        .await
        .assert_tips()
        .assert_virtual_parents_subset()
        .assert_valid_utxo_tip();

    // Mine a long enough chain s.t. the antichain is fully merged
    for _ in 0..32 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await.assert_valid_utxo_tip();
    }
    ctx.assert_tips_num(1);
}

#[tokio::test]
async fn cumulative_subsidy_tracks_branch_switches_and_dag_merges() {
    async fn insert_test_block(ctx: &mut TestContext, mut block: MutableBlock) -> Hash {
        ctx.simulated_time += ctx.consensus.params().target_time_per_block();
        block.header.timestamp = ctx.simulated_time;
        block.header.finalize();
        let hash = block.header.hash;
        ctx.validate_and_insert_block(block.to_immutable()).await;
        hash
    }

    let config = ConfigBuilder::new(MAINNET_PARAMS)
        .skip_proof_of_work()
        .edit_consensus_params(|p| {
            p.max_block_parents = 4;
            p.mergeset_size_limit = 10;
        })
        .build();
    let mut ctx = TestContext::new(TestConsensus::new(&config));
    let genesis = config.genesis.hash;
    let miner_data = ctx.miner_data.clone();

    let branch_a1 = ctx.consensus.build_utxo_valid_block_with_parents(1.into(), vec![genesis], miner_data.clone(), vec![]);
    let branch_a1_hash = insert_test_block(&mut ctx, branch_a1).await;
    let branch_a2 = ctx.consensus.build_utxo_valid_block_with_parents(2.into(), vec![branch_a1_hash], miner_data.clone(), vec![]);
    let branch_a2_hash = insert_test_block(&mut ctx, branch_a2).await;
    let branch_a2_cumulative =
        u64::from_le_bytes(ctx.consensus.get_block_transactions(branch_a2_hash, None).unwrap()[0].payload[16..24].try_into().unwrap());
    assert_eq!(branch_a2_cumulative, 10_000_000);

    let branch_b1 = ctx.consensus.build_utxo_valid_block_with_parents(3.into(), vec![genesis], miner_data.clone(), vec![]);
    let branch_b1_hash = insert_test_block(&mut ctx, branch_b1).await;
    let branch_b2 = ctx.consensus.build_utxo_valid_block_with_parents(4.into(), vec![branch_b1_hash], miner_data.clone(), vec![]);
    let branch_b2_hash = insert_test_block(&mut ctx, branch_b2).await;
    let branch_b3 = ctx.consensus.build_utxo_valid_block_with_parents(5.into(), vec![branch_b2_hash], miner_data, vec![]);
    let branch_b3_hash = insert_test_block(&mut ctx, branch_b3).await;
    assert_eq!(ctx.consensus.get_sink(), branch_b3_hash, "the longer branch should replace branch A as the selected chain");

    let mut pre_merge_issuance = 0u64;
    for hash in [branch_a1_hash, branch_a2_hash, branch_b1_hash, branch_b2_hash, branch_b3_hash] {
        let transactions = ctx.consensus.get_block_transactions(hash, None).unwrap();
        let subsidy = u64::from_le_bytes(transactions[0].payload[8..16].try_into().unwrap());
        pre_merge_issuance = pre_merge_issuance.checked_add(subsidy).unwrap();
    }

    ctx.build_block_template_row(0..1);
    let merge_block = ctx.current_templates.pop_front().unwrap().block;
    assert!(merge_block.header.direct_parents().contains(&branch_a2_hash));
    assert!(merge_block.header.direct_parents().contains(&branch_b3_hash));
    let merge_coinbase = &merge_block.transactions[0];
    let merge_subsidy = u64::from_le_bytes(merge_coinbase.payload[8..16].try_into().unwrap());
    let merge_cumulative = u64::from_le_bytes(merge_coinbase.payload[16..24].try_into().unwrap());
    assert_eq!(merge_cumulative, pre_merge_issuance.checked_add(merge_subsidy).unwrap());

    ctx.validate_and_insert_block(merge_block.to_immutable()).await;
}

#[tokio::test]
async fn cumulative_subsidy_survives_pruning_and_database_recovery() {
    let config = ConfigBuilder::new(MAINNET_PARAMS)
        .skip_proof_of_work()
        .edit_consensus_params(|p| {
            p.finality_depth = 8;
            p.pruning_depth = 20;
            p.pruning_proof_m = 2;
        })
        .build();
    let tempdir = get_kaspa_tempdir();
    let db_path = tempdir.path().to_owned();
    let db = ConnBuilder::default().with_db_path(db_path.clone()).with_files_limit(10).build().unwrap();
    let (notification_sender, _) = async_channel::unbounded();
    let mut ctx = TestContext::new(TestConsensus::with_db(db.clone(), &config, notification_sender));

    for _ in 0..80 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await;
    }

    let tip = ctx.consensus.get_sink();
    assert_ne!(ctx.consensus.pruning_point(), config.genesis.hash, "the test chain must advance its pruning point");
    let transactions = ctx.consensus.get_block_transactions(tip, None).unwrap();
    let tip_cumulative = u64::from_le_bytes(transactions[0].payload[16..24].try_into().unwrap());
    drop(ctx);
    drop(db);

    let db = ConnBuilder::default().with_db_path(db_path).with_create_if_missing(false).with_files_limit(10).build().unwrap();
    let (notification_sender, _) = async_channel::unbounded();
    let mut recovery_config = config.clone();
    recovery_config.process_genesis = false;
    let mut recovered = TestContext::new(TestConsensus::with_db(db.clone(), &recovery_config, notification_sender));
    assert_eq!(recovered.consensus.get_sink(), tip, "recovery must restore the selected chain");
    recovered.build_block_template_row(0..1);
    let next_block = recovered.current_templates.pop_front().unwrap().block;
    let next_subsidy = u64::from_le_bytes(next_block.transactions[0].payload[8..16].try_into().unwrap());
    let next_cumulative = u64::from_le_bytes(next_block.transactions[0].payload[16..24].try_into().unwrap());
    assert_eq!(next_cumulative, tip_cumulative.checked_add(next_subsidy).unwrap());
}

#[tokio::test]
async fn basic_utxo_disqualified_test() {
    kaspa_core::log::try_init_logger("info");
    let config = ConfigBuilder::new(MAINNET_PARAMS)
        .skip_proof_of_work()
        .edit_consensus_params(|p| {
            p.max_block_parents = 4;
            p.mergeset_size_limit = 10;
        })
        .build();

    let mut ctx = TestContext::new(TestConsensus::new(&config));

    // Mine a valid chain
    for _ in 0..10 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await.assert_valid_utxo_tip();
    }

    // Get current sink
    let sink = ctx.consensus.get_sink();

    // Mine a longer disqualified chain
    let disqualified_tip = ctx.build_and_insert_disqualified_chain(vec![config.genesis.hash], 20).await;

    assert_ne!(sink, disqualified_tip);
    assert_eq!(sink, ctx.consensus.get_sink());
    assert_eq!(BlockHashSet::from_iter([sink, disqualified_tip]), BlockHashSet::from_iter(ctx.consensus.get_tips().into_iter()));
    assert!(!ctx.consensus.get_virtual_parents().contains(&disqualified_tip));
}

#[tokio::test]
async fn double_search_disqualified_test() {
    // TODO: add non-coinbase transactions and concurrency in order to complicate the test

    kaspa_core::log::try_init_logger("info");
    let config = ConfigBuilder::new(MAINNET_PARAMS)
        .skip_proof_of_work()
        .edit_consensus_params(|p| {
            p.max_block_parents = 4;
            p.mergeset_size_limit = 10;
            p.min_difficulty_window_size = p.difficulty_window_size;
        })
        .build();
    let mut ctx = TestContext::new(TestConsensus::new(&config));

    // Mine 3 valid blocks over genesis
    ctx.build_block_template_row(0..3)
        .validate_and_insert_row()
        .await
        .assert_tips()
        .assert_virtual_parents_subset()
        .assert_valid_utxo_tip();

    // Mark the one expected to remain on virtual chain
    let original_sink = ctx.consensus.get_sink();

    // Find the roots to be used for the disqualified chains
    let mut virtual_parents = ctx.consensus.get_virtual_parents();
    assert!(virtual_parents.remove(&original_sink));
    let mut iter = virtual_parents.into_iter();
    let root_1 = iter.next().unwrap();
    let root_2 = iter.next().unwrap();
    assert_eq!(iter.next(), None);

    // Mine a valid chain
    for _ in 0..10 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await.assert_valid_utxo_tip();
    }

    // Get current sink
    let sink = ctx.consensus.get_sink();

    assert!(ctx.consensus.reachability_service().is_chain_ancestor_of(original_sink, sink));

    // Mine a long disqualified chain
    let disqualified_tip_1 = ctx.build_and_insert_disqualified_chain(vec![root_1], 30).await;

    // And another shorter disqualified chain
    let disqualified_tip_2 = ctx.build_and_insert_disqualified_chain(vec![root_2], 20).await;

    assert_eq!(ctx.consensus.get_block_status(root_1), Some(BlockStatus::StatusUTXOValid));
    assert_eq!(ctx.consensus.get_block_status(root_2), Some(BlockStatus::StatusUTXOValid));

    assert_ne!(sink, disqualified_tip_1);
    assert_ne!(sink, disqualified_tip_2);
    assert_eq!(sink, ctx.consensus.get_sink());
    assert_eq!(
        BlockHashSet::from_iter([sink, disqualified_tip_1, disqualified_tip_2]),
        BlockHashSet::from_iter(ctx.consensus.get_tips().into_iter())
    );
    assert!(!ctx.consensus.get_virtual_parents().contains(&disqualified_tip_1));
    assert!(!ctx.consensus.get_virtual_parents().contains(&disqualified_tip_2));

    // Mine a long enough valid chain s.t. both disqualified chains are fully merged
    for _ in 0..30 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await.assert_valid_utxo_tip();
    }
    ctx.assert_tips_num(1);
}

fn new_miner_data() -> MinerData {
    let secp = secp256k1::Secp256k1::new();
    let mut rng = rand::thread_rng();
    let (_sk, pk) = secp.generate_keypair(&mut rng);
    let script = ScriptVec::from_slice(&pk.serialize());
    MinerData::new(ScriptPublicKey::new(0, script), vec![])
}

fn inactivity_shortcut_config() -> kaspa_consensus_core::config::Config {
    ConfigBuilder::new(MAINNET_PARAMS)
        .skip_proof_of_work()
        .edit_consensus_params(|p| {
            p.finality_depth = 2;
        })
        .build()
}

/// Blocks with `bs <= finality_depth` have no older chain block at the target
/// depth, so the recorded `inactivity_shortcut_block` clamps to genesis. Forward
/// walks begin advancing it once descendants cross `bs = finality_depth + 1`.
#[tokio::test]
async fn inactivity_shortcut_block_clamps_to_genesis_within_finality_depth() {
    let config = inactivity_shortcut_config();
    let mut ctx = TestContext::new(TestConsensus::new(&config));
    let finality_depth = config.finality_depth();
    assert_eq!(finality_depth, 2);

    let mut chain = vec![config.genesis.hash];
    for _ in 0..2 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await;
        chain.push(ctx.consensus.get_sink());
    }

    for hash in chain.iter().copied().skip(1) {
        let header = ctx.consensus.get_header(hash).unwrap();
        assert!(header.blue_score <= finality_depth);
        let meta = ctx.consensus.smt_block_metadata(hash);
        assert_eq!(meta.inactivity_shortcut_block(), config.genesis.hash, "bs={}", header.blue_score);
    }
}

/// Tip at `bs = finality_depth + 4` records the chain block at
/// `bs = target_bs = tip_bs - finality_depth - 1` as its
/// inactivity_shortcut block hash.
#[tokio::test]
async fn inactivity_shortcut_resolves_to_chain_block_at_target_bs() {
    let config = inactivity_shortcut_config();
    let mut ctx = TestContext::new(TestConsensus::new(&config));
    let finality_depth = config.finality_depth();

    let mut chain = Vec::new();
    for _ in 0..6 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await;
        chain.push(ctx.consensus.get_sink());
    }

    let tip = *chain.last().unwrap();
    let tip_header = ctx.consensus.get_header(tip).unwrap();
    assert_eq!(tip_header.blue_score, 6);
    let target_bs = tip_header.blue_score - finality_depth - 1; // = 3

    let expected_block = *chain.iter().find(|h| ctx.consensus.get_header(**h).unwrap().blue_score == target_bs).unwrap();
    let recorded = ctx.consensus.smt_block_metadata(tip).inactivity_shortcut_block();
    assert_eq!(recorded, expected_block);
}

/// Consecutive chain blocks: the inactivity_shortcut advances by one chain
/// block per parent-to-child step, since `target_bs` grows in lockstep with
/// `blue_score` on a no-merge chain.
#[tokio::test]
async fn inactivity_shortcut_advances_one_block_per_chain_step() {
    let config = inactivity_shortcut_config();
    let mut ctx = TestContext::new(TestConsensus::new(&config));

    let mut chain = vec![config.genesis.hash];
    for _ in 0..6 {
        ctx.build_block_template_row(0..1).validate_and_insert_row().await;
        chain.push(ctx.consensus.get_sink());
    }

    for (i, hash) in chain.iter().copied().enumerate().skip(4) {
        let expected = chain[i - 3];
        assert_eq!(ctx.consensus.smt_block_metadata(hash).inactivity_shortcut_block(), expected, "block index {i}");
    }
}

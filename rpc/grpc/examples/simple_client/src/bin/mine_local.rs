use kaspa_addresses::Address;
use kaspa_consensus_core::block::Block;
use kaspa_grpc_client::GrpcClient;
use kaspa_pow::State;
use kaspa_rpc_core::{RpcRawBlock, api::rpc::RpcApi, notify::mode::NotificationMode};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = GrpcClient::connect_with_args(
        NotificationMode::Direct,
        "grpc://127.0.0.1:19210".to_string(),
        None,
        false,
        None,
        false,
        Some(500_000),
        Default::default(),
    )
    .await?;

    let info = client.get_server_info().await?;
    println!("Network: {}; synced: {}", info.network_id, info.is_synced);

    let before = client.get_block_dag_info().await?;
    let address = Address::try_from("slugtest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqlj6aqpg9")?;
    let template = client.get_block_template(address, Vec::new()).await?;
    let block = Block::try_from(template.block)?;
    println!("Template: blue_score={}, DAA={}, bits=0x{:x}", block.header.blue_score, block.header.daa_score, block.header.bits);

    let pow = State::new(&block.header);
    let mut accepted = false;
    for nonce in 0..10_000_000u64 {
        if pow.check_pow(nonce).0 {
            let mut header = (*block.header).clone();
            header.nonce = nonce;
            let mined = Block::new(header, block.transactions.as_ref().clone());
            let response = client.submit_block(RpcRawBlock::from(&mined), false).await?;
            println!("Mined nonce {nonce}; submit report: {:?}", response.report);
            if !response.report.is_success() {
                return Err("node rejected mined block".into());
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let after = client.get_block_dag_info().await?;
            println!("Block count: {} -> {}; virtual DAA score: {}", before.block_count, after.block_count, after.virtual_daa_score);
            accepted = true;
            break;
        }
    }

    client.disconnect().await?;
    if !accepted {
        return Err("no valid nonce found within 10,000,000 attempts".into());
    }
    Ok(())
}

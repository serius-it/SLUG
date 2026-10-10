// Example of a SLUG client using a directly configured wRPC endpoint.

use kaspa_rpc_core::{GetBlockDagInfoResponse, GetServerInfoResponse, api::rpc::RpcApi};
use kaspa_wrpc_client::{
    KaspaRpcClient, Resolver, WrpcEncoding,
    client::{ConnectOptions, ConnectStrategy},
    prelude::{NetworkId, NetworkType},
    result::Result,
};
use std::process::ExitCode;
use std::time::Duration;

#[tokio::main]
async fn main() -> ExitCode {
    match check_node_status().await {
        Ok(_) => {
            println!("Connected to a SLUG node.");
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!("An error occurred: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn check_node_status() -> Result<()> {
    // Select encoding method to use, depending on node settings
    let encoding = WrpcEncoding::Borsh;

    let url = Some("ws://127.0.0.1:19112");
    let resolver = None;

    // Define the SLUG network your node is connected to
    // You can select NetworkType::Mainnet, NetworkType::Testnet, NetworkType::Devnet, NetworkType::Simnet
    let network_type = NetworkType::Mainnet;
    let selected_network = Some(NetworkId::new(network_type));

    // Advanced options
    let subscription_context = None;

    // Create new wRPC client with parameters defined above
    let client = KaspaRpcClient::new(encoding, url, resolver, selected_network, subscription_context)?;

    // Advanced connection options
    let timeout = 5_000;
    let options = ConnectOptions {
        block_async_connect: true,
        connect_timeout: Some(Duration::from_millis(timeout)),
        strategy: ConnectStrategy::Fallback,
        ..Default::default()
    };

    // Connect to the explicitly configured SLUG node
    client.connect(Some(options)).await?;

    // Retrieve and show SLUG node information
    let GetServerInfoResponse { is_synced, server_version, network_id, has_utxo_index, .. } = client.get_server_info().await?;

    println!("Node version: {server_version}");
    println!("Network: {network_id}");
    println!("Node is synced: {is_synced}");
    println!("Node is indexing UTXOs: {has_utxo_index}");

    // Retrieve and show SLUG network information
    let GetBlockDagInfoResponse {
        block_count,
        header_count,
        tip_hashes,
        difficulty,
        past_median_time,
        virtual_parent_hashes,
        pruning_point_hash,
        virtual_daa_score,
        sink,
        ..
    } = client.get_block_dag_info().await?;

    println!("Block count: {block_count}");
    println!("Header count: {header_count}");
    println!("Tip hashes:");
    for tip_hash in tip_hashes {
        println!("{tip_hash}");
    }
    println!("Difficulty: {difficulty}");
    println!("Past median time: {past_median_time}");
    println!("Virtual parent hashes:");
    for virtual_parent_hash in virtual_parent_hashes {
        println!("{virtual_parent_hash}");
    }
    println!("Pruning point hash: {pruning_point_hash}");
    println!("Virtual DAA score: {virtual_daa_score}");
    println!("Sink: {sink}");

    // get block reward by block hash
    let result = client.get_block_reward_info(sink).await?;

    println!(
        "confirmation count: {:?}, reward: {:?}, color: {:?}",
        result.confirmation_count, result.reward_amount, result.block_color
    );

    // Disconnect client from Kaspa node
    client.disconnect().await?;

    // Return function result
    Ok(())
}

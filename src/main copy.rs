use log::info;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_transaction_status::{TransactionDetails, UiConfirmedBlock, UiInstruction, UiMessage, UiParsedInstruction};
use std::env;
use tokio::time::{sleep, Duration};

/// Token Program ID for SPL Tokens on Solana
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    
    env_logger::init();// Initialize the logger 
    let rpc_url: &str = "https://api.mainnet-beta.solana.com"; // Solana mainnet RPC URL

    let config: RpcBlockConfig = RpcBlockConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::JsonParsed),
        transaction_details: Some(TransactionDetails::Full),
        rewards: Some(false),
        commitment: Some(CommitmentConfig { commitment: CommitmentLevel::Confirmed,}), //  If you ever need a higher level of assurance, you could change it to CommitmentLevel::Finalized, but Confirmed is fine for most real-time use cases.
        max_supported_transaction_version: Some(0),
    };

    let client = RpcClient::new(rpc_url);

    // Monitor new token mint events by polling recent blocks
    loop {
        match get_slot(&client) {
            Ok(slot) => {
                println!("Gonna try to get block");
                if let Err(err) = get_block(slot, &client, &config).await {
                    eprintln!("Error fetching block: {}", err);
                }
            }
            Err(err) => {
                eprintln!("Error in token monitoring: {}", err);
            }
        }

        sleep(Duration::from_secs(5)).await; // Poll every 10 seconds
    }
}

/// Function to monitor for new token minting events
fn get_slot(client: &RpcClient) -> Result<u64, Box<dyn std::error::Error>> {
    let latest_slot = client.get_slot()?;  // Get the latest slot
    info!("Found slot: {:?}", latest_slot);
    Ok(latest_slot)
}

async fn get_block(slot: u64 ,client: &RpcClient, config: &RpcBlockConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(block) = client.get_block_with_config( slot, *config) {
        //println!("{:?}", block);

        let _ = extract_mint_events(block);
        println!("after extract_mint_events");
    }
    Ok(())
}

fn extract_mint_events(block: UiConfirmedBlock) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the block contains transactions
    println!("Check if the block contains transactions");

    if let Some(transactions) = block.transactions {
        for (index, tx) in transactions.iter().enumerate() {
            println!("Processing transaction index: {}", index); // Debugging output
            
            // Access the encoded transaction directly as a JSON value
            let encoded_transaction = &tx.transaction; // This is of type EncodedTransactionWithStatusMeta
            
            // Log the encoded transaction for debugging
           // println!("Encoded transaction: {:?}", encoded_transaction);

            // Attempt to parse the JSON directly
            if let Ok(json_value) = serde_json::to_value(encoded_transaction) {
                // Access the signatures
                if let Some(signatures) = json_value.get("signatures") {
                    if let Some(signatures_array) = signatures.as_array() {
                        for signature in signatures_array {
                            println!("Signature: {:?}", signature);
                        }
                    }
                }
            } else {
                eprintln!("Failed to parse transaction to JSON for transaction index: {}", index);
            }
        }
    } else {
        eprintln!("No transactions found in the block.");
    }
    Ok(())
}
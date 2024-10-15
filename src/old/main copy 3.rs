use log::{debug, error, info, warn};
use serde_json::Value; // Import serde_json for parsing JSON data
use solana_client::rpc_client::{RpcClient, RpcClientConfig};
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::Signature;
use solana_sdk::{config, exit, transaction};
use solana_sdk::{pubkey::Pubkey, signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransactionWithStatusMeta, TransactionDetails, UiConfirmedBlock, UiInstruction, UiMessage, UiTransaction, UiTransactionEncoding
};

use std::env;
use std::str::FromStr;
use std::error::Error;
use serde_json::value;  // Ensure this is included in your dependencies

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
    }
    Ok(())
}


/// Asynchronous function to extract mint events from a confirmed block
async fn extract_mint_events(block: UiConfirmedBlock) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the block contains transactions
    if let Some(transactions) = block.transactions {
        for tx in transactions {
            // Access the transaction directly
            let encoded_transaction = tx.transaction; // This is of type EncodedTransactionWithStatusMeta

            // Attempt to decode the transaction
            if let Some(ui_transaction) = encoded_transaction.decode() {
                // Now you can access the signatures
                for signature in &ui_transaction.signatures {
                    println!("Signature: {:?}", signature);
                }

                // Access the instructions by calling the method
                let instructions = ui_transaction.message.instructions();
                for instruction in instructions {
                    // You can process the instruction here
                    println!("Instruction: {:?}", instruction);
                }
            } else {
                eprintln!("Failed to decode transaction.");
            }
        }
    } else {
        eprintln!("No transactions found in the block.");
    }
    Ok(())
}

use log::{debug, info};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::{commitment_config::{CommitmentConfig, CommitmentLevel}, feature_set::FullInflationFeaturePair};
use solana_transaction_status::{TransactionDetails, UiConfirmedBlock, UiInstruction};
use std::env;
use tokio::time::{sleep, Duration};


/// Token Program ID for SPL Tokens on Solana
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
/// Program ID for Raydium AMM
const RAYDIUM_AMM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
/// Program ID for Orca AMM
const ORCA_AMM_PROGRAM_ID: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";


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
                //println!("Gonna try to get block");
                if let Err(err) = get_block(slot, &client, &config).await {
                    eprintln!("Error fetching block: {}", err);
                }
            }
            Err(err) => {
                eprintln!("Error in token monitoring: {}", err);
            }
        }

        //sleep(Duration::from_secs(0.1)).await; // Poll every 10 seconds
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

        let _ = extract_mint_events(&block, slot);
        let _ = monitor_liquidity_pools(&block);
        //println!("after extract_mint_events");
    }
    Ok(())
}

fn extract_mint_events(block: &UiConfirmedBlock, slot: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the block contains transactions
    //println!("Check if the block contains transactions");

    if let Some(transactions) = &block.transactions {
        let count = transactions.iter().count();
        info!("Counted {} transactions in block {:?}", count, slot);
        for (index, tx) in transactions.iter().enumerate() {
            debug!("Processing transaction index: {}", index); // Debugging output
            
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
                            //println!("Signature: {:?}", signature);
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

/// Function to monitor for new liquidity pools (handling JSON)
fn monitor_liquidity_pools(block: &UiConfirmedBlock) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(transactions) = &block.transactions {
        for (index, tx) in transactions.iter().enumerate() {
            //println!("Monitoring transaction index: {}", index); // Debugging output
            
            // Access the encoded transaction directly as JSON
            let encoded_transaction = &tx.transaction; // Encoded transaction with status meta

            // Convert the encoded transaction to JSON for easier parsing
            if let Ok(json_value) = serde_json::to_value(encoded_transaction) {
                if let Some(message) = json_value.get("message") {
                     if let Some(instructions) = message.get("instructions") {
                         if let Some(instruction_array) = instructions.as_array() {
                            println!("instruction_array \n{:?}",instruction_array);
                            for instruction in instruction_array {
                                // Look for the program ID
                                //if let Some(program_id) = instruction.get("programId") {
                                    //if let Some(program_id_str) = program_id.as_str() {
                                        //if program_id_str == RAYDIUM_AMM_PROGRAM_ID || program_id_str == ORCA_AMM_PROGRAM_ID {
                                            // Check if it's a pool creation instruction
                                             if let Some(parsed_instruction) = instruction.get("parsed") {
                                                 if let Some(parsed_object) = parsed_instruction.as_object() {
                                                     if let Some(instruction_type) = parsed_object.get("type") {
                                                         if instruction_type == "initializePool" {
                                                            println!("-----------------------------------------------------------------------FOUND ONE---------------------------------------------------------------\n{:?}",instruction_array);
                                            //                 // Extract the token pair (simplified example)
                                            //                 if let Some(info) = parsed_object.get("info") {
                                            //                     if let Some(info_obj) = info.as_object() {
                                            //                         let token_a = info_obj.get("tokenA").unwrap().as_str().unwrap();
                                            //                         let token_b = info_obj.get("tokenB").unwrap().as_str().unwrap();
                                            //                         println!("New liquidity pool detected: {} / {}", token_a, token_b);
                                            //                     }
                                            //                 }
                                            //             }
                                            //         }
                                            //     }
                                            }
                                        }
                                    }
                                }
                            }
                         }
                    }
                }
            } else {
                eprintln!("Failed to parse transaction to JSON for transaction index: {}", index);
            }
        }
    } else {
        eprintln!("No transactions found in the block for liquidity monitoring.");
    }
    Ok(())
}
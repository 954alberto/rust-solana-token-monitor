use std::result;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use futures::{stream::StreamExt, SinkExt};
use log::info;
use serde_json::Value;
use solana_client::rpc_client::{RpcClient, RpcClientConfig};
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::config;
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, TransactionDetails, UiInnerInstructions, UiTransactionEncoding};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::client_with_config;
use tokio_tungstenite::tungstenite::Message;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use solana_client::client_error::ClientError;

// use solana_transaction_status::UiTransactionEncoding;

// use solana_client::rpc_config::RpcBlockConfig;
use anyhow::Result; // To easily propagate errors


#[tokio::main]
async fn main() {

    env_logger::init();// Initialize the logger 

    // WebSocket URL for Solana's RPC
    let ws_url = "wss://api.mainnet-beta.solana.com/".to_string(); // Replace with correct WebSocket URL

    // Connect to the WebSocket

    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to new slot notifications
    let subscription_message = r#"{"jsonrpc":"2.0","id":1,"method":"slotSubscribe","params":[]}"#;
    write
        .send(Message::Text(subscription_message.to_string()))
        .await
        .expect("Failed to subscribe");

    // Instantiate the Solana RPC client
    let rpc_url = "https://api.mainnet-beta.solana.com"; // Replace with the correct RPC URL
    
    let config: RpcBlockConfig = RpcBlockConfig {
        encoding: Some(solana_transaction_status::UiTransactionEncoding::JsonParsed),
        transaction_details: Some(TransactionDetails::Full),
        rewards: Some(false),
        commitment: Some(CommitmentConfig { commitment: CommitmentLevel::Confirmed,}), //  If you ever need a higher level of assurance, you could change it to CommitmentLevel::Finalized, but Confirmed is fine for most real-time use cases.
        max_supported_transaction_version: Some(0),
    };

    let client = RpcClient::new(rpc_url.to_string());

    // Listen for incoming slot notifications
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                // Parse the incoming JSON message
                match serde_json::from_str::<Value>(&text) {
                    Ok(value) => {
                        // Extract the slot number from the message
                        if let Some(slot) = value.get("params").and_then(|params| {
                            params.get("result").and_then(|result| result.get("slot"))
                        }) {
                            info!("New slot received: {}", slot);

                            // Fetch and process transactions for the given slot
                            if let Some(slot_num) = slot.as_u64() {
                                let _ = fetch_block_with_retries(slot_num, &client, config).await;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing message: {:?}", e);
                    }
                }
            }
            Ok(Message::Ping(ping)) => {
                // Respond to PING messages to keep connection alive
                write
                    .send(Message::Pong(ping))
                    .await
                    .expect("Failed to send PONG");
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
            }
        }
    }
}



async fn fetch_block_with_retries(slot: u64, client: &RpcClient, config: RpcBlockConfig) -> Result<()> {
    // Define a retry strategy, with an exponential backoff
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter) // Jitter adds randomness to prevent retry spikes
        .take(5);    // Retry up to 5 times

    // Use Retry::spawn to retry the get_block_with_config call
    let result = Retry::spawn(retry_strategy, || {
        let client = client; 
        async move {
            // Attempt to get the block
            client
                .get_block_with_config(slot, config)
                .map_err(|e| e) // Pass the error without wrapping it in tokio_retry::Error
        }
    })
    .await?; // Await the result of the retries, propagate error if all retries fail

    // Handle the block if the request succeeds
    // You can process `result` (which is the fetched block) here
    info!("result{:?}", result);
    Ok(())
}











// // Function to fetch and process transactions for a given slot
// async fn fetch_and_process_transactions(slot: u64, client: &RpcClient, config: RpcBlockConfig) {
//     // Fetch confirmed block for the given slot
//     // sleep(Duration::from_secs(10)); // Poll every 10 seconds
//     // let result = client.get_block_with_encoding(slot, UiTransactionEncoding::JsonParsed);
//     let result = client.get_block_with_config(slot, config);

//     info!("result: {:?}", result);

//     if let Ok(block) = client.get_block_with_encoding(slot, UiTransactionEncoding::JsonParsed) {
//         info!("Block: {:?}", block);
//         let transactions = block.transactions;
//         for transaction in transactions {
//             if let Some(sig) = transaction.transaction.decode() {
//                 for signature in sig.signatures {
//                     info!("SIGNATURE: {:?}", signature);
//                     // process_transaction(signature, &client).await;
//                 }
//                 // Inspect each transaction and look for InitializeMint (new tokens) or custom pool init
//                 // process_transaction(transaction.transaction.signatures[0].clone(), &client).await;
//             }
//         }
//     }
// }








// // Process each transaction to check for pool initialization or token minting
// async fn process_transaction(signature: Signature, client: &RpcClient) {
//     // Fetch transaction details
//     if let Ok(transaction_details) =
//         client.get_transaction(&signature, UiTransactionEncoding::JsonParsed)
//     {
//         let transaction = transaction_details.transaction;
//         if let Some(meta) = transaction.meta {
//             let option_instructions = meta.inner_instructions;
//             let vec_instructions: Vec<UiInnerInstructions> = option_instructions.unwrap();
//             for instruction in vec_instructions {
//                 // Check if this instruction is related to the Token Program or pool initialization
//                 detect_new_pools_or_tokens(&instruction);
//             }
//         }
//     }
// }

// fn detect_new_pools_or_tokens(instruction: &UiInnerInstructions, transaction: &EncodedConfirmedTransactionWithStatusMeta) {
//     // Iterate through each compiled instruction in the inner instructions
//     for compiled_instruction in &instruction.instructions {
//         // Get the program ID from the index in transaction's `accountKeys`
//         if let Some(program_id) = get_program_id(compiled_instruction, transaction) {
//             // Check if it's the SPL Token program
//             if program_id == spl_token::id() {
//                 // Check if the instruction matches the InitializeMint pattern (or similar)
//                 if is_initialize_mint(&compiled_instruction.data) {
//                     println!("New token detected: {:?}", instruction);
//                 }
//             }
//             // Add custom logic to detect liquidity pool initialization (replace with actual program ID)
//             let custom_pool_program_id = Pubkey::from_str("custom-pool-program-id").unwrap(); // Replace with actual program ID
//             if program_id == custom_pool_program_id {
//                 println!("New liquidity pool detected: {:?}", instruction);
//             }
//         }
//     }
// }

// // Helper function to get the program ID from the instruction's program_id_index
// fn get_program_id(instruction: &CompiledInstruction, transaction: &EncodedConfirmedTransactionWithStatusMeta) -> Option<Pubkey> {
//     let program_id_index = instruction.program_id_index as usize;
//     transaction.transaction.message.account_keys.get(program_id_index).and_then(|key| Pubkey::from_str(key).ok())
// }

// // Helper function to detect if the instruction data matches the InitializeMint pattern
// fn is_initialize_mint(data: &str) -> bool {
//     // You can check the data structure for InitializeMint (SPL Token program) here.
//     // For simplicity, this function just returns true if the data length matches InitializeMint or other criteria
//     data.starts_with("initializeMint") // Adjust this to match actual data format
// }
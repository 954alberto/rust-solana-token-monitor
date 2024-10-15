use log::{debug, error, info, warn};
use serde_json::Value; // Import serde_json for parsing JSON data
use solana_client::rpc_client::{RpcClient, RpcClientConfig};
use solana_client::rpc_config::RpcBlockConfig;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::Signature;
use solana_sdk::{config, exit, transaction};
use solana_sdk::{pubkey::Pubkey, signature};
use solana_transaction_status::{
    EncodedConfirmedTransactionWithStatusMeta, TransactionDetails, UiInstruction,
    UiTransactionEncoding,
};
use std::env;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

/// Token Program ID for SPL Tokens on Solana
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    // Initialize the logger
    env_logger::init(); // Solana mainnet RPC URL
    let rpc_url: &str = "https://api.mainnet-beta.solana.com";

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
        if let Err(err) = monitor_new_tokens(&client, &config).await {
            eprintln!("Error in token monitoring: {}", err);
        }
        sleep(Duration::from_secs(10)).await; // Poll every 10 seconds
    }
}

/// Function to monitor for new token minting events
async fn monitor_new_tokens(client: &RpcClient, config: &RpcBlockConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Get the latest slot

    let latest_slot = client.get_slot()?;
    info!("Found slot: {:?}", latest_slot);
    // Try to fetch the confirmed block with retries
    let mut retries = 30; // Number of retries

    while retries > 0 {
        if let Ok(block) = client.get_block_with_config( latest_slot, *config) {
            info!("HELLO FROM THE IF");
            //info!("block : {:?}",block);
            if let Some(sig) = block.transactions {
                info!("HELLO FROM THE second IF");
                println!("{:?}",sig);

                //et test = sig.iter();
                for i in sig {
                    let te = i.transaction;
                    if let Some(te) = te.decode() {
                        let te = te.signatures;
                        println!("te {:?}",te);
                    }
                    // if let Some(te) = te {
                    //     println!("ESTE");
                    //     let sig = te.signatures;
                    //     for s in sig {
                    //         println!("s{:?}",s);
                    //     }
                    // }
                }
                // println!("SIG : {:?}",sig);
            }

            // info!("SIGNATURE {0:?}", block.signatures.);
            // println!("{0:?}", block);
            // let transactions = block.transactions;
            // println!("THIS IS THE signatures: {0:?}", transactions);

            // Process the block transactions
            // while let Some(ref transactions) = transactions {
            //     info!("HELLO FROM THE signatures");
            //     println!("THIS IS THE signatures: {0:?}", transactions);
            //     // if let Some(mint_event) = check_for_mint_event(&client, signatures).await? {
            //     //     println!("New token mint event detected: {:?}", mint_event);
            //     // }
            // }
            return Ok(());
        } else {
            eprintln!("Block not available for slot {}, retrying...", latest_slot);
            sleep(Duration::from_secs(10)).await; // Wait before retrying
                                                  //current_slot = client.get_slot()?; // Update latest slot
            retries -= 1;
        }
    }

    eprintln!("Failed to fetch block after multiple attempts.");

    // // Get the confirmed block for this slot
    // if let Ok(block) = client.get_block(latest_slot) {
    //     for tx in block.transactions {
    //         let signature = tx.transaction.decode().unwrap();
    //         println!("Signature: {signature:?}");

    //         // // Access the signature from the transaction's transaction field
    //         // let signature = tx.transaction.  .signatures[0].clone();
    //         // if let Some(mint_event) = check_for_mint_event(&client, signature).await? {
    //         //     println!("New token mint event detected: {:?}", mint_event);
    //         // }
    //     }
    // }
    Ok(())
}

/// Check if the transaction involves creating a new mint
async fn check_for_mint_event(
    client: &RpcClient,
    signature: Signature,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Fetch the transaction details using the transaction signature
    let transaction: EncodedConfirmedTransactionWithStatusMeta =
        client.get_transaction(&signature, UiTransactionEncoding::JsonParsed)?;

    println!("transaction {transaction:?}");

    //    if let Some(instructions) = transaction.transaction.message.instructions.as_ref() {

    // if let Some(instructions) = int.message.instructions().as_ref() {

    //     // Check if the transaction contains a "CreateMint" instruction for the Token Program
    //     for instruction in instructions.message {
    //         if let UiInstruction::Parsed(parsed_instruction) = instruction {
    //             // Check if the instruction's program matches the token program ID
    //             if parsed_instruction.program_id == TOKEN_PROGRAM_ID {
    //                 // Check for the specific minting action
    //                 if let Some(parsed) = &parsed_instruction.parsed {
    //                     if parsed.get("type") == Some(&Value::String("createMint".to_string())) {
    //                         return Ok(Some(format!("Mint created with signature: {}", signature)));
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
    Ok(None)
}




solana_block_to_block(block, block_number).map(Some)
}
}

fn solana_block_to_block(block: UiConfirmedBlock, slot_number: u64) -> Result<Block> {
fn calc_user_txs(block: &UiConfirmedBlock) -> u64 {
    let mut num_user_txs = 0;

    if let Some(block_txs) = &block.transactions {
        for tx_status in block_txs {
            let tx = tx_status.transaction.decode().unwrap();
            trace!("tx_meta: {:#?}", tx_status.meta.as_ref().unwrap());
            trace!("tx: {:#?}", tx);

            let account_keys = match &tx.message {
                VersionedMessage::Legacy(message) => &message.account_keys,
                VersionedMessage::V0(message) => &message.account_keys,
            };

            let mut num_vote_instrs = 0;
            for instr in tx.message.instructions() {
                let program_id_index = instr.program_id_index;
                let program_id = account_keys[usize::from(program_id_index)];

                if program_id == solana_sdk::vote::program::id() {
                    num_vote_instrs += 1;
                    trace!("found vote instruction");
                } else {
                    trace!("non-vote instruction");
                }
            }
            if num_vote_instrs == tx.message.instructions().len() {
                trace!("it's a vote transaction");
            } else {
                // This doesn't look like a vote transaction
                trace!("it's a non-vote transaction");
                num_user_txs += 1;
            }
        }

        let vote_txs = block_txs
            .len()
            .checked_sub(num_user_txs)
            .expect("underflow");

        debug!("solana total txs: {}", block_txs.len());
        debug!("solana user txs: {}", num_user_txs);
        debug!("solana vote txs: {}", vote_txs);
    } else {
        debug!("solana total txs: None");
    }

    u64::try_from(num_user_txs).expect("u64")
}

Ok(Block {
    chain: Chain::Solana,
    block_number: slot_number,
    prev_block_number: Some(block.parent_slot),
    timestamp: u64::try_from(
        block
            .block_time
            .ok_or_else(|| anyhow!("block time unavailable for solana slot {}", slot_number))?,
    )?,
    num_txs: calc_user_txs(&block),
    hash: block.blockhash,
    parent_hash: block.previous_blockhash,
})
}
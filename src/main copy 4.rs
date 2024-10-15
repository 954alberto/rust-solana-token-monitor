use futures::{stream::StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use serde_json::Value;
use url::Url;

#[tokio::main]
async fn main() {
    // WebSocket URL for Solana's RPC
    let ws_url = "wss://api.mainnet-beta.solana.com/".to_string(); // Replace with the correct WebSocket URL

    // Connect to the WebSocket
    let (ws_stream, _) = connect_async(Url::parse(&ws_url).unwrap()).await.expect("Failed to connect to WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to new slot notifications
    let subscription_message = r#"{"jsonrpc":"2.0","id":1,"method":"slotSubscribe","params":[]}"#;
    write.send(Message::Text(subscription_message.to_string())).await.expect("Failed to subscribe");

    // Listen for incoming messages
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                // Parse the incoming JSON message
                match serde_json::from_str::<Value>(&text) {
                    Ok(value) => {
                        // Extract the slot number from the message
                        if let Some(slot) = value.get("params").and_then(|params| params.get("result").and_then(|result| result.get("slot"))) {
                            println!("New slot received: {}", slot);
                            
                            // You can add further processing for this slot here
                            // For example, fetching transactions associated with this slot
                            // fetch_transactions(slot.as_u64().unwrap()).await;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error parsing message: {:?}", e);
                    }
                }
            }
            Ok(Message::Ping(ping)) => {
                // Respond to PING messages to keep connection alive
                write.send(Message::Pong(ping)).await.expect("Failed to send PONG");
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
            }
        }
    }
}

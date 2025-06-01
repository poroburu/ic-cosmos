use cosmos_utils::*;
use std::env;

fn print_usage() {
    println!("Usage:");
    println!("  build    - Build a new transaction and output the signing command");
    println!("  raw <message_type> - Generate wallet transaction and output sendCosmosTransaction command");
    println!("    message_type can be: send, delegate");
    println!("  broadcast <tx_base64> - Broadcast a signed transaction to the Cosmos Provider testnet");
    println!("  fund     - Print Gaia CLI command to fund the wallet from faucet");
    println!("\nExample:");
    println!("  cargo run -- build");
    println!("  cargo run -- raw send");
    println!("  cargo run -- raw delegate");
    println!("  cargo run -- broadcast \"CpABCo0BChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5k...\"");
    println!("  cargo run -- fund");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("build") => build_transaction()?,
        Some("raw") => {
            if let Some(message_type) = args.get(2) {
                match message_type.as_str() {
                    "send" => generate_raw_transaction(MessageType::Send)?,
                    "delegate" => generate_raw_transaction(MessageType::Delegate)?,
                    _ => {
                        println!(
                            "Error: Unsupported message type '{}'. Supported types: send, delegate",
                            message_type
                        );
                        print_usage();
                    }
                }
            } else {
                println!("Error: Message type required for raw command");
                print_usage();
            }
        }
        Some("broadcast") => {
            if let Some(tx_base64) = args.get(2) {
                broadcast_transaction(tx_base64)?
            } else {
                println!("Error: Transaction base64 required for broadcast command");
                print_usage();
            }
        }
        Some("fund") => print_fund_command()?,
        _ => print_usage(),
    }
    Ok(())
}

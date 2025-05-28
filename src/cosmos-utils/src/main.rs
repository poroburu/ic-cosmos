mod lib;
use std::env;
use lib::*;

fn print_usage() {
    println!("Usage:");
    println!("  build    - Build a new transaction and output the signing command");
    println!("  send <signature_blob> - Create and output a signed transaction in base64 format");
    println!("  broadcast <tx_base64> - Broadcast a signed transaction to the Cosmos Provider testnet");
    println!("  fund     - Print Gaia CLI command to fund the wallet from faucet");
    println!("\nExample:");
    println!("  cargo run -- build");
    println!("  cargo run -- send \"\\fd\\ce\\16\\49\\7b\\c7\\d5\\16\\37\\b8\\4c\\57\\42\\ea\\ed\\78\\44\\b3\\ce\\ec\\1b\\7a\\dc\\e9\\f3\\f5\\e4\\38\\03\\c7\\6c\\e4\\2a\\53\\c6\\42\\c5\\d2\\db\\f5\\e4\\f8\\69\\ca\\b9\\b2\\cc\\fc\\fa\\41\\c0\\63\\77\\7e\\bc\\99\\b0\\85\\18\\a1\\94\\c9\\c4\\ec\"");
    println!("  cargo run -- broadcast \"CpABCo0BChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5k...\"");
    println!("  cargo run -- fund");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("build") => build_transaction()?,
        Some("send") => {
            if let Some(signature) = args.get(2) {
                send_signed_transaction(signature)?
            } else {
                println!("Error: Signature required for send command");
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

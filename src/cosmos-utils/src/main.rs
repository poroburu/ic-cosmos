use cosmos_utils::*;
use hex;
use std::env;

fn print_usage() {
    println!("Usage:");
    println!("  build    - Build a new transaction and output the signing command");
    println!("  raw      - Generate raw unsigned transaction and save JSON to rawtx.json");
    println!("  send <signature_blob> - Create and output a signed transaction in base64 format");
    println!("  broadcast <tx_base64> - Broadcast a signed transaction to the Cosmos Provider testnet");
    println!("  fund     - Print Gaia CLI command to fund the wallet from faucet");
    println!("  test-query <cosmos_address> - Show protobuf encoding for account query");
    println!("\nExample:");
    println!("  cargo run -- build");
    println!("  cargo run -- raw");
    println!("  cargo run -- send \"\\fd\\ce\\16\\49\\7b\\c7\\d5\\16\\37\\b8\\4c\\57\\42\\ea\\ed\\78\\44\\b3\\ce\\ec\\1b\\7a\\dc\\e9\\f3\\f5\\e4\\38\\03\\c7\\6c\\e4\\2a\\53\\c6\\42\\c5\\d2\\db\\f5\\e4\\f8\\69\\ca\\b9\\b2\\cc\\fc\\fa\\41\\c0\\63\\77\\7e\\bc\\99\\b0\\85\\18\\a1\\94\\c9\\c4\\ec\"");
    println!("  cargo run -- broadcast \"CpABCo0BChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5k...\"");
    println!("  cargo run -- fund");
    println!("  cargo run -- test-query cosmos17xqjqfljz4aq6nurwg9r3r9l7gxtajz0hq3ewf");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("build") => build_transaction()?,
        Some("raw") => generate_raw_transaction()?,
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
        Some("test-query") => {
            if let Some(address) = args.get(2) {
                println!("=== Cosmos Account Query Protobuf Analysis ===");
                println!("Address: {}", address);
                println!("Address length: {} characters", address.len());
                println!();

                // Show the raw address bytes
                let address_bytes = address.as_bytes();
                println!("Raw address bytes:");
                println!("  ASCII: {:?}", String::from_utf8_lossy(address_bytes));
                println!("  Hex: {}", hex::encode(address_bytes));
                println!("  Length: {} bytes", address_bytes.len());
                println!();

                // Show protobuf structure before encoding
                println!("Protobuf structure (before encoding):");
                println!("  Field 1 (tag 0x0a = field 1, wire type 2 = length-delimited):");
                println!("    Length: 0x{:02x} ({} bytes)", address.len(), address.len());
                println!("    Value: {}", address);
                println!();

                // Show the encoded query data
                let query_data = format!("0a{:02x}{}", address.len(), hex::encode(address_bytes));
                println!("Encoded query data (after protobuf encoding):");
                println!("  Full hex: {}", query_data);
                println!("  Breakdown:");
                println!("    0a     = Field tag (field 1, wire type 2)");
                println!(
                    "    {:02x}     = Length prefix ({} bytes)",
                    address.len(),
                    address.len()
                );
                println!("    {}... = Hex-encoded address", &hex::encode(address_bytes)[..16]);
                println!();

                // Show curl command
                println!("Equivalent curl command:");
                println!("curl -X POST https://rpc.testcosmos.directory/cosmosicsprovidertestnet \\");
                println!("  -H \"Content-Type: application/json\" \\");
                println!("  -d '{{");
                println!("    \"jsonrpc\": \"2.0\",");
                println!("    \"id\": 1,");
                println!("    \"method\": \"abci_query\",");
                println!("    \"params\": {{");
                println!("      \"path\": \"/cosmos.auth.v1beta1.Query/Account\",");
                println!("      \"data\": \"{}\",", query_data);
                println!("      \"height\": \"0\",");
                println!("      \"prove\": false");
                println!("    }}");
                println!("  }}'");
                println!();

                // Actually make the RPC call and analyze the response
                println!("=== Making RPC Call and Analyzing Response ===");
                match get_account_info(address) {
                    Ok((account_number, sequence)) => {
                        println!("✅ Account found!");
                        println!("Account Number: {}", account_number);
                        println!("Sequence: {}", sequence);
                        println!();

                        // Let's also show the detailed response analysis
                        println!("=== Detailed Response Analysis ===");
                        if let Ok(response_details) = analyze_account_response(address) {
                            println!("{}", response_details);
                        }
                    }
                    Err(e) => {
                        println!("❌ Error querying account: {}", e);
                        println!("This could mean:");
                        println!("  - Account doesn't exist (needs to be funded first)");
                        println!("  - Invalid address format");
                        println!("  - Network connectivity issues");
                    }
                }
            } else {
                println!("Error: Cosmos address required for test-query command");
                print_usage();
            }
        }
        _ => print_usage(),
    }
    Ok(())
}

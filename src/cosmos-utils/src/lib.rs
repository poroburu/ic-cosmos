//! Core Cosmos utilities logic extracted from main.rs for testability and reuse.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use bech32::{self, ToBase32};
use bs58;
use cosmos_sdk_proto::cosmos::{
    bank::v1beta1::MsgSend,
    base::v1beta1::Coin,
    crypto::secp256k1::PubKey,
    staking::v1beta1::MsgDelegate,
    tx::signing::v1beta1::SignMode,
    tx::v1beta1::{AuthInfo, Fee, ModeInfo, SignerInfo, Tx, TxBody},
};
use hex;
use prost::Message;
use prost_types::Any;
use reqwest::blocking::Client;
use ripemd::Ripemd160;
use serde_json::json;
use sha2::{Digest as Sha256Digest, Sha256};
use std::error::Error;
use std::process::Command;

/// Supported message types for transaction generation
#[derive(Debug, Clone)]
pub enum MessageType {
    Send,
    Delegate,
}

#[derive(Message)]
pub struct SignDoc {
    #[prost(bytes, tag = "1")]
    pub body_bytes: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub auth_info_bytes: Vec<u8>,
    #[prost(string, tag = "3")]
    pub chain_id: String,
    #[prost(uint64, tag = "4")]
    pub account_number: u64,
}

#[derive(Message)]
pub struct QueryAccountResponse {
    #[prost(message, optional, tag = "1")]
    pub account: Option<Any>,
}

#[derive(Message)]
pub struct BaseAccount {
    #[prost(string, tag = "1")]
    pub address: String,
    #[prost(message, optional, tag = "2")]
    pub pub_key: Option<Any>,
    #[prost(uint64, tag = "3")]
    pub account_number: u64,
    #[prost(uint64, tag = "4")]
    pub sequence: u64,
}

pub fn public_key_to_cosmos_address(public_key: &str) -> Result<String, Box<dyn Error>> {
    let decoded = bs58::decode(public_key).into_vec()?;
    let mut hasher = Sha256::new();
    hasher.update(&decoded);
    let sha256_hash = hasher.finalize();
    let mut hasher = Ripemd160::new();
    hasher.update(sha256_hash);
    let ripemd160_hash = hasher.finalize();
    let data = ripemd160_hash.to_vec();
    let encoded = bech32::encode("cosmos", data.to_base32(), bech32::Variant::Bech32)?;
    Ok(encoded)
}

pub fn create_send_transaction(
    from_address: &str,
    to_address: &str,
    amount: u64,
    signature: Option<Vec<u8>>,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let msg_send = MsgSend {
        from_address: from_address.to_string(),
        to_address: to_address.to_string(),
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: amount.to_string(),
        }],
    };
    let msg_any = Any {
        type_url: "/cosmos.bank.v1beta1.MsgSend".to_string(),
        value: msg_send.encode_to_vec(),
    };
    let tx_body = TxBody {
        messages: vec![msg_any],
        memo: "".to_string(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };
    let public_key = get_public_key_from_canister()?;
    let pub_key = PubKey {
        key: bs58::decode(&public_key).into_vec()?,
    };
    let pub_key_any = Any {
        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
        value: pub_key.encode_to_vec(),
    };
    let fee = Fee {
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "1000".to_string(),
        }],
        gas_limit: 200000,
        payer: "".to_string(),
        granter: "".to_string(),
    };

    // Get account info
    let (account_number, sequence) = get_account_info(from_address)?;

    let auth_info = AuthInfo {
        signer_infos: vec![SignerInfo {
            public_key: Some(pub_key_any),
            mode_info: Some(ModeInfo {
                sum: Some(cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Sum::Single(
                    cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Single {
                        mode: SignMode::Direct as i32,
                    },
                )),
            }),
            sequence,
        }],
        fee: Some(fee),
        tip: None,
    };

    // Create sign doc with account number
    let sign_doc = SignDoc {
        body_bytes: tx_body.encode_to_vec(),
        auth_info_bytes: auth_info.encode_to_vec(),
        chain_id: "provider".to_string(),
        account_number,
    };

    let sign_bytes = sign_doc.encode_to_vec();
    let mut tx = Tx {
        body: Some(tx_body),
        auth_info: Some(auth_info),
        signatures: vec![],
    };
    if let Some(sig) = signature {
        if sig.len() < 64 {
            eprintln!(
                "Error: signature length is {} bytes, expected at least 64. Signature (hex): {}",
                sig.len(),
                hex::encode(&sig)
            );
            return Err(format!("Signature too short: got {} bytes, expected at least 64", sig.len()).into());
        }
        // Use the raw 64-byte signature as Cosmos expects
        tx.signatures = vec![sig[..64].to_vec()];
    }
    Ok((tx.encode_to_vec(), sign_bytes))
}

pub fn create_delegate_transaction(
    delegator_address: &str,
    validator_address: &str,
    amount: u64,
    signature: Option<Vec<u8>>,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let msg_delegate = MsgDelegate {
        delegator_address: delegator_address.to_string(),
        validator_address: validator_address.to_string(),
        amount: Some(Coin {
            denom: "uatom".to_string(),
            amount: amount.to_string(),
        }),
    };

    let msg_any = Any {
        type_url: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
        value: msg_delegate.encode_to_vec(),
    };

    let tx_body = TxBody {
        messages: vec![msg_any],
        memo: "Delegate to validator".to_string(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let public_key = get_public_key_from_canister()?;
    let pub_key = PubKey {
        key: bs58::decode(&public_key).into_vec()?,
    };
    let pub_key_any = Any {
        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
        value: pub_key.encode_to_vec(),
    };
    let fee = Fee {
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "5000".to_string(),
        }],
        gas_limit: 324000,
        payer: "".to_string(),
        granter: "".to_string(),
    };

    // Get account info
    let (account_number, sequence) = get_account_info(delegator_address)?;

    let auth_info = AuthInfo {
        signer_infos: vec![SignerInfo {
            public_key: Some(pub_key_any),
            mode_info: Some(ModeInfo {
                sum: Some(cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Sum::Single(
                    cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Single {
                        mode: SignMode::Direct as i32,
                    },
                )),
            }),
            sequence,
        }],
        fee: Some(fee),
        tip: None,
    };

    // Create sign doc with account number
    let sign_doc = SignDoc {
        body_bytes: tx_body.encode_to_vec(),
        auth_info_bytes: auth_info.encode_to_vec(),
        chain_id: "provider".to_string(),
        account_number,
    };

    let sign_bytes = sign_doc.encode_to_vec();
    let mut tx = Tx {
        body: Some(tx_body),
        auth_info: Some(auth_info),
        signatures: vec![],
    };
    if let Some(sig) = signature {
        if sig.len() < 64 {
            eprintln!(
                "Error: signature length is {} bytes, expected at least 64. Signature (hex): {}",
                sig.len(),
                hex::encode(&sig)
            );
            return Err(format!("Signature too short: got {} bytes, expected at least 64", sig.len()).into());
        }
        // Use the raw 64-byte signature as Cosmos expects
        tx.signatures = vec![sig[..64].to_vec()];
    }
    Ok((tx.encode_to_vec(), sign_bytes))
}

pub fn get_public_key_from_canister() -> Result<String, Box<dyn Error>> {
    let output = Command::new("dfx")
        .args(["canister", "call", "cosmos_wallet", "address"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    // Remove quotes and whitespace from the output
    let public_key = stdout
        .trim()
        .trim_matches(|c| c == '(' || c == ')' || c == '"' || c == ' ');
    Ok(public_key.to_string())
}

pub fn get_cosmos_address_from_canister() -> Result<String, Box<dyn Error>> {
    let output = Command::new("dfx")
        .args(["canister", "call", "cosmos_wallet", "cosmosAddress"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Parse the result from (variant { Ok = "cosmos1..." })
    if let Some(start) = stdout.find("Ok = \"") {
        let address_start = start + 6; // Skip 'Ok = "'
        if let Some(end) = stdout[address_start..].find('"') {
            let cosmos_address = &stdout[address_start..address_start + end];
            return Ok(cosmos_address.to_string());
        }
    }

    // If we can't parse the success format, check for error
    if stdout.contains("Err") {
        return Err("Failed to get cosmos address from canister".into());
    }

    Err("Unexpected response format from cosmosAddress canister call".into())
}

pub fn get_signature_from_canister(sign_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let output = Command::new("dfx")
        .args([
            "canister",
            "call",
            "--update",
            "cosmos_wallet",
            "signMessage",
            &format!(
                "(blob \"{}\")",
                sign_bytes.iter().map(|b| format!("\\{:02X}", b)).collect::<String>()
            ),
        ])
        .output()?;

    eprintln!("Canister stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("Canister stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8(output.stdout)?;

    // Extract the blob content between quotes
    let blob_content = stdout
        .split("blob \"")
        .nth(1)
        .ok_or_else(|| "Failed to find blob content")?
        .split("\"")
        .next()
        .ok_or_else(|| "Failed to parse blob content")?;

    // Convert escaped hex sequences to bytes
    let signature = blob_content
        .split("\\")
        .filter(|s| !s.is_empty())
        .map(|s| {
            // Handle special cases like \n, \r, etc.
            if s.starts_with('n') {
                Ok(b'\n')
            } else if s.starts_with('r') {
                Ok(b'\r')
            } else if s.starts_with('t') {
                Ok(b'\t')
            } else {
                // Parse hex bytes
                u8::from_str_radix(s, 16).map_err(|e| format!("Failed to parse hex: {}", e).into())
            }
        })
        .collect::<Result<Vec<u8>, Box<dyn Error>>>()?;

    Ok(signature)
}

pub fn print_transaction_json(tx_bytes: &[u8], title: &str, pretty: bool) -> Result<String, Box<dyn Error>> {
    if let Ok(tx) = Tx::decode(&tx_bytes[..]) {
        let json_obj = json!({
            "body": {
                "messages": tx.body.as_ref().map(|b| &b.messages).unwrap_or(&vec![]).iter().map(|msg| {
                    // Decode specific message types
                    match msg.type_url.as_str() {
                        "/cosmos.bank.v1beta1.MsgSend" => {
                            if let Ok(msg_send) = MsgSend::decode(msg.value.as_slice()) {
                                json!({
                                    "@type": msg.type_url,
                                    "from_address": msg_send.from_address,
                                    "to_address": msg_send.to_address,
                                    "amount": msg_send.amount.iter().map(|coin| json!({
                                        "denom": coin.denom,
                                        "amount": coin.amount
                                    })).collect::<Vec<_>>()
                                })
                            } else {
                                json!({
                                    "@type": msg.type_url,
                                    "value": STANDARD.encode(&msg.value)
                                })
                            }
                        },
                        "/cosmos.staking.v1beta1.MsgDelegate" => {
                            if let Ok(msg_delegate) = MsgDelegate::decode(msg.value.as_slice()) {
                                json!({
                                    "@type": msg.type_url,
                                    "delegator_address": msg_delegate.delegator_address,
                                    "validator_address": msg_delegate.validator_address,
                                    "amount": msg_delegate.amount.map(|coin| json!({
                                        "denom": coin.denom,
                                        "amount": coin.amount
                                    })).unwrap_or(json!({}))
                                })
                            } else {
                                json!({
                                    "@type": msg.type_url,
                                    "value": STANDARD.encode(&msg.value)
                                })
                            }
                        },
                        _ => {
                            // For unknown message types, fall back to base64 encoding
                            json!({
                                "@type": msg.type_url,
                                "value": STANDARD.encode(&msg.value)
                            })
                        }
                    }
                }).collect::<Vec<_>>(),
                "memo": tx.body.as_ref().map(|b| &b.memo).unwrap_or(&String::new()),
                "timeout_height": tx.body.as_ref().map(|b| b.timeout_height).unwrap_or(0).to_string(),
                "extension_options": [],
                "non_critical_extension_options": []
            },
            "auth_info": {
                "signer_infos": tx.auth_info.as_ref().map(|a| &a.signer_infos).unwrap_or(&vec![]).iter().map(|signer| {
                    json!({
                        "public_key": signer.public_key.as_ref().map(|pk| json!({
                            "@type": pk.type_url,
                            "key": STANDARD.encode(&pk.value)
                        })).unwrap_or(json!(null)),
                        "mode_info": {
                            "single": {
                                "mode": "SIGN_MODE_DIRECT"
                            }
                        },
                        "sequence": signer.sequence.to_string()
                    })
                }).collect::<Vec<_>>(),
                "fee": tx.auth_info.as_ref().and_then(|a| a.fee.as_ref()).map(|f| json!({
                    "amount": f.amount.iter().map(|coin| json!({
                        "denom": coin.denom,
                        "amount": coin.amount
                    })).collect::<Vec<_>>(),
                    "gas_limit": f.gas_limit.to_string(),
                    "payer": f.payer,
                    "granter": f.granter
                })).unwrap_or_else(|| json!({
                    "amount": [],
                    "gas_limit": "200000",
                    "payer": "",
                    "granter": ""
                })),
                "tip": null
            },
            "signatures": tx.signatures.iter().map(|sig| STANDARD.encode(sig)).collect::<Vec<_>>()
        });

        let json_output = if pretty {
            serde_json::to_string_pretty(&json_obj)?
        } else {
            serde_json::to_string(&json_obj)?
        };

        if !title.is_empty() {
            println!("{}:", title);
        }
        println!("{}", json_output);
        Ok(json_output)
    } else {
        Err("Failed to decode transaction".into())
    }
}

/// Helper function to encode varint
fn encode_varint(value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut val = value;
    while val >= 0x80 {
        result.push(((val & 0x7F) | 0x80) as u8);
        val >>= 7;
    }
    result.push(val as u8);
    result
}

/// Helper function to encode length-delimited field
fn encode_length_delimited(tag: u8, data: &[u8]) -> Vec<u8> {
    let mut result = vec![tag];
    result.extend(encode_varint(data.len() as u64));
    result.extend(data);
    result
}

/// Estimate gas for a transaction by simulating it
pub fn estimate_gas_for_transaction(transaction_json: &serde_json::Value) -> Result<u64, Box<dyn Error>> {
    // Get public key and cosmos address for simulation
    let public_key = get_public_key_from_canister()?;
    let cosmos_address = public_key_to_cosmos_address(&public_key)?;
    let pk_bytes = bs58::decode(&public_key).into_vec()?;

    // Get account info for simulation
    let (account_number, sequence) = get_account_info(&cosmos_address)?;

    // Build a complete transaction for simulation
    let messages_array = transaction_json["body"]["messages"]
        .as_array()
        .ok_or("Missing messages array")?;

    if messages_array.is_empty() {
        return Err("No messages to simulate".into());
    }

    // Create messages for the transaction body
    let mut tx_messages = Vec::new();

    // Encode each message for the simulation
    for msg_json in messages_array {
        let type_url = msg_json["@type"].as_str().ok_or("Missing @type")?;

        let msg_bytes = match type_url {
            "/cosmos.bank.v1beta1.MsgSend" => {
                let msg_send = MsgSend {
                    from_address: msg_json["from_address"].as_str().unwrap_or("").to_string(),
                    to_address: msg_json["to_address"].as_str().unwrap_or("").to_string(),
                    amount: msg_json["amount"]
                        .as_array()
                        .unwrap_or(&Vec::new())
                        .iter()
                        .map(|coin| Coin {
                            denom: coin["denom"].as_str().unwrap_or("").to_string(),
                            amount: coin["amount"].as_str().unwrap_or("0").to_string(),
                        })
                        .collect(),
                };
                msg_send.encode_to_vec()
            }
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                let msg_delegate = MsgDelegate {
                    delegator_address: msg_json["delegator_address"].as_str().unwrap_or("").to_string(),
                    validator_address: msg_json["validator_address"].as_str().unwrap_or("").to_string(),
                    amount: Some(Coin {
                        denom: msg_json["amount"]["denom"].as_str().unwrap_or("uatom").to_string(),
                        amount: msg_json["amount"]["amount"].as_str().unwrap_or("0").to_string(),
                    }),
                };
                msg_delegate.encode_to_vec()
            }
            _ => return Err(format!("Unsupported message type for simulation: {}", type_url).into()),
        };

        let msg_any = Any {
            type_url: type_url.to_string(),
            value: msg_bytes,
        };
        tx_messages.push(msg_any);
    }

    // Create transaction body with proper messages
    let memo = transaction_json["body"]["memo"].as_str().unwrap_or("");
    let tx_body = TxBody {
        messages: tx_messages,
        memo: memo.to_string(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    // Create public key Any
    let pub_key = PubKey { key: pk_bytes.clone() };
    let pub_key_any = Any {
        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
        value: pub_key.encode_to_vec(),
    };

    // Create fee (start with minimal fee for simulation)
    let fee = Fee {
        amount: vec![Coin {
            denom: "uatom".to_string(),
            amount: "1000".to_string(),
        }],
        gas_limit: 1000000, // High limit for simulation
        payer: "".to_string(),
        granter: "".to_string(),
    };

    // Create auth info
    let auth_info = AuthInfo {
        signer_infos: vec![SignerInfo {
            public_key: Some(pub_key_any),
            mode_info: Some(ModeInfo {
                sum: Some(cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Sum::Single(
                    cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Single {
                        mode: SignMode::Direct as i32,
                    },
                )),
            }),
            sequence,
        }],
        fee: Some(fee),
        tip: None,
    };

    // Create sign doc for signing
    let sign_doc = SignDoc {
        body_bytes: tx_body.encode_to_vec(),
        auth_info_bytes: auth_info.encode_to_vec(),
        chain_id: "provider".to_string(),
        account_number,
    };

    let sign_bytes = sign_doc.encode_to_vec();

    // Get signature from canister for simulation
    println!("Getting signature from canister for simulation...");
    let signature = get_signature_from_canister(&sign_bytes)?;

    // Ensure signature is 64 bytes (truncate if longer)
    let signature = if signature.len() >= 64 {
        signature[..64].to_vec()
    } else {
        return Err(format!(
            "Signature too short: got {} bytes, expected at least 64",
            signature.len()
        )
        .into());
    };

    // Build tx for simulation (with signatures)
    let tx = Tx {
        body: Some(tx_body),
        auth_info: Some(auth_info),
        signatures: vec![signature], // Include real signature for simulation
    };

    let tx_bytes = tx.encode_to_vec();

    // Create SimulateRequest protobuf - just wrap the tx in a length-delimited field
    let simulate_request_bytes = encode_length_delimited(0x0a, &tx_bytes);
    let query_data = hex::encode(&simulate_request_bytes);

    // Make the simulation RPC call
    let client = Client::new();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "abci_query",
        "params": {
            "path": "/cosmos.tx.v1beta1.Service/Simulate",
            "data": query_data,
            "height": "0",
            "prove": false
        }
    });

    println!("Making simulation request with real signature...");
    let response = client
        .post("https://rpc.testcosmos.directory/cosmosicsprovidertestnet")
        .json(&request)
        .send()?;

    let response_json: serde_json::Value = response.json()?;

    // Check for error
    if let Some(error) = response_json.get("error") {
        return Err(format!("Simulation RPC error: {}", error).into());
    }

    let result_code = response_json["result"]["response"]["code"].as_i64().unwrap_or(-1);
    if result_code != 0 {
        let log = response_json["result"]["response"]["log"]
            .as_str()
            .unwrap_or("Unknown error");
        println!("Simulation failed: {}", log);
        // Fall back to conservative estimate
        let message_type = messages_array[0]["@type"].as_str().unwrap_or("");
        let fallback_gas = match message_type {
            "/cosmos.bank.v1beta1.MsgSend" => 125_000u64, // Updated based on actual usage: ~97k-104k
            "/cosmos.staking.v1beta1.MsgDelegate" => 350_000u64, // Updated based on actual usage: ~324k-344k
            _ => 250_000u64,
        };
        println!("Using fallback estimate: {}", fallback_gas);
        return Ok(fallback_gas);
    }

    // Parse simulation response
    let response_value = response_json["result"]["response"]["value"]
        .as_str()
        .ok_or("Missing simulation response value")?;

    let decoded = STANDARD.decode(response_value)?;

    // Parse SimulateResponse to extract gas_used
    // The structure should be: SimulateResponse { gas_info: { gas_wanted, gas_used }, result }

    println!("Simulation successful! Parsing gas usage...");
    let decoded_hex = hex::encode(&decoded);
    println!(
        "Simulation response (hex): {}",
        &decoded_hex[..std::cmp::min(200, decoded_hex.len())]
    );

    // Try to extract gas_used from the response
    // Look for gas_used field (tag 0x10) in the response
    if decoded.len() >= 16 {
        // Look for gas_used pattern in the first part of the response
        for i in 0..std::cmp::min(50, decoded.len() - 8) {
            if decoded[i] == 0x10 {
                // gas_used field tag
                let mut pos = i + 1;
                if let Ok(gas_used) = read_varint_at(&decoded, &mut pos) {
                    if gas_used > 50_000 && gas_used < 2_000_000 {
                        // Reasonable range
                        // Use different buffer multipliers based on message type
                        let message_type = messages_array[0]["@type"].as_str().unwrap_or("");
                        let buffer_multiplier = match message_type {
                            "/cosmos.bank.v1beta1.MsgSend" => 1.25, // Send needs more buffer due to variability
                            "/cosmos.staking.v1beta1.MsgDelegate" => 1.15, // Delegate is more predictable
                            _ => 1.2,
                        };
                        let with_buffer = (gas_used as f64 * buffer_multiplier) as u64;
                        println!(
                            "✅ Simulated gas_used: {}, recommended: {} ({}x buffer)",
                            gas_used, with_buffer, buffer_multiplier
                        );
                        return Ok(with_buffer);
                    }
                }
            }
        }
    }

    // If simulation parsing fails, fall back to conservative estimate
    let message_type = messages_array[0]["@type"].as_str().unwrap_or("");
    let fallback_gas = match message_type {
        "/cosmos.bank.v1beta1.MsgSend" => 125_000u64, // Updated based on actual usage: ~97k-104k
        "/cosmos.staking.v1beta1.MsgDelegate" => 350_000u64, // Updated based on actual usage: ~324k-344k
        _ => 250_000u64,
    };
    println!("Simulation parsing failed, using fallback estimate: {}", fallback_gas);
    Ok(fallback_gas)
}

/// Helper function to read varint at specific position
fn read_varint_at(data: &[u8], pos: &mut usize) -> Result<u64, Box<dyn Error>> {
    let mut value = 0u64;
    let mut shift = 0;

    while *pos < data.len() {
        let byte = data[*pos];
        *pos += 1;

        value |= ((byte & 0x7F) as u64) << shift;

        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;

        if shift >= 64 {
            return Err("Varint too long".into());
        }
    }

    Err("Unexpected end of data while reading varint".into())
}

/// Calculate appropriate fee based on gas limit
pub fn calculate_fee_for_gas(gas_limit: u64, gas_price: f64) -> u64 {
    (gas_limit as f64 * gas_price).ceil() as u64
}

pub fn generate_raw_transaction(message_type: MessageType) -> Result<(), Box<dyn Error>> {
    let cosmos_address = get_cosmos_address_from_canister()?;
    println!("Cosmos address: {}", cosmos_address);

    // First, create a base transaction to estimate gas
    let base_json = match message_type {
        MessageType::Send => {
            println!("Generating MsgSend transaction for IC Cosmos wallet...");
            json!({
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.bank.v1beta1.MsgSend",
                            "from_address": cosmos_address,
                            "to_address": cosmos_address,
                            "amount": [
                                {
                                    "denom": "uatom",
                                    "amount": "1000"
                                }
                            ]
                        }
                    ]
                }
            })
        }
        MessageType::Delegate => {
            println!("Generating MsgDelegate transaction for IC Cosmos wallet...");
            let validator_address = "cosmosvaloper1e5yfpc8l6g4808fclmlyd38tjgxuwshnmjkrq6";
            println!("Validator address: {}", validator_address);
            json!({
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.staking.v1beta1.MsgDelegate",
                            "delegator_address": cosmos_address,
                            "validator_address": validator_address,
                            "amount": {
                                "denom": "uatom",
                                "amount": "1000"
                            }
                        }
                    ]
                }
            })
        }
    };

    // Estimate gas requirement
    let estimated_gas = estimate_gas_for_transaction(&base_json)?;
    let gas_limit = estimated_gas.to_string();

    // Calculate fee (using 0.01 uatom per gas unit, optimized for lower fees)
    let gas_price = 0.01;
    let fee_amount = calculate_fee_for_gas(estimated_gas, gas_price);

    println!("Estimated gas needed: {}", estimated_gas);
    println!("Calculated fee: {} uatom", fee_amount);

    let json_obj = match message_type {
        MessageType::Send => {
            json!({
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.bank.v1beta1.MsgSend",
                            "from_address": cosmos_address,
                            "to_address": cosmos_address,
                            "amount": [
                                {
                                    "denom": "uatom",
                                    "amount": "1000"
                                }
                            ]
                        }
                    ],
                    "memo": "Send transaction",
                    "timeout_height": "0",
                    "extension_options": [],
                    "non_critical_extension_options": []
                },
                "auth_info": {
                    "signer_infos": [],
                    "fee": {
                        "amount": [
                            {
                                "denom": "uatom",
                                "amount": fee_amount.to_string()
                            }
                        ],
                        "gas_limit": gas_limit,
                        "payer": "",
                        "granter": ""
                    }
                },
                "signatures": []
            })
        }
        MessageType::Delegate => {
            let validator_address = "cosmosvaloper1e5yfpc8l6g4808fclmlyd38tjgxuwshnmjkrq6";
            json!({
                "body": {
                    "messages": [
                        {
                            "@type": "/cosmos.staking.v1beta1.MsgDelegate",
                            "delegator_address": cosmos_address,
                            "validator_address": validator_address,
                            "amount": {
                                "denom": "uatom",
                                "amount": "1000"
                            }
                        }
                    ],
                    "memo": "Delegate to validator",
                    "timeout_height": "0",
                    "extension_options": [],
                    "non_critical_extension_options": []
                },
                "auth_info": {
                    "signer_infos": [],
                    "fee": {
                        "amount": [
                            {
                                "denom": "uatom",
                                "amount": fee_amount.to_string()
                            }
                        ],
                        "gas_limit": gas_limit,
                        "payer": "",
                        "granter": ""
                    }
                },
                "signatures": []
            })
        }
    };

    let compact_json = serde_json::to_string(&json_obj)?;
    let escaped_json = compact_json.replace("\"", "\\\"");

    println!("\nTransaction JSON:");
    println!("{}", serde_json::to_string_pretty(&json_obj)?);

    println!("\nTo send this transaction with the IC Cosmos wallet, run:");
    println!("dfx canister call cosmos_wallet sendCosmosTransaction '(variant {{ Testnet }}, null, \"provider\", \"{}\")' --update", escaped_json);

    Ok(())
}

pub fn build_transaction() -> Result<(), Box<dyn Error>> {
    // Show address generation
    println!("=== Address Generation ===");

    println!("Generated with: dfx canister call cosmos_wallet address");
    let public_key = get_public_key_from_canister()?;
    println!("Secp256k1 public key: {}", public_key);
    println!("Generated with: dfx canister call cosmos_wallet cosmosAddress");
    let cosmos_address = get_cosmos_address_from_canister()?;
    println!("Cosmos address: {}", cosmos_address);

    // Show the transaction structure in JSON format
    println!("\n=== Raw Transaction ===");
    let (tx_bytes, sign_bytes) = create_send_transaction(&cosmos_address, &cosmos_address, 1000, None)?;
    print_transaction_json(&tx_bytes, "", true)?;

    // Decode and display the SignDoc structure in human-readable format
    println!("\n=== What's Being Signed (Canonical Sign Document) ===");
    if let Ok(sign_doc) = SignDoc::decode(&sign_bytes[..]) {
        println!("Chain ID: {}", sign_doc.chain_id);
        println!("Account Number: {}", sign_doc.account_number);

        // Decode and display the body
        if let Ok(tx_body) = TxBody::decode(sign_doc.body_bytes.as_slice()) {
            println!("Transaction Body:");
            if !tx_body.memo.is_empty() {
                println!("  Memo: \"{}\"", tx_body.memo);
            }
            println!("  Messages: {} message(s)", tx_body.messages.len());

            for (i, msg) in tx_body.messages.iter().enumerate() {
                println!("    Message {}: {}", i + 1, msg.type_url);

                // Decode specific message types for better readability
                match msg.type_url.as_str() {
                    "/cosmos.bank.v1beta1.MsgSend" => {
                        if let Ok(msg_send) = MsgSend::decode(msg.value.as_slice()) {
                            println!("      From: {}", msg_send.from_address);
                            println!("      To: {}", msg_send.to_address);
                            for coin in &msg_send.amount {
                                println!("      Amount: {} {}", coin.amount, coin.denom);
                            }
                        }
                    }
                    "/cosmos.staking.v1beta1.MsgDelegate" => {
                        if let Ok(msg_delegate) = MsgDelegate::decode(msg.value.as_slice()) {
                            println!("      Delegator: {}", msg_delegate.delegator_address);
                            println!("      Validator: {}", msg_delegate.validator_address);
                            if let Some(amount) = &msg_delegate.amount {
                                println!("      Amount: {} {}", amount.amount, amount.denom);
                            }
                        }
                    }
                    _ => {
                        println!("      Value: {} bytes (binary data)", msg.value.len());
                    }
                }
            }
        }

        // Decode and display the auth info
        if let Ok(auth_info) = AuthInfo::decode(sign_doc.auth_info_bytes.as_slice()) {
            if let Some(fee) = &auth_info.fee {
                println!("Fee & Gas:");
                for coin in &fee.amount {
                    println!("  Fee: {} {}", coin.amount, coin.denom);
                }
                println!("  Gas Limit: {}", fee.gas_limit);
            }

            for (i, signer) in auth_info.signer_infos.iter().enumerate() {
                println!("Signer {}: sequence {}", i + 1, signer.sequence);
            }
        }
    } else {
        println!("Failed to decode SignDoc structure");
    }

    // Show the canonical sign document encodings
    println!("\nCanonical sign document encodings:");
    println!("Base64: {}", STANDARD.encode(&sign_bytes));
    println!("Hex: {}", hex::encode(&sign_bytes));

    // Show the canister call command
    println!("\nTo sign this document with the canister:");
    println!("dfx canister call --update cosmos_wallet signMessage \\");
    println!(
        "  '(blob \"{}\")'",
        sign_bytes.iter().map(|b| format!("\\{:02X}", b)).collect::<String>()
    );

    println!("\nGetting signature from canister...");
    let signature = get_signature_from_canister(&sign_bytes)?;

    // Show signature encoding details
    println!("\n=== Signature Details ===");
    println!("Raw signature (hex): {}", hex::encode(&signature));
    println!("Signature (base64): {}", STANDARD.encode(&signature));

    println!("\nCreating signed transaction...");
    let (final_tx, _) = create_send_transaction(&cosmos_address, &cosmos_address, 1000, Some(signature))?;

    // Show the final signed transaction in JSON format only
    print_transaction_json(&final_tx, "Final Signed Transaction", true)?;

    println!("\n=== Broadcasting Options ===");
    println!("Option 1 - Using cosmos-utils:");
    println!("cargo run -- broadcast \"{}\"", STANDARD.encode(&final_tx));

    println!("\nOption 2 - Direct HTTP broadcast:");
    println!("curl -X POST https://rpc.testcosmos.directory/cosmosicsprovidertestnet \\");
    println!("  -H \"Content-Type: application/json\" \\");
    println!(
        "  -d '{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"broadcast_tx_sync\",\"params\":{{\"tx\":\"{}\"}}}}'",
        STANDARD.encode(&final_tx)
    );

    Ok(())
}

pub fn broadcast_transaction(tx_base64: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "broadcast_tx_sync",
        "params": {
            "tx": tx_base64
        }
    });
    let response = client
        .post("https://rpc.testcosmos.directory/cosmosicsprovidertestnet")
        .json(&request)
        .send()?;
    let response_json: serde_json::Value = response.json()?;
    println!("\nTransaction broadcast response:");
    println!("{}", serde_json::to_string_pretty(&response_json)?);
    println!(
        "https://www.mintscan.io/ics-testnet-provider/tx/{}",
        response_json["result"]["hash"].as_str().unwrap_or("")
    );
    Ok(())
}

pub fn print_fund_command() -> Result<(), Box<dyn Error>> {
    let public_key = get_public_key_from_canister()?;
    let cosmos_address = public_key_to_cosmos_address(&public_key)?;

    println!("\nTo fund your wallet, run this command:");
    println!("gaiad tx bank send <faucet_address> {} 100000uatom --chain-id provider --node https://rpc.testcosmos.directory/cosmosicsprovidertestnet --fees 1000uatom", cosmos_address);
    println!(
        "\nThis will send 100,000 uatom from the faucet wallet to your address: {}",
        cosmos_address
    );

    Ok(())
}

pub fn get_account_info(address: &str) -> Result<(u64, u64), Box<dyn Error>> {
    let client = Client::new();

    // Create the query data - format is: 0a<length><address_string>
    // The address should be encoded as a string, not as raw bytes
    let query_data = format!("0a{:02x}{}", address.len(), hex::encode(address.as_bytes()));

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "abci_query",
        "params": {
            "path": "/cosmos.auth.v1beta1.Query/Account",
            "data": query_data,
            "height": "0",
            "prove": false
        }
    });

    let response = client
        .post("https://rpc.testcosmos.directory/cosmosicsprovidertestnet")
        .json(&request)
        .send()?;

    let response_json: serde_json::Value = response.json()?;

    // Check for error in response
    if let Some(error) = response_json.get("error") {
        return Err(format!("RPC error: {}", error).into());
    }

    // Check for error code in response
    if let Some(code) = response_json["result"]["response"]["code"].as_i64() {
        if code != 0 {
            let log = response_json["result"]["response"]["log"]
                .as_str()
                .unwrap_or("Unknown error");
            return Err(format!("Query error (code {}): {}", code, log).into());
        }
    }

    let result = response_json["result"]["response"]["value"]
        .as_str()
        .ok_or_else(|| "Failed to get account info")?;

    let decoded = STANDARD.decode(result)?;

    // First parse the QueryAccountResponse
    let query_response = QueryAccountResponse::decode(&decoded[..])?;

    // Get the account Any message
    let account_any = query_response.account.ok_or_else(|| "No account found")?;

    // Parse the BaseAccount from the account field
    let account = BaseAccount::decode(account_any.value.as_slice())?;

    // If account number is 0, it means the account doesn't exist yet
    if account.account_number == 0 {
        return Err("Account does not exist yet. Please fund it first.".into());
    }

    Ok((account.account_number, account.sequence))
}

pub fn analyze_account_response(address: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let query_data = format!("0a{:02x}{}", address.len(), hex::encode(address.as_bytes()));

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "abci_query",
        "params": {
            "path": "/cosmos.auth.v1beta1.Query/Account",
            "data": query_data,
            "height": "0",
            "prove": false
        }
    });

    let response = client
        .post("https://rpc.testcosmos.directory/cosmosicsprovidertestnet")
        .json(&request)
        .send()?;

    let response_json: serde_json::Value = response.json()?;

    let mut analysis = String::new();

    // Check for error in response
    if let Some(error) = response_json.get("error") {
        return Ok(format!("RPC Error: {}", error));
    }

    // Check for error code in response
    if let Some(code) = response_json["result"]["response"]["code"].as_i64() {
        if code != 0 {
            let log = response_json["result"]["response"]["log"]
                .as_str()
                .unwrap_or("Unknown error");
            return Ok(format!("Query Error (code {}): {}", code, log));
        }
    }

    let result = response_json["result"]["response"]["value"]
        .as_str()
        .ok_or_else(|| "Failed to get account info")?;

    analysis.push_str(&format!("Raw response value (base64): {}\n", result));

    let decoded = STANDARD.decode(result)?;
    analysis.push_str(&format!("Decoded response (hex): {}\n", hex::encode(&decoded)));
    analysis.push_str(&format!("Decoded response length: {} bytes\n\n", decoded.len()));

    // Parse the QueryAccountResponse
    let query_response = QueryAccountResponse::decode(&decoded[..])?;
    analysis.push_str("Parsed QueryAccountResponse:\n");

    if let Some(account_any) = &query_response.account {
        analysis.push_str(&format!("  Account type URL: {}\n", account_any.type_url));
        analysis.push_str(&format!("  Account value length: {} bytes\n", account_any.value.len()));
        analysis.push_str(&format!(
            "  Account value (hex): {}\n\n",
            hex::encode(&account_any.value)
        ));

        // Parse the BaseAccount
        if let Ok(account) = BaseAccount::decode(account_any.value.as_slice()) {
            analysis.push_str("Parsed BaseAccount structure:\n");
            analysis.push_str(&format!("  Address: {}\n", account.address));
            analysis.push_str(&format!("  Account Number: {}\n", account.account_number));
            analysis.push_str(&format!("  Sequence: {}\n", account.sequence));

            if let Some(pub_key) = &account.pub_key {
                analysis.push_str(&format!("  Public Key Type: {}\n", pub_key.type_url));
                analysis.push_str(&format!("  Public Key Value (hex): {}\n", hex::encode(&pub_key.value)));
                analysis.push_str(&format!("  Public Key Value length: {} bytes\n", pub_key.value.len()));

                // If it's a secp256k1 public key, decode it
                if pub_key.type_url == "/cosmos.crypto.secp256k1.PubKey" {
                    if let Ok(secp_key) = PubKey::decode(pub_key.value.as_slice()) {
                        analysis.push_str(&format!("  Secp256k1 Key (hex): {}\n", hex::encode(&secp_key.key)));
                        analysis.push_str(&format!("  Secp256k1 Key length: {} bytes\n", secp_key.key.len()));
                    }
                }
            } else {
                analysis.push_str("  Public Key: None (account not used yet)\n");
            }
        } else {
            analysis.push_str("Failed to parse BaseAccount\n");
        }
    } else {
        analysis.push_str("No account found in response\n");
    }

    Ok(analysis)
}

/// Analyze gas usage from a transaction result to improve estimates
pub fn analyze_gas_usage_from_result(tx_result_json: &str) -> Result<(), Box<dyn Error>> {
    let result: serde_json::Value = serde_json::from_str(tx_result_json)?;

    let gas_wanted = result["data"]["gas_wanted"].as_str().unwrap_or("0").parse::<u64>()?;
    let gas_used = result["data"]["gas_used"].as_str().unwrap_or("0").parse::<u64>()?;
    let code = result["data"]["code"].as_u64().unwrap_or(0);

    // Extract message type from the transaction
    let message_type = result["data"]["tx"]["body"]["messages"][0]["@type"]
        .as_str()
        .unwrap_or("unknown");

    println!("=== Gas Usage Analysis ===");
    println!("Message Type: {}", message_type);
    println!("Gas Wanted: {}", gas_wanted);
    println!("Gas Used: {}", gas_used);
    println!("Result: {}", if code == 0 { "SUCCESS" } else { "FAILED" });

    if gas_used > 0 {
        let efficiency = (gas_used as f64 / gas_wanted as f64) * 100.0;
        println!("Gas Efficiency: {:.1}% ({} / {})", efficiency, gas_used, gas_wanted);

        if code != 0 && gas_used >= gas_wanted {
            println!("⚠️  OUT OF GAS: Need at least {} gas", gas_used + 1);
            println!("💡 Recommended gas limit: {}", ((gas_used as f64) * 1.2) as u64);
        } else if efficiency < 70.0 {
            println!(
                "💰 OVER-PROVISIONED: Could reduce gas limit to {}",
                ((gas_used as f64) * 1.15) as u64
            );
        } else if efficiency > 95.0 {
            println!(
                "⚠️  CLOSE CALL: Consider increasing buffer to {}",
                ((gas_used as f64) * 1.2) as u64
            );
        } else {
            println!("✅ OPTIMAL: Gas allocation is reasonable");
        }

        // Suggest improvements to our estimates
        match message_type {
            "/cosmos.staking.v1beta1.MsgDelegate" => {
                let current_estimate = 324000; // Current estimate from our function
                let recommended = ((gas_used as f64) * 1.2) as u64;
                if recommended != current_estimate {
                    println!("📊 Code Update Suggestion:");
                    println!(
                        "   Update MsgDelegate base gas from 270,000 to {}",
                        ((gas_used as f64) * 1.0) as u64
                    );
                    println!("   This would give final estimate: {}", recommended);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_to_cosmos_address() {
        let public_key = get_public_key_from_canister().unwrap();
        let addr = public_key_to_cosmos_address(&public_key).unwrap();
        // Check prefix and length
        assert!(addr.starts_with("cosmos1"));
        assert_eq!(addr.len(), 45); // cosmos1 + 39 chars
    }
}

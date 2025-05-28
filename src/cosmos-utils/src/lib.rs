//! Core Cosmos utilities logic extracted from main.rs for testability and reuse.

use bech32::{self, ToBase32};
use ripemd::Ripemd160;
use sha2::{Sha256, Digest as Sha256Digest};
use std::error::Error;
use cosmos_sdk_proto::cosmos::{
    base::v1beta1::Coin,
    tx::v1beta1::{Tx, TxBody, AuthInfo, SignerInfo, ModeInfo, Fee},
    tx::signing::v1beta1::SignMode,
    bank::v1beta1::MsgSend,
    crypto::secp256k1::PubKey,
};
use prost_types::Any;
use prost::Message;
use hex;
use base64;
use reqwest::blocking::Client;
use serde_json::json;
use bs58;
use std::process::Command;

#[derive(Message)]
pub struct SignDoc {
    #[prost(bytes, tag="1")]
    pub body_bytes: Vec<u8>,
    #[prost(bytes, tag="2")]
    pub auth_info_bytes: Vec<u8>,
    #[prost(string, tag="3")]
    pub chain_id: String,
    #[prost(uint64, tag="4")]
    pub account_number: u64,
}

#[derive(Message)]
pub struct QueryAccountResponse {
    #[prost(message, optional, tag="1")]
    pub account: Option<Any>,
}

#[derive(Message)]
pub struct BaseAccount {
    #[prost(string, tag="1")]
    pub address: String,
    #[prost(message, optional, tag="2")]
    pub pub_key: Option<Any>,
    #[prost(uint64, tag="3")]
    pub account_number: u64,
    #[prost(uint64, tag="4")]
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

pub fn get_sign_bytes(tx_body: &TxBody, auth_info: &AuthInfo) -> Result<Vec<u8>, Box<dyn Error>> {
    let tx = Tx {
        body: Some(tx_body.clone()),
        auth_info: Some(auth_info.clone()),
        signatures: vec![],
    };
    Ok(tx.encode_to_vec())
}

pub fn create_send_transaction(from_address: &str, to_address: &str, amount: u64, signature: Option<Vec<u8>>) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
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
                    }
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
            eprintln!("Error: signature length is {} bytes, expected at least 64. Signature (hex): {}", sig.len(), hex::encode(&sig));
            return Err(format!("Signature too short: got {} bytes, expected at least 64", sig.len()).into());
        }
        // Use the raw 64-byte signature as Cosmos expects
        tx.signatures = vec![sig[..64].to_vec()];
    }
    Ok((tx.encode_to_vec(), sign_bytes))
}

pub fn get_public_key_from_canister() -> Result<String, Box<dyn Error>> {
    let output = Command::new("dfx")
        .args(["canister", "call", "solana_wallet", "address"])
        .output()?;
    
    let stdout = String::from_utf8(output.stdout)?;
    // Remove quotes and whitespace from the output
    let public_key = stdout.trim().trim_matches(|c| c == '(' || c == ')' || c == '"' || c == ' ');
    Ok(public_key.to_string())
}

pub fn get_signature_from_canister(sign_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let output = Command::new("dfx")
        .args(["canister", "call", "--update", "solana_wallet", "signMessage", &format!("(blob \"{}\")", sign_bytes.iter().map(|b| format!("\\{:02X}", b)).collect::<String>())])
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

pub fn build_transaction() -> Result<(), Box<dyn Error>> {
    let public_key = get_public_key_from_canister()?;
    let cosmos_address = public_key_to_cosmos_address(&public_key)?;
    println!("Cosmos address: {}", cosmos_address);
    let (tx_bytes, sign_bytes) = create_send_transaction(&cosmos_address, &cosmos_address, 1000, None)?;
    println!("Raw transaction (base64): {}", base64::encode(&tx_bytes));
    println!("Raw transaction (hex): 0x{}", hex::encode(&tx_bytes));
    println!("\nCanonical sign bytes (base64): {}", base64::encode(&sign_bytes));
    println!("Canonical sign bytes (hex): 0x{}", hex::encode(&sign_bytes));
    
    println!("\nGetting signature from canister...");
    let signature = get_signature_from_canister(&sign_bytes)?;
    
    println!("\nCreating signed transaction...");
    let (final_tx, _) = create_send_transaction(&cosmos_address, &cosmos_address, 1000, Some(signature))?;
    println!("\nSigned transaction (base64): {}", base64::encode(&final_tx));
    println!("Signed transaction (hex): 0x{}", hex::encode(&final_tx));
    
    println!("\nTo broadcast this transaction, run:");
    println!("cargo run -- broadcast \"{}\"", base64::encode(&final_tx));
    
    Ok(())
}

pub fn send_signed_transaction(signature_blob: &str) -> Result<(), Box<dyn Error>> {
    let public_key = get_public_key_from_canister()?;
    let cosmos_address = public_key_to_cosmos_address(&public_key)?;
    let signature = signature_blob
        .trim_matches(|c| c == '"' || c == '\\')
        .split("\\")
        .filter(|s| !s.is_empty())
        .map(|s| u8::from_str_radix(s, 16).unwrap_or(0))
        .collect::<Vec<u8>>();
    let (final_tx, _) = create_send_transaction(&cosmos_address, &cosmos_address, 1000, Some(signature))?;
    println!("{}", base64::encode(&final_tx));
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
    Ok(())
}

pub fn print_fund_command() -> Result<(), Box<dyn Error>> {
    let public_key = get_public_key_from_canister()?;
    let cosmos_address = public_key_to_cosmos_address(&public_key)?;
    
    println!("\nTo fund your wallet, run this command:");
    println!("gaiad tx bank send cosmos17xqjqfljz4aq6nurwg9r3r9l7gxtajz0hq3ewf {} 100000uatom --chain-id provider --node https://rpc.testcosmos.directory/cosmosicsprovidertestnet --fees 1000uatom", cosmos_address);
    println!("\nThis will send 100,000 uatom from the faucet wallet to your address: {}", cosmos_address);
    
    Ok(())
}

pub fn get_account_info(address: &str) -> Result<(u64, u64), Box<dyn Error>> {
    let client = Client::new();
    
    // Create the query data - format is: 0a<length><address_string>
    // The address should be encoded as a string, not as raw bytes
    let query_data = format!(
        "0a{:02x}{}",
        address.len(),
        hex::encode(address.as_bytes())
    );
    
    println!("Address: {}", address);
    println!("Query data: {}", query_data);
    
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
    
    println!("Sending request: {}", serde_json::to_string_pretty(&request)?);
    
    let response = client
        .post("https://rpc.testcosmos.directory/cosmosicsprovidertestnet")
        .json(&request)
        .send()?;
    
    let response_json: serde_json::Value = response.json()?;
    println!("Response: {}", serde_json::to_string_pretty(&response_json)?);
    
    // Check for error in response
    if let Some(error) = response_json.get("error") {
        return Err(format!("RPC error: {}", error).into());
    }
    
    // Check for error code in response
    if let Some(code) = response_json["result"]["response"]["code"].as_i64() {
        if code != 0 {
            let log = response_json["result"]["response"]["log"].as_str()
                .unwrap_or("Unknown error");
            return Err(format!("Query error (code {}): {}", code, log).into());
        }
    }
    
    let result = response_json["result"]["response"]["value"].as_str()
        .ok_or_else(|| "Failed to get account info")?;
    
    let decoded = base64::decode(result)?;
    println!("Decoded value (hex): {}", hex::encode(&decoded));
    
    // First parse the QueryAccountResponse
    let query_response = QueryAccountResponse::decode(&decoded[..])?;
    
    // Get the account Any message
    let account_any = query_response.account.ok_or_else(|| "No account found")?;
    println!("Account type URL: {}", account_any.type_url);
    println!("Account value (hex): {}", hex::encode(&account_any.value));
    
    // Parse the BaseAccount from the account field
    let account = BaseAccount::decode(account_any.value.as_slice())?;
    println!("Parsed account: address={}, account_number={}, sequence={}", 
        account.address, account.account_number, account.sequence);
    
    // If account number is 0, it means the account doesn't exist yet
    if account.account_number == 0 {
        return Err("Account does not exist yet. Please fund it first.".into());
    }
    
    Ok((account.account_number, account.sequence))
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

    #[test]
    fn test_create_send_transaction() {
        let public_key = get_public_key_from_canister().unwrap();
        let addr = public_key_to_cosmos_address(&public_key).unwrap();
        let (tx_bytes, sign_bytes) = create_send_transaction(&addr, &addr, 1000, None).unwrap();
        assert!(!tx_bytes.is_empty());
        assert!(!sign_bytes.is_empty());
    }

    #[test]
    fn test_get_sign_bytes_consistency() {
        let public_key = get_public_key_from_canister().unwrap();
        let addr = public_key_to_cosmos_address(&public_key).unwrap();
        let (tx_bytes, sign_bytes1) = create_send_transaction(&addr, &addr, 1000, None).unwrap();
        // Rebuild the tx_body and auth_info to test get_sign_bytes directly
        let msg_send = MsgSend {
            from_address: addr.clone(),
            to_address: addr.clone(),
            amount: vec![Coin {
                denom: "uatom".to_string(),
                amount: "1000".to_string(),
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
        let pub_key = PubKey {
            key: bs58::decode(&public_key).into_vec().unwrap(),
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
        let auth_info = AuthInfo {
            signer_infos: vec![SignerInfo {
                public_key: Some(pub_key_any),
                mode_info: Some(ModeInfo {
                    sum: Some(cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Sum::Single(
                        cosmos_sdk_proto::cosmos::tx::v1beta1::mode_info::Single {
                            mode: SignMode::Direct as i32,
                        }
                    )),
                }),
                sequence: 0,
            }],
            fee: Some(fee),
            tip: None,
        };
        let sign_bytes2 = get_sign_bytes(&tx_body, &auth_info).unwrap();
        assert_eq!(sign_bytes1, sign_bytes2);
    }

    #[test]
    fn test_send_signed_transaction() {
        let public_key = get_public_key_from_canister().unwrap();
        let addr = public_key_to_cosmos_address(&public_key).unwrap();
        // Use a dummy signature for testing
        let dummy_signature = vec![0u8; 64]; // 64 bytes dummy signature
        let (final_tx, _) = create_send_transaction(&addr, &addr, 1000, Some(dummy_signature)).unwrap();
        assert!(!final_tx.is_empty());
    }
} 
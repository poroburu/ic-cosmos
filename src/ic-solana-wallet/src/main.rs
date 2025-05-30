use std::str::FromStr;

use candid::candid_method;
use cosmwasm_std::{from_base64, BankMsg, Coin, CosmosMsg, Uint128};
use ic_cdk::update;
use ic_solana::{
    rpc_client::{RpcConfig, RpcResult, RpcServices},
    types::{BlockHash, Pubkey, RpcSendTransactionConfig, Transaction},
};
use ic_solana_wallet::{
    eddsa::{ecdsa_public_key, sign_with_ecdsa},
    state::{read_state, InitArgs, State},
    utils::validate_caller_not_anonymous,
};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use bech32::Hrp;
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Simple structs for account info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CosmosAccountInfo {
    pub account_number: u64,
    pub sequence: u64,
}

// Transaction structure using cosmwasm_std types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CosmosTransaction {
    pub from_address: String,
    pub to_address: String,
    pub amount: Vec<Coin>,
    pub fee: Vec<Coin>,
    pub gas_limit: u64,
    pub memo: String,
    pub chain_id: String,
    pub account_number: u64,
    pub sequence: u64,
}

/// Utility function to convert a public key to a Cosmos address
fn public_key_to_cosmos_address(public_key: &str) -> Result<String, String> {
    let decoded = bs58::decode(public_key)
        .into_vec()
        .map_err(|e| format!("Failed to decode public key: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&decoded);
    let sha256_hash = hasher.finalize();

    let mut hasher = Ripemd160::new();
    hasher.update(sha256_hash);
    let ripemd160_hash = hasher.finalize();

    let data = ripemd160_hash.to_vec();
    let hrp = Hrp::parse("cosmos").map_err(|e| format!("Failed to parse HRP: {}", e))?;
    let encoded =
        bech32::encode::<bech32::Bech32>(hrp, &data).map_err(|e| format!("Failed to encode address: {}", e))?;

    Ok(encoded)
}

/// Parse ABCI query response to extract account number and sequence
/// This is a simplified parser for the specific protobuf format we expect
fn parse_account_info_from_abci(response_value: &str) -> Result<(u64, u64), String> {
    // Decode the base64 response using cosmwasm_std
    let decoded = from_base64(response_value).map_err(|e| format!("Failed to decode base64 response: {}", e))?;

    // Parse the protobuf response manually
    // We expect: QueryAccountResponse -> account (Any) -> BaseAccount

    let mut cursor = 0;

    // Helper function to read a varint from the byte stream
    let read_varint = |data: &[u8], pos: &mut usize| -> Result<u64, String> {
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
                return Err("Varint too long".to_string());
            }
        }

        Err("Unexpected end of data while reading varint".to_string())
    };

    // Helper function to read a length-delimited field
    let read_bytes = |data: &[u8], pos: &mut usize| -> Result<Vec<u8>, String> {
        let length = read_varint(data, pos)? as usize;
        if *pos + length > data.len() {
            return Err("Length exceeds remaining data".to_string());
        }
        let result = data[*pos..*pos + length].to_vec();
        *pos += length;
        Ok(result)
    };

    // Parse QueryAccountResponse
    while cursor < decoded.len() {
        if cursor >= decoded.len() {
            break;
        }

        let tag = decoded[cursor];
        cursor += 1;

        let field_number = tag >> 3;
        let wire_type = tag & 0x07;

        if field_number == 1 && wire_type == 2 {
            // account field (Any type)
            // Read the Any message bytes
            let any_bytes = read_bytes(&decoded, &mut cursor)?;

            // Parse the Any message to find the type_url and value
            let mut any_cursor = 0;
            let mut base_account_bytes: Option<Vec<u8>> = None;

            while any_cursor < any_bytes.len() {
                let any_tag = any_bytes[any_cursor];
                any_cursor += 1;

                let any_field = any_tag >> 3;
                let any_wire = any_tag & 0x07;

                if any_field == 1 && any_wire == 2 {
                    // type_url
                    let type_url_bytes = read_bytes(&any_bytes, &mut any_cursor)?;
                    let type_url = String::from_utf8_lossy(&type_url_bytes);

                    if !type_url.ends_with("BaseAccount") {
                        return Err(format!("Unexpected account type: {}", type_url));
                    }
                } else if any_field == 2 && any_wire == 2 {
                    // value
                    base_account_bytes = Some(read_bytes(&any_bytes, &mut any_cursor)?);
                } else {
                    // Skip unknown field
                    if any_wire == 2 {
                        read_bytes(&any_bytes, &mut any_cursor)?;
                    } else if any_wire == 0 {
                        read_varint(&any_bytes, &mut any_cursor)?;
                    } else {
                        return Err("Unsupported wire type in Any message".to_string());
                    }
                }
            }

            // Parse BaseAccount
            if let Some(base_account_data) = base_account_bytes {
                let mut ba_cursor = 0;
                let mut account_number = 0u64;
                let mut sequence = 0u64;

                while ba_cursor < base_account_data.len() {
                    let ba_tag = base_account_data[ba_cursor];
                    ba_cursor += 1;

                    let ba_field = ba_tag >> 3;
                    let ba_wire = ba_tag & 0x07;

                    match ba_field {
                        1 => {
                            // address (string)
                            if ba_wire == 2 {
                                read_bytes(&base_account_data, &mut ba_cursor)?;
                            }
                        }
                        2 => {
                            // pub_key (Any)
                            if ba_wire == 2 {
                                read_bytes(&base_account_data, &mut ba_cursor)?;
                            }
                        }
                        3 => {
                            // account_number (uint64)
                            if ba_wire == 0 {
                                account_number = read_varint(&base_account_data, &mut ba_cursor)?;
                            }
                        }
                        4 => {
                            // sequence (uint64)
                            if ba_wire == 0 {
                                sequence = read_varint(&base_account_data, &mut ba_cursor)?;
                            }
                        }
                        _ => {
                            // Skip unknown field
                            if ba_wire == 2 {
                                read_bytes(&base_account_data, &mut ba_cursor)?;
                            } else if ba_wire == 0 {
                                read_varint(&base_account_data, &mut ba_cursor)?;
                            }
                        }
                    }
                }

                if account_number == 0 {
                    return Err("Account does not exist (account_number is 0)".to_string());
                }

                return Ok((account_number, sequence));
            }
        } else {
            // Skip unknown field
            if wire_type == 2 {
                read_bytes(&decoded, &mut cursor)?;
            } else if wire_type == 0 {
                read_varint(&decoded, &mut cursor)?;
            } else {
                return Err("Unsupported wire type in QueryAccountResponse".to_string());
            }
        }
    }

    Err("No valid account found in response".to_string())
}

/// Create sign document bytes for Cosmos transaction signing using manual protobuf encoding
fn create_sign_doc_bytes(transaction: &CosmosTransaction, public_key: &[u8]) -> Result<Vec<u8>, String> {
    // Helper function to encode varint
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

    // Helper function to encode length-delimited field
    fn encode_length_delimited(tag: u8, data: &[u8]) -> Vec<u8> {
        let mut result = vec![tag];
        result.extend(encode_varint(data.len() as u64));
        result.extend(data);
        result
    }

    // Helper function to encode string field
    fn encode_string(tag: u8, value: &str) -> Vec<u8> {
        encode_length_delimited(tag, value.as_bytes())
    }

    // Helper function to encode uint64 field
    fn encode_uint64(tag: u8, value: u64) -> Vec<u8> {
        let mut result = vec![tag];
        result.extend(encode_varint(value));
        result
    }

    // Create MsgSend protobuf bytes
    let mut msg_send_bytes = Vec::new();
    msg_send_bytes.extend(encode_string(0x0a, &transaction.from_address)); // from_address = 1
    msg_send_bytes.extend(encode_string(0x12, &transaction.to_address)); // to_address = 2
    
    // Encode amount array (field 3)
    for coin in &transaction.amount {
        let mut coin_bytes = Vec::new();
        coin_bytes.extend(encode_string(0x0a, &coin.denom)); // denom = 1
        coin_bytes.extend(encode_string(0x12, &coin.amount.to_string())); // amount = 2
        msg_send_bytes.extend(encode_length_delimited(0x1a, &coin_bytes)); // amount = 3
    }

    // Create Any message for MsgSend
    let type_url = "/cosmos.bank.v1beta1.MsgSend";
    let mut msg_any_bytes = Vec::new();
    msg_any_bytes.extend(encode_string(0x0a, type_url)); // type_url = 1
    msg_any_bytes.extend(encode_length_delimited(0x12, &msg_send_bytes)); // value = 2

    // Create TxBody
    let mut tx_body_bytes = Vec::new();
    tx_body_bytes.extend(encode_length_delimited(0x0a, &msg_any_bytes)); // messages = 1
    tx_body_bytes.extend(encode_string(0x12, &transaction.memo)); // memo = 2
    tx_body_bytes.extend(encode_uint64(0x18, 0)); // timeout_height = 3

    // Create PubKey
    let mut pub_key_bytes = Vec::new();
    pub_key_bytes.extend(encode_length_delimited(0x0a, public_key)); // key = 1

    // Create Any message for PubKey
    let pub_key_type_url = "/cosmos.crypto.secp256k1.PubKey";
    let mut pub_key_any_bytes = Vec::new();
    pub_key_any_bytes.extend(encode_string(0x0a, pub_key_type_url)); // type_url = 1
    pub_key_any_bytes.extend(encode_length_delimited(0x12, &pub_key_bytes)); // value = 2

    // Create Fee
    let mut fee_bytes = Vec::new();
    // Encode fee amount array (field 1)
    for coin in &transaction.fee {
        let mut coin_bytes = Vec::new();
        coin_bytes.extend(encode_string(0x0a, &coin.denom)); // denom = 1
        coin_bytes.extend(encode_string(0x12, &coin.amount.to_string())); // amount = 2
        fee_bytes.extend(encode_length_delimited(0x0a, &coin_bytes)); // amount = 1
    }
    fee_bytes.extend(encode_uint64(0x10, transaction.gas_limit)); // gas_limit = 2
    fee_bytes.extend(encode_string(0x1a, "")); // payer = 3
    fee_bytes.extend(encode_string(0x22, "")); // granter = 4

    // Create ModeInfo Single
    let mut mode_info_single_bytes = Vec::new();
    mode_info_single_bytes.extend(encode_uint64(0x08, 1)); // mode = SIGN_MODE_DIRECT = 1

    // Create ModeInfo
    let mut mode_info_bytes = Vec::new();
    mode_info_bytes.extend(encode_length_delimited(0x0a, &mode_info_single_bytes)); // single = 1

    // Create SignerInfo
    let mut signer_info_bytes = Vec::new();
    signer_info_bytes.extend(encode_length_delimited(0x0a, &pub_key_any_bytes)); // public_key = 1
    signer_info_bytes.extend(encode_length_delimited(0x12, &mode_info_bytes)); // mode_info = 2
    signer_info_bytes.extend(encode_uint64(0x18, transaction.sequence)); // sequence = 3

    // Create AuthInfo
    let mut auth_info_bytes = Vec::new();
    auth_info_bytes.extend(encode_length_delimited(0x0a, &signer_info_bytes)); // signer_infos = 1
    auth_info_bytes.extend(encode_length_delimited(0x12, &fee_bytes)); // fee = 2

    // Create SignDoc
    let mut sign_doc_bytes = Vec::new();
    sign_doc_bytes.extend(encode_length_delimited(0x0a, &tx_body_bytes)); // body_bytes = 1
    sign_doc_bytes.extend(encode_length_delimited(0x12, &auth_info_bytes)); // auth_info_bytes = 2
    sign_doc_bytes.extend(encode_string(0x1a, &transaction.chain_id)); // chain_id = 3
    sign_doc_bytes.extend(encode_uint64(0x20, transaction.account_number)); // account_number = 4

    Ok(sign_doc_bytes)
}

/// Build final transaction for broadcasting using manual protobuf encoding
fn build_transaction_for_broadcast(
    transaction: &CosmosTransaction,
    public_key: &[u8],
    signature: &[u8],
) -> Result<String, String> {
    // Helper function to encode varint
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

    // Helper function to encode length-delimited field
    fn encode_length_delimited(tag: u8, data: &[u8]) -> Vec<u8> {
        let mut result = vec![tag];
        result.extend(encode_varint(data.len() as u64));
        result.extend(data);
        result
    }

    // Helper function to encode string field
    fn encode_string(tag: u8, value: &str) -> Vec<u8> {
        encode_length_delimited(tag, value.as_bytes())
    }

    // Helper function to encode uint64 field
    fn encode_uint64(tag: u8, value: u64) -> Vec<u8> {
        let mut result = vec![tag];
        result.extend(encode_varint(value));
        result
    }

    // Create MsgSend protobuf bytes
    let mut msg_send_bytes = Vec::new();
    msg_send_bytes.extend(encode_string(0x0a, &transaction.from_address)); // from_address = 1
    msg_send_bytes.extend(encode_string(0x12, &transaction.to_address)); // to_address = 2
    
    // Encode amount array (field 3)
    for coin in &transaction.amount {
        let mut coin_bytes = Vec::new();
        coin_bytes.extend(encode_string(0x0a, &coin.denom)); // denom = 1
        coin_bytes.extend(encode_string(0x12, &coin.amount.to_string())); // amount = 2
        msg_send_bytes.extend(encode_length_delimited(0x1a, &coin_bytes)); // amount = 3
    }

    // Create Any message for MsgSend
    let type_url = "/cosmos.bank.v1beta1.MsgSend";
    let mut msg_any_bytes = Vec::new();
    msg_any_bytes.extend(encode_string(0x0a, type_url)); // type_url = 1
    msg_any_bytes.extend(encode_length_delimited(0x12, &msg_send_bytes)); // value = 2

    // Create TxBody
    let mut tx_body_bytes = Vec::new();
    tx_body_bytes.extend(encode_length_delimited(0x0a, &msg_any_bytes)); // messages = 1
    tx_body_bytes.extend(encode_string(0x12, &transaction.memo)); // memo = 2
    tx_body_bytes.extend(encode_uint64(0x18, 0)); // timeout_height = 3

    // Create PubKey
    let mut pub_key_bytes = Vec::new();
    pub_key_bytes.extend(encode_length_delimited(0x0a, public_key)); // key = 1

    // Create Any message for PubKey
    let pub_key_type_url = "/cosmos.crypto.secp256k1.PubKey";
    let mut pub_key_any_bytes = Vec::new();
    pub_key_any_bytes.extend(encode_string(0x0a, pub_key_type_url)); // type_url = 1
    pub_key_any_bytes.extend(encode_length_delimited(0x12, &pub_key_bytes)); // value = 2

    // Create Fee
    let mut fee_bytes = Vec::new();
    // Encode fee amount array (field 1)
    for coin in &transaction.fee {
        let mut coin_bytes = Vec::new();
        coin_bytes.extend(encode_string(0x0a, &coin.denom)); // denom = 1
        coin_bytes.extend(encode_string(0x12, &coin.amount.to_string())); // amount = 2
        fee_bytes.extend(encode_length_delimited(0x0a, &coin_bytes)); // amount = 1
    }
    fee_bytes.extend(encode_uint64(0x10, transaction.gas_limit)); // gas_limit = 2
    fee_bytes.extend(encode_string(0x1a, "")); // payer = 3
    fee_bytes.extend(encode_string(0x22, "")); // granter = 4

    // Create ModeInfo Single
    let mut mode_info_single_bytes = Vec::new();
    mode_info_single_bytes.extend(encode_uint64(0x08, 1)); // mode = SIGN_MODE_DIRECT = 1

    // Create ModeInfo
    let mut mode_info_bytes = Vec::new();
    mode_info_bytes.extend(encode_length_delimited(0x0a, &mode_info_single_bytes)); // single = 1

    // Create SignerInfo
    let mut signer_info_bytes = Vec::new();
    signer_info_bytes.extend(encode_length_delimited(0x0a, &pub_key_any_bytes)); // public_key = 1
    signer_info_bytes.extend(encode_length_delimited(0x12, &mode_info_bytes)); // mode_info = 2
    signer_info_bytes.extend(encode_uint64(0x18, transaction.sequence)); // sequence = 3

    // Create AuthInfo
    let mut auth_info_bytes = Vec::new();
    auth_info_bytes.extend(encode_length_delimited(0x0a, &signer_info_bytes)); // signer_infos = 1
    auth_info_bytes.extend(encode_length_delimited(0x12, &fee_bytes)); // fee = 2

    // Create final Tx
    let mut tx_bytes = Vec::new();
    tx_bytes.extend(encode_length_delimited(0x0a, &tx_body_bytes)); // body = 1
    tx_bytes.extend(encode_length_delimited(0x12, &auth_info_bytes)); // auth_info = 2
    tx_bytes.extend(encode_length_delimited(0x1a, signature)); // signatures = 3

    // Encode as base64
    Ok(STANDARD.encode(&tx_bytes))
}

/// Returns the public key of the Solana wallet associated with the caller.
///
/// # Returns
///
/// - `String`: The Solana public key as a string.
#[update]
#[candid_method]
pub async fn address() -> String {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![caller.as_slice().to_vec()];
    let pk = ecdsa_public_key(key_name, derived_path).await;

    // For secp256k1, compressed public key is 33 bytes:
    // - First byte is 0x02 or 0x03 (compression prefix)
    // - Followed by 32 bytes of the x-coordinate
    if pk.len() != 33 {
        panic!("Expected 33-byte compressed public key");
    }

    //let x_coordinate = &pk[1..];
    Pubkey::try_from(pk).expect("Invalid public key").to_string()
}

/// Signs a provided message using the caller's Eddsa key.
///
/// # Parameters
///
/// - `message` (`String`): The message to be signed.
///
/// # Returns
///
/// - `RpcResult<String>`: The signature as a base58 encoded string on success, or an `RpcError` on
///   failure.
#[update(name = "signMessage")]
#[candid_method(update, rename = "signMessage")]
pub async fn sign_message(message: Vec<u8>) -> Vec<u8> {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![caller.as_slice().to_vec()];
    sign_with_ecdsa(key_name, derived_path, message).await
}

/// Signs and sends a transaction to the Solana network.
///
/// # Parameters
///
/// - `provider` (`String`): The Solana RPC provider ID.
/// - `raw_transaction` (`String`): The serialized unsigned transaction.
/// - `config` (`Option<RpcSendTransactionConfig>`): Optional configuration for sending the
///   transaction.
///
/// # Returns
///
/// - `RpcResult<String>`: The transaction signature as a string on success, or an `RpcError` on
///   failure.
#[update(name = "sendTransaction")]
#[candid_method(update, rename = "sendTransaction")]
pub async fn send_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    raw_transaction: String,
    params: Option<RpcSendTransactionConfig>,
) -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let sol_canister = read_state(|s| s.cos_canister);

    let mut tx = Transaction::from_str(&raw_transaction).expect("Invalid transaction");

    // Fetch the recent blockhash if it's not set
    if tx.message.recent_blockhash == BlockHash::default() {
        let response =
            ic_cdk::call::<_, (RpcResult<String>,)>(sol_canister, "sol_getLatestBlockhash", (&source,)).await?;
        tx.message.recent_blockhash = BlockHash::from_str(&response.0?).expect("Invalid recent blockhash");
    }

    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![caller.as_slice().to_vec()];

    let signature = sign_with_ecdsa(key_name, derived_path, tx.message_data())
        .await
        .try_into()
        .expect("Invalid signature");

    tx.add_signature(0, signature);

    let response = ic_cdk::call::<_, (RpcResult<String>,)>(
        sol_canister,
        "sol_sendTransaction",
        (&source, config, tx.to_string(), params),
    )
    .await?;

    response.0
}

/// Returns the Cosmos address derived from the caller's public key.
///
/// # Returns
///
/// - `RpcResult<String>`: The Cosmos address as a string.
#[update(name = "cosmosAddress")]
#[candid_method(update, rename = "cosmosAddress")]
pub async fn cosmos_address() -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![caller.as_slice().to_vec()];
    let pk = ecdsa_public_key(key_name, derived_path).await;

    public_key_to_cosmos_address(&bs58::encode(&pk).into_string())
        .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Failed to derive Cosmos address: {}", e)))
}

/// Signs and sends a Cosmos transaction using cosmwasm_std types.
///
/// # Parameters
///
/// - `source` (`RpcServices`): The Cosmos RPC provider ID.
/// - `config` (`Option<RpcConfig>`): Optional configuration for the RPC call.
/// - `raw_transaction` (`String`): The serialized unsigned Cosmos transaction in JSON format.
/// - `chain_id` (`String`): The chain ID for the Cosmos network.
///
/// # Returns
///
/// - `RpcResult<String>`: The transaction broadcast result on success, or an `RpcError` on failure.
#[update(name = "sendCosmosTransaction")]
#[candid_method(query, rename = "sendCosmosTransaction")]
pub async fn send_cosmos_transaction(
    source: RpcServices,
    config: Option<RpcConfig>,
    chain_id: String,
    raw_transaction: String,
) -> RpcResult<String> {
    let caller = validate_caller_not_anonymous();
    let cos_canister = read_state(|s| s.cos_canister);

    // Parse the raw JSON transaction
    let tx_json: serde_json::Value = serde_json::from_str(&raw_transaction)
        .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Failed to parse transaction: {}", e)))?;

    // Extract the from_address from the first message
    let from_address = tx_json["body"]["messages"][0]["from_address"].as_str().ok_or_else(|| {
        ic_solana::rpc_client::RpcError::ParseError("Missing from_address in transaction".to_string())
    })?;

    // Get our public key and derive the Cosmos address
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![caller.as_slice().to_vec()];
    let pk = ecdsa_public_key(key_name.clone(), derived_path.clone()).await;

    let our_cosmos_address = public_key_to_cosmos_address(&bs58::encode(&pk).into_string())
        .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(e))?;

    // Verify that we own the from_address
    if from_address != our_cosmos_address {
        return Err(ic_solana::rpc_client::RpcError::ParseError(
            "Transaction from_address does not match our wallet address".to_string(),
        ));
    }

    // Get account info (account_number and sequence) via abci_query
    let query_data = format!("0a{:02x}{}", from_address.len(), hex::encode(from_address.as_bytes()));
    let account_info_result = ic_cdk::call::<_, (RpcResult<ic_solana::types::ABCIQueryResult>,)>(
        cos_canister,
        "sol_getAbciQuery",
        (
            &source,
            config.clone(),
            "/cosmos.auth.v1beta1.Query/Account".to_string(),
            query_data,
            "0".to_string(),
            false,
        ),
    )
    .await
    .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Failed to call abci_query: {:?}", e)))?;

    let abci_result = account_info_result.0?;

    // Parse the ABCI response to get account info
    let (account_number, sequence) = if abci_result.response.code == 0 {
        // Success case - check if we have a value
        if abci_result.response.value.is_empty() {
            return Err(ic_solana::rpc_client::RpcError::ParseError(
                "Empty response value from ABCI query".to_string(),
            ));
        }

        // Parse the protobuf response
        parse_account_info_from_abci(&abci_result.response.value)
            .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Failed to parse account info: {}", e)))?
    } else {
        // Error case
        let log_msg = if abci_result.response.log.is_empty() {
            "Unknown error".to_string()
        } else {
            abci_result.response.log.clone()
        };

        return Err(ic_solana::rpc_client::RpcError::ParseError(format!(
            "ABCI query failed with code {}: {}",
            abci_result.response.code, log_msg
        )));
    };

    // Extract transaction details from JSON and convert to cosmwasm_std types
    let to_address = tx_json["body"]["messages"][0]["to_address"]
        .as_str()
        .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing to_address".to_string()))?;

    // Parse amounts using cosmwasm_std::Coin
    let amount_array = tx_json["body"]["messages"][0]["amount"]
        .as_array()
        .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing amount array".to_string()))?;

    let mut amounts = Vec::new();
    for amt in amount_array {
        let denom = amt["denom"]
            .as_str()
            .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing denom".to_string()))?;
        let amount_str = amt["amount"]
            .as_str()
            .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing amount".to_string()))?;
        let amount_uint = Uint128::from_str(amount_str)
            .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Invalid amount: {}", e)))?;

        amounts.push(Coin {
            denom: denom.to_string(),
            amount: amount_uint,
        });
    }

    // Parse fee using cosmwasm_std::Coin
    let fee_array = tx_json["auth_info"]["fee"]["amount"]
        .as_array()
        .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing fee amount".to_string()))?;

    let mut fees = Vec::new();
    for fee_coin in fee_array {
        let denom = fee_coin["denom"]
            .as_str()
            .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing fee denom".to_string()))?;
        let amount_str = fee_coin["amount"]
            .as_str()
            .ok_or_else(|| ic_solana::rpc_client::RpcError::ParseError("Missing fee amount".to_string()))?;
        let amount_uint = Uint128::from_str(amount_str)
            .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Invalid fee amount: {}", e)))?;

        fees.push(Coin {
            denom: denom.to_string(),
            amount: amount_uint,
        });
    }

    let gas_limit = tx_json["auth_info"]["fee"]["gas_limit"]
        .as_str()
        .unwrap_or("200000")
        .parse::<u64>()
        .unwrap_or(200000);

    let memo = tx_json["body"]["memo"].as_str().unwrap_or("");

    // Create transaction structure
    let transaction = CosmosTransaction {
        from_address: from_address.to_string(),
        to_address: to_address.to_string(),
        amount: amounts,
        fee: fees,
        gas_limit,
        memo: memo.to_string(),
        chain_id: chain_id.clone(),
        account_number,
        sequence,
    };

    // Create sign doc for signing
    let sign_bytes =
        create_sign_doc_bytes(&transaction, &pk).map_err(|e| ic_solana::rpc_client::RpcError::ParseError(e))?;

    // Sign the transaction
    let signature = sign_with_ecdsa(key_name, derived_path, sign_bytes).await;

    // Ensure signature is 64 bytes (truncate if longer)
    let signature = if signature.len() >= 64 {
        signature[..64].to_vec()
    } else {
        return Err(ic_solana::rpc_client::RpcError::ParseError(format!(
            "Signature too short: got {} bytes, expected at least 64",
            signature.len()
        )));
    };

    // Build final transaction for broadcast
    let tx_base64 = build_transaction_for_broadcast(&transaction, &pk, &signature)
        .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(e))?;

    // Broadcast the transaction
    let broadcast_result = ic_cdk::call::<_, (RpcResult<ic_solana::types::BroadcastTxResult>,)>(
        cos_canister,
        "sol_getBroadcastTxSync",
        (&source, config, tx_base64),
    )
    .await
    .map_err(|e| ic_solana::rpc_client::RpcError::ParseError(format!("Failed to broadcast transaction: {:?}", e)))?;

    let result = broadcast_result.0?;
    Ok(result.hash)
}

#[ic_cdk::init]
fn init(args: InitArgs) {
    State::init(args)
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    State::pre_upgrade()
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: Option<InitArgs>) {
    State::post_upgrade(args)
}

fn main() {}

ic_cdk::export_candid!();

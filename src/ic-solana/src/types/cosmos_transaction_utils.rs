use base64::{engine::general_purpose::STANDARD, Engine as _};
use bech32::{encode, Variant, ToBase32};
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Simple structs for account info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CosmosAccountInfo {
    pub account_number: u64,
    pub sequence: u64,
}

/// Transaction structure for Cosmos transactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CosmosTransaction {
    pub from_address: String,
    pub to_address: String,
    pub amount: Vec<CosmosCoin>,
    pub fee: Vec<CosmosCoin>,
    pub gas_limit: u64,
    pub memo: String,
    pub chain_id: String,
    pub account_number: u64,
    pub sequence: u64,
}

/// Coin structure for Cosmos amounts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CosmosCoin {
    pub denom: String,
    pub amount: String,
}

impl CosmosCoin {
    pub fn new(denom: impl Into<String>, amount: impl Into<String>) -> Self {
        Self {
            denom: denom.into(),
            amount: amount.into(),
        }
    }
}

/// Utility function to convert a public key to a Cosmos address
pub fn public_key_to_cosmos_address(public_key: &str) -> Result<String, String> {
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
    let encoded = encode("cosmos", data.to_base32(), Variant::Bech32)
        .map_err(|e| format!("Failed to encode address: {}", e))?;

    Ok(encoded)
}

/// Parse ABCI query response to extract account number and sequence
/// This is a simplified parser for the specific protobuf format we expect
pub fn parse_account_info_from_abci(response_value: &str) -> Result<(u64, u64), String> {
    // Decode the base64 response
    let decoded = STANDARD.decode(response_value)
        .map_err(|e| format!("Failed to decode base64 response: {}", e))?;

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

/// Helper function to encode string field
fn encode_string(tag: u8, value: &str) -> Vec<u8> {
    encode_length_delimited(tag, value.as_bytes())
}

/// Helper function to encode uint64 field
fn encode_uint64(tag: u8, value: u64) -> Vec<u8> {
    let mut result = vec![tag];
    result.extend(encode_varint(value));
    result
}

/// Create sign document bytes for Cosmos transaction signing using manual protobuf encoding
pub fn create_sign_doc_bytes(transaction: &CosmosTransaction, public_key: &[u8]) -> Result<Vec<u8>, String> {
    // Create MsgSend protobuf bytes
    let mut msg_send_bytes = Vec::new();
    msg_send_bytes.extend(encode_string(0x0a, &transaction.from_address)); // from_address = 1
    msg_send_bytes.extend(encode_string(0x12, &transaction.to_address)); // to_address = 2
    
    // Encode amount array (field 3)
    for coin in &transaction.amount {
        let mut coin_bytes = Vec::new();
        coin_bytes.extend(encode_string(0x0a, &coin.denom)); // denom = 1
        coin_bytes.extend(encode_string(0x12, &coin.amount)); // amount = 2
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
        coin_bytes.extend(encode_string(0x12, &coin.amount)); // amount = 2
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
pub fn build_transaction_for_broadcast(
    transaction: &CosmosTransaction,
    public_key: &[u8],
    signature: &[u8],
) -> Result<String, String> {
    // Create MsgSend protobuf bytes
    let mut msg_send_bytes = Vec::new();
    msg_send_bytes.extend(encode_string(0x0a, &transaction.from_address)); // from_address = 1
    msg_send_bytes.extend(encode_string(0x12, &transaction.to_address)); // to_address = 2
    
    // Encode amount array (field 3)
    for coin in &transaction.amount {
        let mut coin_bytes = Vec::new();
        coin_bytes.extend(encode_string(0x0a, &coin.denom)); // denom = 1
        coin_bytes.extend(encode_string(0x12, &coin.amount)); // amount = 2
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
        coin_bytes.extend(encode_string(0x12, &coin.amount)); // amount = 2
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
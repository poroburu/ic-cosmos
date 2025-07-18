
use candid::candid_method;
use ic_cdk::update;
use ic_cosmos::{
    rpc_client::{RpcConfig, RpcResult, RpcServices},
    types::{
        build_transaction_for_broadcast, create_sign_doc_bytes, extract_signer_address_from_message,
        parse_account_info_from_abci, public_key_to_cosmos_address, CosmosCoin, CosmosMessage,
        CosmosTransaction, Pubkey,
    },
};
use ic_cosmos_wallet::{
    eddsa::{ecdsa_public_key, sign_with_ecdsa},
    state::{read_state, InitArgs, State},
    utils::validate_caller_not_anonymous,
};

/// Returns the public key of the Cosmos wallet associated with the caller.
///
/// # Returns
///
/// - `String`: The Cosmos public key as a string.
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
        .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(format!("Failed to derive Cosmos address: {}", e)))
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
        .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(format!("Failed to parse transaction: {}", e)))?;

    // Parse messages from the transaction
    let messages_array = tx_json["body"]["messages"].as_array().ok_or_else(|| {
        ic_cosmos::rpc_client::RpcError::ParseError("Missing messages array in transaction body".to_string())
    })?;

    if messages_array.is_empty() {
        return Err(ic_cosmos::rpc_client::RpcError::ParseError(
            "Transaction must contain at least one message".to_string(),
        ));
    }

    // Convert JSON messages to CosmosMessage structs
    let mut cosmos_messages = Vec::new();
    for msg_json in messages_array {
        let type_url = msg_json["@type"]
            .as_str()
            .ok_or_else(|| ic_cosmos::rpc_client::RpcError::ParseError("Missing @type field in message".to_string()))?;

        // Clone the message value without the @type field
        let mut msg_value = msg_json.clone();
        if let Some(obj) = msg_value.as_object_mut() {
            obj.remove("@type");
        }

        cosmos_messages.push(CosmosMessage {
            type_url: type_url.to_string(),
            value: msg_value,
        });
    }

    // Get our public key and derive the Cosmos address
    let key_name = read_state(|s| s.ecdsa_key.to_owned());
    let derived_path = vec![caller.as_slice().to_vec()];
    let pk = ecdsa_public_key(key_name.clone(), derived_path.clone()).await;

    let our_cosmos_address = public_key_to_cosmos_address(&bs58::encode(&pk).into_string())
        .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(e))?;

    // Verify that we own all the signer addresses in the messages
    for message in &cosmos_messages {
        let signer_address =
            extract_signer_address_from_message(message).map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(e))?;

        if signer_address != our_cosmos_address {
            return Err(ic_cosmos::rpc_client::RpcError::ParseError(format!(
                "Message signer address '{}' does not match our wallet address '{}'",
                signer_address, our_cosmos_address
            )));
        }
    }

    // Get account info (account_number and sequence) via abci_query
    let query_data = format!(
        "0a{:02x}{}",
        our_cosmos_address.len(),
        hex::encode(our_cosmos_address.as_bytes())
    );
    let account_info_result = ic_cdk::call::<_, (RpcResult<ic_cosmos::types::ABCIQueryResult>,)>(
        cos_canister,
        "cos_getAbciQuery",
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
    .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(format!("Failed to call abci_query: {:?}", e)))?;

    let abci_result = account_info_result.0?;

    // Parse the ABCI response to get account info
    let (account_number, sequence) = if abci_result.response.code == 0 {
        // Success case - check if we have a value
        if abci_result.response.value.is_empty() {
            return Err(ic_cosmos::rpc_client::RpcError::ParseError(
                "Empty response value from ABCI query".to_string(),
            ));
        }

        // Parse the protobuf response
        parse_account_info_from_abci(&abci_result.response.value)
            .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(format!("Failed to parse account info: {}", e)))?
    } else {
        // Error case
        let log_msg = if abci_result.response.log.is_empty() {
            "Unknown error".to_string()
        } else {
            abci_result.response.log.clone()
        };

        return Err(ic_cosmos::rpc_client::RpcError::ParseError(format!(
            "ABCI query failed with code {}: {}",
            abci_result.response.code, log_msg
        )));
    };

    // Parse fee and convert to CosmosCoin
    let fee_array = tx_json["auth_info"]["fee"]["amount"]
        .as_array()
        .ok_or_else(|| ic_cosmos::rpc_client::RpcError::ParseError("Missing fee amount".to_string()))?;

    let mut fees = Vec::new();
    for fee_coin in fee_array {
        let denom = fee_coin["denom"]
            .as_str()
            .ok_or_else(|| ic_cosmos::rpc_client::RpcError::ParseError("Missing fee denom".to_string()))?;
        let amount_str = fee_coin["amount"]
            .as_str()
            .ok_or_else(|| ic_cosmos::rpc_client::RpcError::ParseError("Missing fee amount".to_string()))?;

        fees.push(CosmosCoin::new(denom, amount_str));
    }

    let gas_limit = tx_json["auth_info"]["fee"]["gas_limit"]
        .as_str()
        .unwrap_or("200000")
        .parse::<u64>()
        .unwrap_or(200000);

    let memo = tx_json["body"]["memo"].as_str().unwrap_or("");

    // Create transaction structure
    let transaction = CosmosTransaction {
        messages: cosmos_messages,
        fee: fees,
        gas_limit,
        memo: memo.to_string(),
        chain_id: chain_id.clone(),
        account_number,
        sequence,
    };

    // Create sign doc for signing
    let sign_bytes =
        create_sign_doc_bytes(&transaction, &pk).map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(e))?;

    // Sign the transaction
    let signature = sign_with_ecdsa(key_name, derived_path, sign_bytes).await;

    // Ensure signature is 64 bytes (truncate if longer)
    let signature = if signature.len() >= 64 {
        signature[..64].to_vec()
    } else {
        return Err(ic_cosmos::rpc_client::RpcError::ParseError(format!(
            "Signature too short: got {} bytes, expected at least 64",
            signature.len()
        )));
    };

    // Build final transaction for broadcast
    let tx_base64 = build_transaction_for_broadcast(&transaction, &pk, &signature)
        .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(e))?;

    // Broadcast the transaction
    let broadcast_result = ic_cdk::call::<_, (RpcResult<ic_cosmos::types::BroadcastTxResult>,)>(
        cos_canister,
        "cos_getBroadcastTxSync",
        (&source, config, tx_base64),
    )
    .await
    .map_err(|e| ic_cosmos::rpc_client::RpcError::ParseError(format!("Failed to broadcast transaction: {:?}", e)))?;

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

use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use cosmrs::bank::MsgSend;
use cosmrs::crypto::secp256k1::VerifyingKey;
use cosmrs::crypto::PublicKey;
use cosmrs::tendermint::{block, chain};
use cosmrs::tx::{self, Msg, Raw, SignDoc};
use cosmrs::Coin;
use ic_cosmos::{rpc_client::RpcServices, types::Pubkey};
use test_utils::MockOutcallBuilder;

mod setup;

use crate::setup::CosmosWalletSetup;

#[test]
fn test_address() {
    let setup = CosmosWalletSetup::new();

    let addr = setup.call_update::<_, String>("address", ()).wait();
    assert_eq!(addr, "2BRR6P64ikaqCyJA1oLFW2a6FrBmHwjF8RHZeEQioqKw9");
    let public_key = get_pubkey(addr);
    println!("addr: {:?}", public_key.account_id("neutron"));
    let addr = setup.as_controller().call_update::<_, String>("address", ()).wait();
    assert_eq!(addr, "tpjAAg3YphZd7zytYuCbFojxC5RDTFTrbSJTj1iM2zNZ");
}

#[test]
fn test_sign_message() {
    let setup = CosmosWalletSetup::new();
    let message: Vec<u8> = "test123".as_bytes().to_vec();
    let address = setup.call_update::<_, String>("address", ()).wait();
    println!("address: {}", address);
    let pubkey = Pubkey::from_str(&address).unwrap();
    let signature = setup
        .call_update::<_, Vec<u8>>("signMessage", (message.clone(),))
        .wait();

    let is_valid = pubkey.verify_signature(&message, &signature);
    assert!(is_valid)
}
#[test]
fn test_sign_cosmos_message() {
    let setup = CosmosWalletSetup::new();

    let address = setup.call_update::<_, String>("address", ()).wait();
    println!("address: {}", address);
    let message = get_unsigned_tx_bytes(address.clone()).unwrap();
    let pubkey = Pubkey::from_str(&address).unwrap();

    // Now we can pass the bytes directly without encoding
    let signature = setup
        .call_update::<_, Vec<u8>>("signMessage", (message.clone(),))
        .wait();

    // Verify directly with the original message bytes
    let is_valid = pubkey.verify_signature(&message, &signature);
    assert!(is_valid);

    println!("signature: {:?}", signature);
    println!("message(base64): {:?}", STANDARD.encode(&message));
    println!("signature(base64): {:?}", STANDARD.encode(&signature));
    let signed_tx = get_signed_tx_bytes(address, signature).unwrap();
    println!("signed_tx(base64): {:?}", STANDARD.encode(&signed_tx));
}
// common transaction data
fn sign_doc_data(address: String) -> SignDoc {
    let public_key = get_pubkey(address);
    let address = public_key.account_id("neutron");

    // Create message
    let msg_send = MsgSend {
        from_address: address.unwrap(),
        to_address: "neutron1wqs3zz2gnkhksd5sma6uca3rxs9tdtx8rchxsw".parse().unwrap(),
        amount: vec![Coin {
            denom: "untrn".parse().unwrap(),
            amount: 1u8.into(),
        }],
    };

    // Create body
    let tx_body = tx::Body::new(vec![msg_send.to_any().unwrap()], "memo", block::Height::from(0u32));

    // Create auth info
    let auth_info = tx::AuthInfo {
        signer_infos: vec![tx::SignerInfo {
            public_key: Some(tx::SignerPublicKey::Single(public_key)),
            mode_info: tx::ModeInfo::single(tx::SignMode::Direct),
            sequence: 0,
        }],
        fee: tx::Fee {
            amount: vec![Coin {
                denom: "untrn".parse().unwrap(),
                amount: 1060u64.into(),
            }],
            gas_limit: 200000,
            payer: None,
            granter: None,
        },
    };

    let chain_id = chain::Id::try_from("pion-1").unwrap();

    SignDoc::new(&tx_body, &auth_info, &chain_id, 577723).unwrap()
}

fn get_pubkey(address: String) -> PublicKey {
    let address_bytes = bs58::decode(address).into_vec().unwrap();
    let pubkey = VerifyingKey::from_sec1_bytes(&address_bytes).unwrap();
    PublicKey::from(pubkey)
}
fn get_unsigned_tx_bytes(address: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let sign_doc = sign_doc_data(address);

    // Get the raw bytes that will be hashed and signed by the canister
    let message_bytes = sign_doc.into_bytes()?;

    Ok(message_bytes)
}

fn raw_tx_data(address: String, signature: Vec<u8>) -> Result<Raw, Box<dyn std::error::Error>> {
    let sign_doc = sign_doc_data(address);

    // Create TxRaw with empty signatures
    let tx_raw = cosmrs::proto::cosmos::tx::v1beta1::TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature], // Empty signatures for unsigned transaction
    };

    Ok(Raw::from(tx_raw))
}

fn get_signed_tx_bytes(address: String, signature: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let raw_tx = raw_tx_data(address, signature)?;
    let message_bytes = raw_tx.to_bytes()?;
    Ok(message_bytes)
}

// TODO: fix
// #[test]
#[allow(dead_code)]
fn test_send_transaction() {
    let setup = CosmosWalletSetup::new();

    let raw_tx ="4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT";

    let signature = setup
        .call_update::<_, String>("sendTransaction", (RpcServices::Mainnet, (), raw_tx))
        .mock_http_once(MockOutcallBuilder::new(200, r#"{"jsonrpc":"2.0","result":{"context":{"slot":2792},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":3090}},"id":1}"#))
        .mock_http_once(MockOutcallBuilder::new(200, r#"{"jsonrpc":"2.0","result":"2EanSSkn5cjv9DVKik5gtBkN1wwbV1TAXQQ5yu2RTPGwgrhEywVAQR2veu895uCDzvYwWZe6vD1Bcn8s7r22W17w","id":2}"#))
        .wait();

    println!("signature: {}", signature);
}

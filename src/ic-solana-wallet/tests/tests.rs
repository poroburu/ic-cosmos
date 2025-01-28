use std::str::FromStr;

use cosmrs::bank::MsgSend;
use cosmrs::crypto::PublicKey;
use cosmrs::proto::cosmos::tx::v1beta1::TxRaw;
use cosmrs::tendermint::chain::Id;
use cosmrs::tendermint::{block, chain};
use cosmrs::tx::{self, Body, Msg, Raw, SignDoc};
use cosmrs::{AccountId, Coin, Denom};
use ic_solana::{rpc_client::RpcServices, types::Pubkey};
use test_utils::MockOutcallBuilder;

mod setup;

use crate::setup::SolanaWalletSetup;

#[test]
fn test_address() {
    let setup = SolanaWalletSetup::new();

    let addr = setup.call_update::<_, String>("address", ()).wait();
    assert_eq!(addr, "21Nb5RPUh9q47wq6nsFcb48ngqqndHq4EH5Goyjm3xgmo");
    let addr = setup.as_controller().call_update::<_, String>("address", ()).wait();
    assert_eq!(addr, "247fCqSWyNCyWAJZ5HoXMoZLMgtPRg1WChWFgwWzEshok");
}

#[test]
fn test_sign_message() {
    let setup = SolanaWalletSetup::new();
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
    let setup = SolanaWalletSetup::new();
    let message = get_unsigned_tx_bytes().unwrap();

    let address = setup.call_update::<_, String>("address", ()).wait();
    println!("address: {}", address);
    let pubkey = Pubkey::from_str(&address).unwrap();

    // Now we can pass the bytes directly without encoding
    let signature = setup
        .call_update::<_, Vec<u8>>("signMessage", (message.clone(),))
        .wait();

    // Verify directly with the original message bytes
    let is_valid = pubkey.verify_signature(&message, &signature);
    assert!(is_valid);

    println!("signature: {:?}", signature);
    println!("message(base64): {:?}", base64::encode(&message));
    println!("signature(base64): {:?}", base64::encode(&signature));
    let signed_tx = get_signed_tx_bytes(signature).unwrap();
    println!("signed_tx(base64): {:?}", base64::encode(&signed_tx));
}
// common transaction data
fn sign_doc_data() -> SignDoc {
    let pubkey_json =
        r#"{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"A2NNkZ4Sj7WjERQs+itX7dc+BO8HeSgdEwspUg9Px8Fa"}"#;
    let public_key = PublicKey::from_json(pubkey_json).unwrap();

    // Create message
    let msg_send = MsgSend {
        from_address: "neutron100ge9lnqxrqc9fhsmeavxg3f20g7rcw075pzlq".parse().unwrap(),
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

    SignDoc::new(&tx_body, &auth_info, &chain_id, 577644).unwrap()
}

fn get_unsigned_tx_bytes() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let sign_doc = sign_doc_data();

    // Get the raw bytes that will be hashed and signed by the canister
    let message_bytes = sign_doc.into_bytes()?;

    Ok(message_bytes)
}

fn raw_tx_data(signature: Vec<u8>) -> Result<Raw, Box<dyn std::error::Error>> {
    let sign_doc = sign_doc_data();

    // Create TxRaw with empty signatures
    let tx_raw = cosmrs::proto::cosmos::tx::v1beta1::TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature], // Empty signatures for unsigned transaction
    };

    Ok(Raw::from(tx_raw))
}

fn get_signed_tx_bytes(signature: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let raw_tx = raw_tx_data(signature)?;
    let message_bytes = raw_tx.to_bytes()?;
    Ok(message_bytes)
}

// TODO: fix
// #[test]
#[allow(dead_code)]
fn test_send_transaction() {
    let setup = SolanaWalletSetup::new();

    let raw_tx ="4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT";

    let signature = setup
        .call_update::<_, String>("sendTransaction", (RpcServices::Mainnet, (), raw_tx))
        .mock_http_once(MockOutcallBuilder::new(200, r#"{"jsonrpc":"2.0","result":{"context":{"slot":2792},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":3090}},"id":1}"#))
        .mock_http_once(MockOutcallBuilder::new(200, r#"{"jsonrpc":"2.0","result":"2EanSSkn5cjv9DVKik5gtBkN1wwbV1TAXQQ5yu2RTPGwgrhEywVAQR2veu895uCDzvYwWZe6vD1Bcn8s7r22W17w","id":2}"#))
        .wait();

    println!("signature: {}", signature);
}

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use candid::{CandidType, Principal};
use ic_management_canister_types::{
    DerivationPath, EcdsaCurve, EcdsaKeyId, ECDSAPublicKeyArgs, ECDSAPublicKeyResponse,
    SignWithECDSAArgs, SignWithECDSAReply,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-ecdsa-production-key
pub const ECDSA_SIGN_COST: u128 = 26_153_846_153;

#[derive(Debug, Clone, Deserialize, Serialize, CandidType)]
pub enum EcdsaKey {
    TestKeyLocal,
    TestKey1,
    ProductionKey1,
    Custom(String),
}

impl Display for EcdsaKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-ecdsa
        let key_str = match self {
            EcdsaKey::TestKeyLocal => "dfx_test_key",
            EcdsaKey::TestKey1 => "test_key_1",
            EcdsaKey::ProductionKey1 => "key_1",
            EcdsaKey::Custom(key) => key,
        };
        f.write_str(key_str)
    }
}

impl FromStr for EcdsaKey {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dfx_test_key" => Ok(EcdsaKey::TestKeyLocal),
            "test_key_1" => Ok(EcdsaKey::TestKey1),
            "key_1" => Ok(EcdsaKey::ProductionKey1),
            _ => Ok(EcdsaKey::Custom(s.to_string())),
        }
    }
}

/// Fetches the secp256k1 public key from the cosmos canister.
pub async fn ecdsa_public_key(key: EcdsaKey, derivation_path: Vec<ByteBuf>) -> Vec<u8> {
    let res: Result<(ECDSAPublicKeyResponse,), _> = ic_cdk::call(
        Principal::management_canister(),
        "ecdsa_public_key",
        (ECDSAPublicKeyArgs {
            canister_id: None,
            derivation_path: DerivationPath::new(derivation_path),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key.to_string(),
            },
        },),
    )
    .await;

    res.expect("Failed to fetch secp256k1 public key").0.public_key
}

/// Signs a message with an secp256k1 key.
pub async fn sign_with_ecdsa(key: EcdsaKey, derivation_path: Vec<ByteBuf>, message: Vec<u8>) -> Vec<u8> {
    ic_cdk::api::call::msg_cycles_accept128(ECDSA_SIGN_COST);

    let res: Result<(SignWithECDSAReply,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (SignWithECDSAArgs {
            message_hash: sha256(&message),
            derivation_path: DerivationPath::new(derivation_path),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: key.to_string(),
            },
        },),
        ECDSA_SIGN_COST as u64,
    )
    .await;

    res.expect("Failed to sign with secp256k1").0.signature
}

// https://github.com/dfinity/examples/blob/master/rust/threshold-ecdsa/src/ecdsa_example_rust/src/lib.rs#L81
fn sha256(input: &Vec<u8>) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}
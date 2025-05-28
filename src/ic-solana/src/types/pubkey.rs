use std::{fmt, mem, str::FromStr};

use candid::CandidType;
use ic_crypto_secp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use thiserror::Error;


/// Number of bytes in a pubkey
pub const PUBKEY_BYTES: usize = 32 + 1;

/// Maximum string length of a base58 encoded pubkey
const MAX_BASE58_LEN: usize = 44 + 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, CandidType)]
pub struct Pubkey(pub(crate) [u8; PUBKEY_BYTES]);

impl Default for Pubkey {
    fn default() -> Self {
        Self([0u8; PUBKEY_BYTES])
    }
}

impl<'de> Deserialize<'de> for Pubkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        bytes.try_into()
            .map(Self)
            .map_err(|_| serde::de::Error::custom("Invalid pubkey length"))
    }
}

impl Serialize for Pubkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_vec().serialize(serializer)
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParsePubkeyError {
    #[error("String is the wrong size")]
    WrongSize,
    #[error("Invalid Base58 string")]
    Invalid,
}

impl Pubkey {
    pub fn new(key: [u8; PUBKEY_BYTES]) -> Self {
        Self(key)
    }

    pub fn to_bytes(self) -> [u8; PUBKEY_BYTES] {
        self.0
    }

    /// Verify an secp256k1 signature
    ///
    /// Returns Ok if the signature is valid, or Err otherwise
    pub fn verify_signature(&self, msg: &[u8], signature: &[u8]) -> bool {
        println!("self: {:?}", self);
        // Reconstruct the full compressed SEC1 format (33 bytes)
        // Use 0x02 prefix for even y-coordinate, 0x03 for odd
        //let prefix = if self.0[31] & 1 == 0 { 0x02 } else { 0x03 };
        //let mut sec1_bytes = vec![prefix];
        //sec1_bytes.extend_from_slice(&self.0);  // Add the x-coordinate we stored

        let pubkey = PublicKey::deserialize_sec1(&self.0).expect("invalid public key");
        pubkey.verify_ecdsa_signature(msg, signature)
    }
}

impl FromStr for Pubkey {
    type Err = ParsePubkeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > MAX_BASE58_LEN{
            return Err(ParsePubkeyError::WrongSize);
        }
        let pubkey_vec = bs58::decode(s).into_vec().map_err(|_| ParsePubkeyError::Invalid)?;
        if pubkey_vec.len() != mem::size_of::<Pubkey>() {
            Err(ParsePubkeyError::WrongSize)
        } else {
            Pubkey::try_from(pubkey_vec).map_err(|_| ParsePubkeyError::Invalid)
        }
    }
}

impl From<[u8; PUBKEY_BYTES]> for Pubkey {
    #[inline]
    fn from(from: [u8; PUBKEY_BYTES]) -> Self {
        Self(from)
    }
}

impl TryFrom<&[u8]> for Pubkey {
    type Error = std::array::TryFromSliceError;

    #[inline]
    fn try_from(pubkey: &[u8]) -> Result<Self, Self::Error> {
        <[u8; PUBKEY_BYTES]>::try_from(pubkey).map(Self::from)
    }
}

impl TryFrom<Vec<u8>> for Pubkey {
    type Error = std::array::TryFromSliceError;

    #[inline]
    fn try_from(pubkey: Vec<u8>) -> Result<Self, Self::Error> {
        println!("pubkey: {:?}", pubkey);
        <[u8; PUBKEY_BYTES]>::try_from(&pubkey[..]).map(Self::from)
    }
}

impl TryFrom<&str> for Pubkey {
    type Error = ParsePubkeyError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Pubkey::from_str(s)
    }
}

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

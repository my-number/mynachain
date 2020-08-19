use crate::certs;
use frame_support::dispatch::{Decode, Encode, Vec};
use myna::crypto;
use rsa::RSAPublicKey;
use sp_core::{Blake2Hasher, Hasher};
pub type AccountId = [u8; 32];
pub type Signature = Vec<u8>;
pub type Nonce = u64;
pub type Balance = u64;

/// The struct of individual account
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Account {
    pub cert: Vec<u8>,
    pub id: AccountId,
    pub nonce: Nonce,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SignedData {
    pub tbs: Tx,
    pub signature: Signature,
    pub id: AccountId,
}
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub enum Tx {
    CreateAccount(TxCreateAccount),
    Send(TxSend),
    Mint(TxMint),
    Vote(TxVote),
    Other,
}
impl Default for Tx {
    fn default() -> Self {
        Tx::Other
    }
}
impl SignedData {
    pub fn verify(&self, pubkey: RSAPublicKey) -> Result<(), &'static str> {
        let encoded = self.tbs.encode();
        let sighash = Blake2Hasher::hash(&encoded);

        match crypto::verify(pubkey, sighash.as_ref(), &self.signature[..]) {
            Ok(()) => return Ok(()),
            Err(_) => return Err("Verification failed"),
        }
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxCreateAccount {
    pub cert: Vec<u8>,
    pub nonce: Nonce,
}

impl TxCreateAccount {
    pub fn check_ca(&self) -> Result<(), &'static str> {
        for ca in certs::auth_ca.iter() {
            if crypto::verify_cert(&self.cert[..], ca).is_ok() {
                return Ok(());
            }
        }
        return Err("Failed to check CA");
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxSend {
    pub to: AccountId,
    pub amount: Balance,
    pub nonce: Nonce,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxMint {
    pub amount: Balance,
    pub nonce: Nonce,
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxVote {
    pub amount: Balance,
    pub nonce: Nonce,
}

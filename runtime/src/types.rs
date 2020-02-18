use frame_support::dispatch::{Decode, Encode, Vec};
use myna::crypto;
use sp_core::{Blake2Hasher, Hasher};
use rsa::RSAPublicKey;
use crate::certs;

pub type AccountId = u64;
pub type Signature = Vec<u8>;
pub type uNonce = u64;
pub type Balance = u64;

/// The struct of individual account
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Account {
    pub cert: Vec<u8>,
    pub id: AccountId,
    pub nonce: uNonce,
}

pub trait Signed {
    fn get_id(&self) -> &AccountId;
    fn get_signature(&self) -> &Signature;
    fn verify(&self, pubkey: RSAPublicKey)->Result<(), &'static str>;
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SignedData<Tx> where Tx: Encode + Clone {
    pub tbs: Tx,
    pub signature: Signature,
    pub id: AccountId,
}

impl<Tx> Signed for SignedData<Tx> where Tx: Encode + Clone {
    fn get_id(&self) -> &AccountId{
        &self.id
    }
    fn get_signature(&self) -> &Signature{
        &self.signature
    }
    fn verify(&self, pubkey: RSAPublicKey)->Result<(), &'static str> {
        let encoded = self.tbs.encode();
        let sighash = Blake2Hasher::hash(&encoded);
        match crypto::verify(pubkey, sighash.as_ref(), &self.signature[..]){
            Ok(())=>return Ok(()),
            Err(_)=>return Err("Verification failed")
        }
    }
}


#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxCreateAccount {
    pub cert: Vec<u8>,
    pub nonce: uNonce
}

impl TxCreateAccount {
    pub fn check_ca(&self) -> Result<(), &'static str>  {
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
    pub nonce: uNonce
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxMint {
    pub amount: Balance,
    pub nonce: uNonce
}

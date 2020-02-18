use frame_support::dispatch::{Decode, Encode, Vec};
use myna::crypto;
use sp_core::Blake2Hasher;
use crate::certs;

pub type AccountId = u64;
pub type Signature = Vec<u8>;
pub type uNonce = u64;
/// The struct of individual account
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Account {
    pub cert: Vec<u8>,
    pub id: AccountId,
    pub nonce: uNonce,
}
pub type Balance = u64;

pub trait Nonce {
    fn get_nonce(&self) -> uNonce {
        self.nonce
    }
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SignedData<T> where T: Encode + Decode + Default + Clone + PartialEq + Nonce {
    pub tbs: T,
    pub signature: Signature,
    pub id: AccountId,
}


impl<T> SignedData<T> {
    pub fn verify(&self, pubkey: &[u8])->Result<(), &'static str> {
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


impl Nonce for TxCreateAccount{}
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

pub struct TxSend {
    pub to: AccountId,
    pub amount: Balance,
    pub nonce: uNonce
}


impl Nonce for TxSend {}

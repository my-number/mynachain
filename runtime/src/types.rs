use frame_support::dispatch::{Decode, Encode, Vec};
use myna::crypto;
use sp_core::Blake2Hasher;

pub type AccountId = u64;
pub type Signature = Vec<u8>;
pub type uNonce = u64
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
    pub fn get_nonce(&self) -> uNonce;
}
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SignedData<T> where T: Encode + Decode + Default + Clone + PartialEq + Debug + Nonce {
    pub tbs: T,
    pub signature: Signature,
    pub id: AccountId,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct TxCreateAccount {
    pub cert: Vec<u8>,
    pub nonce: uNonce
}

impl Nonce for TxCreateAccount{
    fn get_nonce(&self) -> uNonce {
        self.nonce
    }
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

use crate::types;
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{Decode, DispatchError, DispatchResult, Encode, Vec},
    ensure,
    traits::{Currency, ExistenceRequirement},
};
use sp_std::vec;
use myna::crypto;
use system::{ensure_none, ensure_signed};

/// The module's configuration trait.
pub trait Trait: balances::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MynaChainModule {

        AccountCount get(fn account_count): types::AccountId;
        Accounts get(fn account): map types::AccountId => types::Account;
        Balance get(fn balance): map types::AccountId => types::Balance;
    }
}

decl_event!(
    pub enum Event
    {
        AccountAdd(types::AccountId),
        Transferred(types::AccountId, types::AccountId, types::Balance),
        Minted(types::AccountId, types::Balance),
    }
);
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Create an Account
        /// nonce must be zero
        /// id must be zero
        pub fn create_account(origin, tx: types::SignedData<types::TxCreateAccount>) -> DispatchResult {
            ensure_none(origin)?;
            ensure!(tx.tbs.nonce==0, "Nonce is not zero");
            ensure!(tx.id==0, "Id is not zero");
            tx.tbs.check_ca()?;
            
            let cert: Vec<u8> = tx.tbs.cert;
            let sig: types::Signature = tx.tbs.signature;
            let pubkey = crypto::extract_pubkey(&cert[..]).map_err(|_| "failed to get pubkey")?;
            tx.verify(&pubkey[..]);
            Self::insert_account(cert)?;
            Ok(())
        }

        pub fn send(origin, tx: types::SignedData<types::TxSend>) -> DispatchResult {
            ensure_none(origin)?;
            let from = Self::ensure_rsa_signed(tx)?;
            
            let to = tx.tbs.to;
            let amount = tx.to.amount;
            Self::transfer(from,to, amount)?;
            Self::increase_nonce(from)?;
            Ok(())
        }
        /*
        pub fn mint(origin, signed_data: types::SignedData, amount: types::Balance)-> DispatchResult {
            let from = Self::ensure_rsa_signed(origin, signed_data, vec![0u8])?;
            let pre_bal = Balance::get(from);
            let new_bal = pre_bal.checked_add(amount).ok_or("overflow")?;
            Balance::insert(from, new_bal);

            Self::increment_nonce(from)?;
            Self::deposit_event(Event::Minted(from, amount));

            Ok(())
        }*/
    }
}

impl<T: Trait> Module<T> {
    pub fn insert_account(cert: Vec<u8>) -> DispatchResult {
        
        let new_id = AccountCount::get();
        let new_account = types::Account {
            cert: cert,
            id: new_id,
            nonce: 0,
        };
        Accounts::insert(new_id, new_account);
        AccountCount::mutate(|t| *t += 1);

        Self::deposit_event(Event::AccountAdd(new_id));

        Ok(())
    }
    pub fn ensure_rsa_signed<TxType>(tx: types::SignedData<TxType>) -> Result<types::AccountId, &'static str> {
        ensure!(Accounts::exists(tx.id), "Account not found");
        let account = Accounts::get(tx.id);
        tx.verify()?;
        Ok(account.id)
    }
/*    pub fn check_cert(
        cert: &Vec<u8>,
        sig: types::Signature,
        serialized: &Vec<u8>
    ) -> Result<(), &'static str> {
        
        let pubkey = crypto::extract_pubkey(&cert[..]).map_err(|_| "failed to get pubkey")?;
        crypto::verify(pubkey, &serialized[..], &sig).map_err(|_| "failed to verify")?;
        Ok(())
    }
    
    pub fn ensure_rsa_signed<Origin>(
        origin: Origin,
        signed_data: types::SignedData,
        serialized: Vec<u8>
    ) -> Result<types::AccountId, &'static str> {
        ensure!(Accounts::exists(signed_data.id), "Account not found");
        let account = Accounts::get(signed_data.id);

        Self::check_cert(&account.cert, signed_data.signature, &serialized)?;
        Ok(account.id)
    }
*/
    pub fn transfer(
        from: types::AccountId,
        to: types::AccountId,
        amount: types::Balance,
    ) -> DispatchResult {
        ensure!(Accounts::exists(from), "Account not found");
        ensure!(Accounts::exists(to), "Account not found");

        let pre_bal_from = Balance::get(from);
        let new_bal_from = pre_bal_from.checked_sub(amount).ok_or("overflow")?;

        let pre_bal_to = Balance::get(to);
        let new_bal_to = pre_bal_to.checked_add(amount).ok_or("overflow")?;

        Balance::insert(from, new_bal_from);
        Balance::insert(to, new_bal_to);
        Self::deposit_event(Event::Transferred(from, to, amount));
        Ok(())
    }

    pub fn increment_nonce(id: types::AccountId) -> DispatchResult {
        ensure!(Accounts::exists(id), "Account not found");

        let mut account = Accounts::get(id);
        account.nonce += 1;
        Accounts::insert(id, account);

        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::{assert_ok, impl_outer_origin, parameter_types, weights::Weight};
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }
    impl Trait for Test {
        type Event = ();
    }
    type MynaChainModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        new_test_ext().execute_with(|| {});
    }
}

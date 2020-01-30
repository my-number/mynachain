use frame_support::{
    decl_event, decl_module, decl_storage, dispatch,
    dispatch::{Decode, Encode, Vec},
    traits::{Currency, ExistenceRequirement},
};
use system::{ensure_none, ensure_signed};
use myna::crypto;
use crate::certs;
/// The module's configuration trait.
pub trait Trait: balances::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

mod custom_types {
    use frame_support::dispatch::{Decode, Encode, Vec};
    
    pub type AccountId = u64;
    pub type Signature = Vec<u8>;

    /// The struct of individual account
    #[derive(Encode, Decode, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Account {
        pub cert: Vec<u8>,
        pub id: AccountId,
    }
    pub type Balance = u64;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
    pub struct SignedData {
        pub signature: Signature,
        pub id: AccountId,
    }
}



// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MynaChainModule {

        AccountCount get(account_count): custom_types::AccountId;
        Accounts get(account): map custom_types::AccountId => custom_types::Account;
        Balance get(balance): map custom_types::AccountId => custom_types::Balance;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        Success(AccountId),
    }
);
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        pub fn create_account(origin, cert: Vec<u8>, sig: custom_types::Signature) -> dispatch::Result {
            ensure_none(origin)?;
            // check signed by cacert
            // check cert by signature
            Self::insert_account(cert)?;
            Ok(())
        }

        pub fn send(origin, signed_data: custom_types::SignedData, to: custom_types::AccountId, amount: custom_types::Balance) -> dispatch::Result {
            ensure_none(origin)?;
            let from = Self::ensure_rsa_signed(signed_data)?;
            Self::transfer(from,to, amount)?;
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn check_ca(cert: &Vec<u8>) -> dispatch::Result {
        for ca in certs::auth_ca.iter() {
            if crypto::verify_cert(&cert[..],ca).is_ok() {
                return Ok(());
            }
        };
        return Err("Failed to check CA");
    }
    pub fn insert_account(cert: Vec<u8>) ->dispatch::Result {
        Self::check_ca(&cert)?;
        let new_id = <AccountCount>::get();
        let new_account = custom_types::Account {
            cert,
            id: new_id
        };
        <Accounts>::insert(new_id, new_account);
        <AccountCount>::mutate(|t| *t += 1);
        Ok(())
    }
    pub fn ensure_rsa_signed(
        signed_data: custom_types::SignedData,
    ) -> Result<custom_types::AccountId, &'static str> {
        let account = <Accounts>::get(signed_data.id);
        let pubkey = crypto::extract_pubkey(&account.cert).map_err(|_| "failed to get pubkey")?;
        crypto::verify(pubkey, &[0u8], &signed_data.signature).map_err(|_| "failed to verify");
        Ok(account.id)
    }
    pub fn transfer(
        from: custom_types::AccountId,
        to: custom_types::AccountId,
        amount: custom_types::Balance,
    ) -> dispatch::Result {
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

// ensure_signedみたいに,ensure_accountみたいなの

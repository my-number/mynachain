use frame_support::{
    decl_event, decl_module, decl_storage, dispatch,
    dispatch::{Decode, Encode, Vec},
    traits::{Currency,ExistenceRequirement}
};
use system::{ensure_none, ensure_signed};

/// The module's configuration trait.
pub trait Trait: balances::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/// The struct of individual account
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Account<Hash> {
    cert: Vec<u8>,
    id: Hash,
    initiator: Hash,
}
// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MynaChainModule {

        AccountCount get(account_count): u64;
        Accounts get(account): map T::Hash => Account<T::Hash>;
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

        pub fn create_account(origin, cert: Vec<u8>) -> dispatch::Result {
            ensure_none(origin)?;
            Self::insert_account(cert)?;
            Ok(())
        }

        pub fn send(origin, amount: u64, to: T::AccountId ) -> dispatch::Result {
            let sender = ensure_signed(origin)?;
            <balances::Module<T> as Currency<_>>::transfer(&sender, &to,  <T as balances::Trait>::Balance::try_from(amount).unwrap(), ExistenceRequirement::KeepAlive)
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn insert_account(cert: Vec<u8>) -> dispatch::Result {
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

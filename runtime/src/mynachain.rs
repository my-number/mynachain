use crate::{types, BlockNumber};
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::{Decode, DispatchError, DispatchResult, Encode, Vec},
    ensure,
    traits::{Currency, ExistenceRequirement},
    weights::Weight
};
use myna::crypto;
use sp_std::vec;
use system::{ensure_none, ensure_signed, ensure_root};

use sp_core::{Blake2Hasher, Hasher};
use sp_runtime::traits::CheckedDiv;

pub const DISTRIBUTION_TERM: crate::BlockNumber = 10;
pub const MAX_VOTE_BALANCE_PER_TERM: types::Balance = 10000;
/// The module's configuration trait.
pub trait Trait: balances::Trait {
    // TODO: Add other types and constants required configure this module.
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MynaChainModule {
        AccountCount get(fn account_count): u64;
        AccountEnumerator get(fn account_enum): map u64 => types::AccountId;
        Accounts get(fn account): map types::AccountId => types::Account;
        RawBalance get(fn balance): map types::AccountId => types::Balance;
        CumulativeVotes get(fn votes_cum): map BlockNumber => types::Balance; // 投票の累積和。ちなみにゲッターのcumはCumulativeのprefixです。念の為。
    }
}

decl_event!(
    pub enum Event {
        AccountAdd(types::AccountId),
        Transferred(types::AccountId, types::AccountId, types::Balance),
        Minted(types::AccountId, types::Balance),
        Voted(types::AccountId, types::Balance),
        AlwaysOk,
    }
);
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        
    		#[weight = 0]
        pub fn go(_origin, tx: types::SignedData) -> DispatchResult{
            match tx.clone().tbs {
                types::Tx::CreateAccount(t) => Self::create_account(tx, t),
                types::Tx::Send(t) => Self::send(tx, t),
                types::Tx::Mint(t) => Self::mint(tx, t),
                types::Tx::Vote(t) => Self::vote(tx, t),
                _ => Ok(())
            }
        }
        pub fn on_initialize(block_number: BlockNumber,) -> Weight {
            if block_number % DISTRIBUTION_TERM == 0{
                let term_number = block_number / DISTRIBUTION_TERM;
                let val_n = CumulativeVotes::get(term_number): // N th value
                CumulativeVotes::insert(term_number + 1, val_n); // a[N+1] = a[N]
            }
            0.into()
        }
    }
}

impl<T: Trait> Module<T> {
    /// Create an Account
    /// nonce must be zero
    /// id must be zero
    pub fn create_account(tx: types::SignedData, tbs: types::TxCreateAccount) -> DispatchResult {
        ensure!(tbs.nonce == 0, "Nonce is not zero");

        tbs.check_ca()?;

        let sig = &tx.signature;
        let pubkey = crypto::extract_pubkey(&tbs.cert[..]).map_err(|_| "failed to get pubkey")?;
        tx.verify(pubkey)?;
        Self::insert_account(tbs.cert)?;
        Ok(())
    }

    pub fn send(tx: types::SignedData, tbs: types::TxSend) -> DispatchResult {
        let to = tbs.to;
        let from = Self::ensure_rsa_signed(&tx)?;
        let amount = tbs.amount;
        Self::transfer(from, to, amount)?;
        Self::increment_nonce(from)?;
        Ok(())
    }
    pub fn mint(tx: types::SignedData, tbs: types::TxMint) -> DispatchResult {
        let from = Self::ensure_rsa_signed(&tx)?;
        let amount = tbs.amount;
        let pre_bal = RawBalance::get(from);
        let new_bal = pre_bal.checked_add(amount).ok_or("overflow")?;
        RawBalance::insert(from, new_bal);
        Self::increment_nonce(from)?;
        Self::deposit_event(Event::Minted(from, amount));

        Ok(())
    }
    pub fn vote(tx: types::SignedData, tbs: types::TxVote) -> DispatchResult {
        let from = Self::ensure_rsa_signed(&tx)?;
        let amount = tbs.amount;
        let term = Self::term_number() + 1;
        let pre_bal = CumulativeVotes::get(term);
        let new_bal = pre_bal.checked_add(amount).ok_or("overflow")?;
        ensure!(new_bal <= MAX_VOTE_BALANCE_PER_TERM, "too large amount");

        CumulativeVotes::insert(term, new_bal);
        Self::increment_nonce(from)?;
        Self::deposit_event(Event::Voted(from, amount));

        Ok(())
    }
}
// module func starts here
impl<T: Trait> Module<T> {
    pub fn insert_account(cert: Vec<u8>) -> DispatchResult {
        let new_account_id = Blake2Hasher::hash(&cert[..]);

        ensure!(!Accounts::exists(new_account_id), "Account already exists");

        let new_count = AccountCount::get();

        let new_account = types::Account {
            cert,
            id: new_account_id,
            nonce: 0,
        };
        Accounts::insert(new_account_id, new_account);
        AccountEnumerator::insert(new_count, new_account_id);
        AccountCount::mutate(|t| *t += 1);

        Self::deposit_event(Event::AccountAdd(new_account_id));

        Ok(())
    }
    pub fn ensure_rsa_signed(tx: &types::SignedData) -> Result<types::AccountId, &'static str> {
        ensure!(Accounts::exists(tx.id), "Account not found");
        let account = Accounts::get(tx.id);
        let pubkey =
            crypto::extract_pubkey(&account.cert[..]).map_err(|_| "failed to get pubkey")?;
        tx.verify(pubkey)?;
        Ok(account.id)
    }

    pub fn transfer(
        from: types::AccountId,
        to: types::AccountId,
        amount: types::Balance,
    ) -> DispatchResult {
        ensure!(Accounts::exists(from), "Account not found");
        ensure!(Accounts::exists(to), "Account not found");

        Self::compute_balance(from)
            .checked_sub(amount)
            .ok_or("underflow")?;
        Self::compute_balance(to)
            .checked_add(amount)
            .ok_or("overflow")?;

        let new_rawbal_to = RawBalance::get(to) - amount;
        let new_rawbal_from = RawBalance::get(from) - amount;

        RawBalance::insert(from, new_rawbal_from);
        RawBalance::insert(to, new_rawbal_to);
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
    pub fn term_number() -> BlockNumber {
        (<system::Module<T>>::block_number() / DISTRIBUTION_TERM).into()
    }
    pub fn compute_balance(id: types::AccountId) -> Result<types::Balance, &'static str> {
        ensure!(Accounts::exists(id), "Account not found");
        let raw_bal = RawBalance::get(id);
        let confirmed_sum = Self::votes_cum(Self::term_number());
        let distributed_bal = confirmed_sum;
        Ok(raw_bal + distributed_bal)
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

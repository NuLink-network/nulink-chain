use super::*;
use crate as pallet_nuproxy;
use sp_core::H256;
use frame_support::parameter_types;
use frame_support::{assert_ok};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
pub use pallet_balances;
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;


// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		NulinkPolicy: pallet_policy::{Pallet, Call, Storage, Event<T>},
		NuLinkProxy: pallet_nuproxy::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_policy::Config for Test {
	type Event = Event;
	type Balance = u64;
	type PolicyHandle = NuLinkProxy;
}
parameter_types! {
	pub const NulinkPalletId: PalletId = PalletId(*b"py/proxy");
	pub const InitRewardUnit: u64 = 100;
}

impl Config for Test {
	type Event = Event;
	type Balance = u64;
	type Currency = Balances;
	type GetPolicyInfo = NulinkPolicy;
	type PalletId = NulinkPalletId;
	type RewardUnit = InitRewardUnit;
}

pub const A: u64 = 77;
pub const B: u64 = 78;
pub const OWNER: u64 = 79;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	// system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> { balances: vec![(A, 100), (B, 1000),(OWNER,1000)] };
	genesis.assimilate_storage(&mut t).unwrap();
	t.into()
}
pub fn make_stake_infos(id: u64,lock_balance: u64,count: u32) -> StakeInfo<<Test as frame_system::Config>::AccountId,u64> {
	StakeInfo{
		coinbase: id.clone(),
		workbase: [0;32],
		iswork: true,
		locked_balance: lock_balance,
		workcount: count,
	}
}
pub fn create_policy(id: u64,amount: u64,period: u64,policyid: u128,stakers: Vec<u64>) -> u128 {
	assert_ok!(NulinkPolicy::base_create_policy(id,policyid,amount,period,stakers));
	policyid
}
pub fn set_the_policy(id: u64,value: u64,pid: u128) -> u128 {
	assert_ok!(NuLinkProxy::create_policy(id,value,pid.clone()));
	pid.clone()
}
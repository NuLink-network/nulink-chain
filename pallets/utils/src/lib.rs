#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

// pub use pallet::*;
use frame_support::{dispatch::DispatchResult,inherent::Vec, pallet_prelude::*};
use frame_system::pallet_prelude::*;


pub type PolicyID = u128;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct PolicyInfo<AccountId,BlockNumber> {
	pub pID: PolicyID,
	pub policyStart: BlockNumber,
	pub period: BlockNumber,
	pub policyStop: BlockNumber,
	pub policyOwner:  AccountId,
	pub stackers:  Vec<AccountId>,
}


pub trait GetPolicyInfo<AccountId,PolicyID,BlockNumber> {
	fn get_policy_info_by_pid(pid: PolicyID) -> Result<PolicyInfo<AccountId, BlockNumber>, DispatchError>;
}

pub trait BasePolicy<AccountId,Balance,PolicyID> {
	/// the user create policy and reserve a asset into the vault.
	fn create_policy(who: AccountId,amount: Balance,pid: PolicyID) -> DispatchResult;
}

impl<AccountId, Balance, PolicyID> BasePolicy<AccountId, Balance, PolicyID> for () {
	fn create_policy(who: AccountId,amount: Balance,pid: PolicyID) -> DispatchResult {Ok(())}
}
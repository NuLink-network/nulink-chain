//! # Policy Pallet
//! The policy management pallet will manage the policy fees and distribute
//! them to the stakers accordingly.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use sp_runtime::{traits::{
	AtLeast32BitUnsigned, One,
}, DispatchError};
use frame_support::{ensure,dispatch::DispatchResult,inherent::Vec, pallet_prelude::*};
use codec::MaxEncodedLen;
use nulink_utils::{BasePolicy,GetPolicyInfo,PolicyID,PolicyInfo};


#[macro_use]
extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The units in which we record balances of the outside's balance value.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// the policy handle for pallet nuproxy
		type PolicyHandle: BasePolicy<Self::AccountId,Self::Balance,PolicyID>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn policys)]
	/// Metadata of a staker.
	pub(super) type Policies<T: Config> = StorageMap<_, Blake2_128Concat, PolicyID,
		PolicyInfo<T::AccountId,T::BlockNumber,T::Balance>,
		ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		CreateNewPolicy(PolicyID, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Repeat Policy ID
		RepeatPolicyID,
		/// Not found the Policy
		NotFoundPolicyID,
		/// the policy over period
		PolicyOverPeriod,
		/// the policy not belong to the account
		NotPolicyOwner,
		/// at least one staker
		NotStaker,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allow users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// create policy by user and set the key params to nulink network.
		///
		/// Origin must be Signed.
		/// `pid`: the ID of the policy,produced by the user offline.
		/// `amount`: the amount of the local asset(NlK),used to reward for the
		/// stakers.
		/// `period`: Indicates the time range for the staker to process the policy,
		/// calculated by the number of block numbers.
		/// `stakers`: the worker of the nulink network,it works for the crypto network.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_policy(origin: OriginFor<T>,pid: PolicyID,amount: T::Balance,
		period: T::BlockNumber,stakers: Vec<T::AccountId>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::base_create_policy(owner,pid,amount,period,stakers)
		}
		/// Revoke the policy by the user before they create it. If there is remaining reward
		/// for this policy, it will be returned to the policy creator.
		///
		/// Origin must be Signed.
		/// `pid`: the ID of the policy, produced by the user offline.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn revoke_policy(origin: OriginFor<T>,pid: PolicyID) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::base_revoke_policy(pid,owner)
		}
	}
}

impl<T: Config> Pallet<T>  {
	///
	pub fn base_create_policy(owner: T::AccountId,pid: PolicyID,amount: T::Balance,period: T::BlockNumber,
							  stakers: Vec<T::AccountId>) -> DispatchResult {
		ensure!(!Policies::<T>::contains_key(pid), Error::<T>::RepeatPolicyID);
		ensure!(stakers.len() > 0, Error::<T>::NotStaker);

		// let uni = stakers.clone().into_iter().unique();
		let mut uni_stakers: Vec<T::AccountId> = vec![];
		for s in stakers {
			if !uni_stakers.contains(&s) {
				uni_stakers.push(s.clone());
			}
		}
		// Reserve the asset
		Policies::<T>::insert(pid, PolicyInfo{
			p_id:	pid.clone(),
			period: period,
			policy_start:  frame_system::Pallet::<T>::block_number() + One::one(),
			policy_stop:  period + frame_system::Pallet::<T>::block_number() + One::one(),
			policy_owner: owner.clone(),
			policy_balance: amount.clone(),
			stackers: uni_stakers.clone(),
		});
		T::PolicyHandle::create_policy(owner.clone(),amount,pid.clone(),uni_stakers.clone())?;
		// Emit an event.
		Self::deposit_event(Event::CreateNewPolicy(pid, owner.clone()));

		Ok(())
	}
	pub fn base_revoke_policy(pid: PolicyID,owner: T::AccountId) -> DispatchResult {
		ensure!(Policies::<T>::contains_key(pid), Error::<T>::NotFoundPolicyID);

		Policies::<T>::try_mutate(pid, |policy| -> DispatchResult{
			let cur = frame_system::Pallet::<T>::block_number();
			ensure!(cur > policy.policy_start, Error::<T>::PolicyOverPeriod);
			ensure!(policy.policy_stop >= cur, Error::<T>::PolicyOverPeriod);
			ensure!(policy.policy_owner == owner, Error::<T>::NotPolicyOwner);
			policy.policy_stop = cur;
			Ok(())
		})
	}
	pub fn get_policy_info_by_pid(pid: PolicyID) -> Result<PolicyInfo<T::AccountId, T::BlockNumber,T::Balance>, DispatchError> {
		ensure!(Policies::<T>::contains_key(pid), Error::<T>::NotFoundPolicyID);
		let info = Policies::<T>::get(pid);
		Ok(info.clone())
	}
}

impl<T: Config> GetPolicyInfo<T::AccountId,PolicyID,T::BlockNumber,T::Balance> for Pallet<T> {
	/// Get the policy info by the policy id, it may be called by the nuproxy pallet to calculate
	///  the reward for the epoch.
	fn get_policy_info_by_pid(pid: PolicyID) -> Result<PolicyInfo<T::AccountId, T::BlockNumber,T::Balance>, DispatchError> {
		Self::get_policy_info_by_pid(pid)
	}
}



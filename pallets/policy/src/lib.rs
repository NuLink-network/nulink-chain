#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

use sp_runtime::{traits::{
	AtLeast32BitUnsigned, One, CheckedAdd, CheckedSub,
	Saturating, StaticLookup, Zero, Hash,
}, DispatchError};
use frame_support::{ensure,dispatch::DispatchResult, pallet_prelude::*};
use codec::MaxEncodedLen;
use nulink_utils::{BasePolicy,PolicyID,PolicyInfo};

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
		type PolicyHandle: BasePolicy<Self::Balance,Self::AccountId,PolicyID>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	#[pallet::storage]
	#[pallet::getter(fn policys)]
	/// Metadata of an staker.
	pub(super) type Polices<T> = StorageMap<_, Blake2_128Concat, PolicyID,
		PolicyInfo<T::AccountId,T::BlockNumber>,
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
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_policy(origin: OriginFor<T>,pid: PolicyID,
		period: T::BlockNumber,stakers: Vec<T::AccountId>) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			Self::base_create_policy(owner,pid,period,stakers)
		}
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
		ensure!(!Polices::<T>::contains_key(pid), Error::<T>::RepeatPolicyID);
		// reserve the asset

		Polices::<T>::insert(pid, PolicyInfo{
			pID:	pid.clone(),
			period: period,
			policyStart: One::one() + frame_system::Pallet::<T>::block_number(),
			policyStop: One::one() + period + frame_system::Pallet::<T>::block_number(),
			policyOwner: owner,
			stackers: stakers.clone(),
		});
		T::PolicyHandle::create_policy(owner.clone(),amount,pid.clone())?;
		// Emit an event.
		Self::deposit_event(Event::CreateNewPolicy(pid, owner.clone()));

		Ok(())
	}
	pub fn base_revoke_policy(pid: PolicyID,owner: T::AccountId) -> DispatchResult {
		ensure!(Polices::<T>::contains_key(pid), Error::<T>::NotFoundPolicyID);

		Polices::<T>::try_mutate(pid,|policy| -> DispatchResult{
			let cur = frame_system::Pallet::<T>::block_number();
			ensure!(cur > policy.policyStart, Error::<T>::PolicyOverPeriod);
			ensure!(policy.policyStop >= cur, Error::<T>::PolicyOverPeriod);
			policy.policyStop = cur;
			Ok(())
		})
	}
	pub fn get_policy_info_by_pid(pid: PolicyID) -> Result<PolicyInfo<T::AccountId, T::BlockNumber>, DispatchError> {
		ensure!(Polices::<T>::contains_key(pid), Error::<T>::NotFoundPolicyID);
		let info = Polices::<T>::get(pid);
		Ok(info.clone())
	}
}

impl<T: Config> BasePolicyInfo<T::AccountId,PolicyID,T::BlockNumber> for Pallet<T> {
	fn get_policy_info_by_pid(pid: PolicyID) -> Result<PolicyInfo<T::AccountId, T::BlockNumber>, DispatchError> {
		Self::get_policy_info_by_pid(pid)
	}
}
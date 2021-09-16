#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;
use sp_runtime::DispatchResult;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type PolicyID = u128;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen)]
pub struct PolicyInfo<AccountId,BlockNumber> {
	pub(super) pID: PolicyID,
	pub(super) policyPeriod: BlockNumber,
	pub(super) policyStop: BlockNumber,
	pub(super) policyOwner:  AccountId,
	pub(super) stackers:  Vec<AccountId>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

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
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn crate_policy(origin: OriginFor<T>,pid: PolicyID,
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
	pub fn base_create_policy(owner: T::AccountId,pid: PolicyID,period: T::BlockNumber,stakers: Vec<T::AccountId>) -> DispatchResult {
		ensure!(!Polices::<T>::contains_key(pid), Error::<T>::RepeatPolicyID);
		Polices::<T>::insert(pid, PolicyInfo{
			pID:	pid,
			policyPeriod: period + frame_system::Pallet::<T>::block_number(),
			policyStop: period + frame_system::Pallet::<T>::block_number(),
			policyOwner: owner,
			stackers: stakers.clone(),
		});
		// Emit an event.
		Self::deposit_event(Event::CreateNewPolicy(pid, owner.clone()));
		Ok(())
	}
	pub fn base_revoke_policy(pid: PolicyID,owner: T::AccountId) -> DispatchResult {
		ensure!(Polices::<T>::contains_key(pid), Error::<T>::NotFoundPolicyID);
		Polices::<T>::try_mutate(pid,|policy| -> DispatchResult{
			let cur = frame_system::Pallet::<T>::block_number();
			if policy.policyStop > cur {
				policy.policyStop = cur;
				Ok(())
			} else {
				Error::<T>::PolicyOverPeriod.into()
			}
		})
	}
}

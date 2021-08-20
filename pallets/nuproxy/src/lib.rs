#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod types;


pub use pallet::*;
use sp_runtime::{traits::{
	AtLeast32BitUnsigned, One, CheckedAdd, CheckedSub, Saturating, StaticLookup, Zero,
}, ArithmeticError, DispatchResult};
pub use types::{StakeInfo};
use sp_runtime::traits::Hash;
use frame_support::traits::Get;
use parity_scale_codec::Joiner;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::traits::ReservableCurrency;


	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The units in which we record balances of the outside's balance value.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// The balance unit for the staker's reward.
		#[pallet::constant]
		type RewardUnit: Get<BalanceOf<Self>>;
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

	// the wather can only start working after registration
	#[pallet::storage]
	#[pallet::getter(fn watchers)]
	pub(super) type Watchers<T> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	/// Metadata of an staker.
	pub(super) type Stakers<T> = StorageMap<_, Blake2_128Concat, T::Hash,
		StakeInfo<T::AccountId, T::Balance>,
		ValueQuery>;

	#[pallet::storage]
	/// reserved rewards of the stakers,staker need claim it.
	pub(super) type Rewards<T> =  StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;


	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		AlreadyExist,
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
		pub fn set_watcher(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			match <Watchers<T>>::get(who.clone()) {
				None => {
					<Watchers<T>>::insert(who.clone(),1);
					Ok(())
				},
				Some(val) => {
					Err(Error::<T>::AlreadyExist)?
				},
			}
		}
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_staker_infos_and_mint(origin: OriginFor<T>,
							   infos: Vec<StakeInfo<T::AccountId,T::Balance>>) -> DispatchResult {

			Self::mint_by_watcher(Self::calc_reward_by_epoch())?;
			Self::update_stakers(infos)
		}
	}
}

impl<T: Config> Pallet<T>  {

	pub fn calc_staker_hash(staker: StakeInfo<T::AccountId,T::Balance>) -> T::Hash {
		let mut s = staker.clone();
		s.iswork = false;
		s.lockedBalance = Zero::zero();
		T::Hashing::hash_of(&s)
	}
	pub fn calc_reward_by_epoch() -> T::Balance {
		let unit = T::RewardUnit::get();
		One::one()
	}
	pub fn get_total_staking() -> T::Balance {
		Stakers::<T>::iter()
			.filter(|&(_,val)| val.iswork)
			.map(|(_,val)| val.clone())
			.fold(Zero::zero(),|acc,v|{
				acc + v.lockedBalance
			})
	}
	pub fn exist_watcher(watcher: T::AccountId) -> bool {
		Watchers::<T>::get(watcher.clone()).is_one()
	}
	pub fn remove_unused_watcher() {
		let unused: Vec<_> = Watchers::<T>::iter()
			.filter(|&(_, &v)| v == 0)
			.map(|(k, _)| k.clone())
			.collect();
		for k in unused { Watchers::<T>::remove(&k); }
	}
	pub fn update_stakers(new_stakers: Vec<StakeInfo<T::AccountId,T::Balance>>) -> DispatchResult {
		let keys = Stakers::<T>::iter_keys().collect::<Vec<_>>();
		for key in keys {Stakers::<T>::mutate(key.clone(),|value|{
			value.iswork = false;
		})}
		for new_staker in new_stakers {
			let new_key = Self::calc_staker_hash(new_staker.clone());
			if Stakers::<T>::contains_key(new_key.clone()) {
				Stakers::<T>::mutate(new_key.clone(),|value|{
					value.iswork = true;
				});
			} else {
				Stakers::<T>::insert(new_key.clone(),new_staker);
			}
		}
		Ok(())
	}
	pub fn mint_by_staker(all_reward: T::Balance) -> DispatchResult {
		let total = Self::get_total_staking();
		// Stakers::<T>::iter()
		// 	.filter(|&(_,val)| val.iswork)
		// 	.map(|(_,val)| {
		//
		// 	});
		Ok(())
	}
}

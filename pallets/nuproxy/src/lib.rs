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
	AtLeast32BitUnsigned, One, CheckedAdd, CheckedSub,
	Saturating, StaticLookup, Zero,Hash,
}, ArithmeticError};
pub use types::{StakeInfo};
use frame_support::{
	traits::{Get,ReservableCurrency, ExistenceRequirement::AllowDeath, Currency},
	dispatch::DispatchResult, pallet_prelude::*, PalletId
};
use parity_scale_codec::Joiner;
use sp_runtime::traits::AccountIdConversion;
use sp_core::crypto::Ss58AddressFormat::ZeroAccount;

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
		/// The currency trait.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The balance unit for the staker's reward.
		#[pallet::constant]
		type RewardUnit: Get<BalanceOf<Self>>;
		/// The nulink's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
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
		/// Account hasn't reward in the epoch
		AccountNotExist,
		/// Account balance must be greater than or equal to the transfer amount
		BalanceLow,
		/// Balance should be non-zero
		BalanceZero,
		/// Vault balance must be greater than or equal to the reward amount
		VaultBalanceLow,
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

			Self::mint_by_staker(Self::calc_reward_by_epoch())?;
			Self::update_stakers(infos)
		}
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn claim_reward_by_staker(origin: OriginFor<T>,amount: T::Balance) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::base_reward(staker,amount)
		}
	}
}

impl<T: Config> Pallet<T>  {
	/// The account ID of the treasury pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
	pub fn vault_balance() -> T::Balance {
		let vault = Self::account_id();
		T::Currency::free_balance(&vault)
	}
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
		let cur_all_reward: Vec<_> = Stakers::<T>::iter()
			.filter(|&(_,val)| val.iswork)
			.map(|(_,val)| {
				let reward = (val.lockedBalance.clone() * all_reward.clone()) / total.clone();
				(val.coinbase.clone(),reward)
			})
			.collect();

		let count = cur_all_reward.len();
		let mut all: T::Balance = Zero::zero();
		let mut left: T::Balance = Zero::zero();

		for i in 0..count {
			if i == count - 1 {
				left = all_reward.saturating_sub(all.clone());
			} else {
				left = cur_all_reward[i].1.clone();
			}
			all = all.saturating_add(left.clone());
			Rewards::<T>::mutate(cur_all_reward[i].0.clone(),|b| -> DispatchResult {
				let amount = b.saturating_add(left.clone());
				*b = *amount;
				Ok(())
			})
		}
		Ok(())
	}
	pub fn base_reward(staker: T::AccountId,amount: T::Balance) -> DispatchResult {
		ensure!(amount >= Zero::zero(), Error::<T>::BalanceLow);

		if !Rewards::<T>::contains_key(staker.clone()) {
			Err(Error::<T>::AccountNotExist)?
		}

		Rewards::<T>::mutate(staker.clone(), |&mut old_balance| -> DispatchResult {
			ensure!(old_balance >= amount, Error::<T>::BalanceLow);
			ensure!(Self::vault_balance() >= amount, Error::<T>::VaultBalanceLow);

			old_balance = old_balance.checked_sub(&amount)
				.ok_or(Error::<T>::BalanceLow)?;
			let valut: T::AccountId = Self::account_id();
			T::Currency::transfer(&valut,&staker,amount,AllowDeath)
		})
	}
}

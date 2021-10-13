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
	Saturating, StaticLookup, Zero,Hash,AccountIdConversion,
}};
pub use types::{StakeInfo};
use frame_support::{
	ensure,
	traits::{Get,ReservableCurrency, ExistenceRequirement::AllowDeath, Currency},
	dispatch::DispatchResult, pallet_prelude::*, PalletId
};

use codec::MaxEncodedLen;
use crate::types::BasePolicy;
use nulink_utils::{PolicyID,PolicyInfo,GetPolicyInfo};

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
		/// the policy infos handle for pallet policy
		type GetPolicyInfo: GetPolicyInfo<Self::AccountId,PolicyID,Self::BlockNumber>;
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
	#[pallet::getter(fn stakers)]
	/// Metadata of an staker.
	pub(super) type Stakers<T> = StorageMap<_, Blake2_128Concat, T::Hash,
		StakeInfo<T::AccountId, T::Balance>,
		ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn policy_reserve)]
	/// reserve asset for policy assigned to the stakers
	pub(super) type PolicyReserve<T> =  StorageMap<_, Blake2_128Concat, u128, (T::AccountId,T::Balance,T::BlockNumber), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rewards)]
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
		/// Reserve asset to the vault
		ReserveBalance(T::AccountId,T::Balance),
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
		/// low block number for Policy Reward
		LowBlockNumber,
		/// not found the reserve for the policy id
		NoReserve,
		RepeatReserve,
		/// watcher not exist
		NoWatcher,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {

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
		/// update the staker infos and calc reward by epoch with the called by watchers
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn update_staker_infos_and_mint(origin: OriginFor<T>,
							   infos: Vec<StakeInfo<T::AccountId,T::Balance>>) -> DispatchResult {
			let watcher = ensure_signed(origin)?;
			ensure!(Self::exist_watcher(watcher), Error::<T>::NoWatcher);

			Self::mint_by_staker(Self::calc_reward_by_epoch())?;
			Self::reward_in_epoch(frame_system::Pallet::<T>::block_number())?;
			Self::update_stakers(infos)
		}
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn claim_reward_by_staker(origin: OriginFor<T>,amount: T::Balance) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::base_reward(staker,amount)
		}
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn reserve_to_vault(origin: OriginFor<T>,amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(amount >= Zero::zero(), Error::<T>::BalanceLow);

			let valut: T::AccountId = Self::account_id();
			T::Currency::transfer(&who,&valut,amount,AllowDeath)?;

			// Emit an event.
			Self::deposit_event(Event::ReserveBalance(who, amount));
			Ok(())
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
	pub fn coinbase_to_staker_key(accounts: Vec<T::AccountId>) -> Vec<T::Hash> {
		let all_keys = Stakers::<T>::iter_keys().collect::<Vec<_>>();
		let keys: Vec<_> = Stakers::<T>::iter()
			.filter(|&(_,val)| accounts.into_iter().find(|x|x==val.coinbase).is_some())
			.map(|(i,val)| {
				all_keys[i].clone()
			})
			.collect();
		keys
	}
	pub fn get_policy_by_pallet(pid: PolicyID) -> Result<PolicyInfo<AccountId, BlockNumber>, DispatchError> {
		T::GetPolicyInfo::get_policy_info_by_pid(pid)
	}
	pub fn assigned_by_policy_reward(keys: Vec<T::AccountId>,allAmount: T::Balance) -> DispatchResult {
		let count = keys.len();
		let amount = allAmount / count.into();
		for i in 0..count {
			Rewards::<T>::mutate(keys[i].clone(),|b| -> DispatchResult {
				let new_amount = b.saturating_add(amount);
				*b = *new_amount;
				Ok(())
			})
		}
		Ok(())
	}
	/// calc every policy reward by epoch
	pub fn calc_reward_in_policy(num: T::BlockNumber,pid: PolicyID) -> DispatchResult {
		ensure!(PolicyReserve::<T>::contains_key(pid), Error::<T>::NoReserve);

		match Self::get_policy_by_pallet(pid) {
			Ok(info) => {
				ensure!(num >= info.policyStart, Error::<T>::LowBlockNumber);
				let range = info.period;

				if let Some((_,reserve,last)) = PolicyReserve::<T>::get(pid) {
					let mut lastAssign = last;
					if lastAssign == Zero::zero() {
						lastAssign = info.policyStart
					}
					ensure!(num >= lastAssign, Error::<T>::LowBlockNumber);
					let useblock = num - lastAssign;

					if lastAssign >= info.policyStop || useblock == Zero::zero() {
						/// user was revoke the policy and stop it
						return Ok(())
					}

					let mut all = reserve * useblock.into() as u128 / range.into() as u128;
					if all > reserve {
						all = reserve;
					}

					Self::assigned_by_policy_reward(info.stackers.clone(),all)?;

					PolicyReserve::<T>::mutate(pid,|x|->DispatchResult {
						let new_amount = x.1.saturating_sub(all);
						*x.1 = *new_amount;
						*x.2 = num;
						Ok(())
					})
				} else {
					Err(Error::<T>::NoReserve)?
				}
			},
			Err(e) => e,
		}
	}

	pub fn reward_in_epoch(num: T::BlockNumber) -> DispatchResult {
		let all_keys = PolicyReserve::<T>::iter_keys().collect::<Vec<_>>();
		for i in 0..all_keys.len() {
			Self::calc_reward_in_policy(num,all_keys[i]);
		}
		Ok(())
	}
}

impl<T: Config> BasePolicy<T::AccountId,T::Balance,T::PolicyID> for Pallet<T> {
	/// policy owner will reserve asset(local asset) to the vault when create policy.
	fn create_policy(who: T::AccountId,amount: T::Balance,pid: T::PolicyID) -> DispatchResult {
		ensure!(!PolicyReserve::<T>::contains_key(pid), Error::<T>::RepeatReserve);

		PolicyReserve::<T>::mutate(pid, |&mut old_balance| -> DispatchResult {
			*old_balance = (who,amount,Zero::zero());
			let valut: T::AccountId = Self::account_id();
			T::Currency::transfer(&who,&valut,amount,AllowDeath)
		})
	}
}
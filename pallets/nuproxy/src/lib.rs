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

#[macro_use]
extern crate alloc;

pub use pallet::*;
use sp_runtime::{traits::{
	AtLeast32BitUnsigned, One, CheckedSub,
	Saturating, Zero,Hash,AccountIdConversion,
}};
use sp_std::convert::TryInto;
pub use types::{StakeInfo};
use frame_support::{
	ensure,inherent::Vec,
	traits::{Get,ReservableCurrency, ExistenceRequirement::AllowDeath, Currency},
	dispatch::DispatchResult, pallet_prelude::*, PalletId
};

use codec::MaxEncodedLen;
use crate::types::BalanceOf;
use nulink_utils::{PolicyID,PolicyInfo,GetPolicyInfo,BasePolicy};

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

	// the wather can only start working after registration
	#[pallet::storage]
	#[pallet::getter(fn watchers)]
	pub(super) type Watchers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn stakers)]
	/// Metadata of an staker.
	pub(super) type Stakers<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash,
		StakeInfo<T::AccountId, BalanceOf<T>>,
		ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn policy_reserve)]
	/// reserve asset for policy assigned to the stakers
	/// key: policy id
	/// Accountid: the owner of the policy's creator
	/// Balance: the asset of the policy which use to assigned to stakers
	/// BlockNumber: the block number when the reward was last distributed
	pub(super) type PolicyReserve<T: Config> =  StorageMap<_, Blake2_128Concat, u128,
		(T::AccountId,BalanceOf<T>,T::BlockNumber), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rewards)]
	/// reserved rewards of the stakers or policy creator,staker or the creator need claim it.
	pub(super) type Rewards<T: Config> =  StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;


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
		ReserveBalance(T::AccountId,BalanceOf<T>),
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
		/// the period of the policy was invalid
		InValidPeriod,
		/// convert failed from blocknumber to i32
		ConvertFailed,
		/// only one watcher
		OnlyOneWatcher,
		InvalidStaker,
		RepeatStakerCoinBase,
		NoStakerKey,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// register the watcher
		/// ps: Only supports one watcher for the time being
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn register_watcher(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::exist_watcher(who.clone()),Error::<T>::AlreadyExist);
			let count = <Watchers<T>>::iter().count() as u64;
			ensure!(count < 1 ,Error::<T>::OnlyOneWatcher);
			<Watchers<T>>::insert(who.clone(),1);
			Ok(())
		}
		/// Update the staker infos and calculate rewards each epoch, will called by watchers.
		/// Update the staker infos from ethereum network and incentive stakers every epoch if
		/// the staker still works in the ethereum. If staker stops working, the watcher will periodically
		/// notify the nulink network and stop rewarding it.
		///
		/// `infos`: the new stakers in next epoch from ethereum by watcher set.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn update_staker_infos_and_mint(origin: OriginFor<T>,infos: Vec<StakeInfo<T::AccountId,BalanceOf<T>>>) -> DispatchResult {
			let watcher = ensure_signed(origin)?;
			ensure!(Self::exist_watcher(watcher), Error::<T>::NoWatcher);

			let all_reward = Self::calc_reward_by_epoch();
			if Self::vault_balance() >= all_reward {
				Self::mint_by_staker(all_reward)?;
			}
			Self::reward_in_epoch(frame_system::Pallet::<T>::block_number())?;
			Self::update_stakers(infos)?;
			Self::remove_unused_staker();
			Ok(())
		}
		/// claim the reward by the staker account after the every epoch.
		///
		/// Origin must be Signed.
		/// `amount`: the amount asset(NLK) to be claimed.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn claim_reward_by_staker(origin: OriginFor<T>,amount: BalanceOf<T>) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			Self::base_reward(staker,amount)
		}
		
		/// The user who revokes the policy can claim the remaining assets(NLK) 
		/// from the creation of the policy store.
		///
		/// Origin must be Signed,the user account who revoked the policy.
		/// `amount`: the amount asset(NLK) to be claimed.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn claim_reward_by_user(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			let amount = Rewards::<T>::get(account.clone());
			Self::base_reward(account,amount)
		}
		/// it wiil reserve asset(NLK) to vault for reward stakers by every epoch.
		///
		///  Origin must be Signed.
		/// `amount`: the amount asset(NLK) to be reserve.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn reserve_to_vault(origin: OriginFor<T>,amount: BalanceOf<T>) -> DispatchResult {
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
	/// The account ID of the treasury.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}
	/// the balance(local balance(NLK)) of the vault.
	///
	/// the vault's balance can reserved by any user use local balance.
	pub fn vault_balance() -> BalanceOf<T> {
		let vault = Self::account_id();
		T::Currency::free_balance(&vault)
	}
	/// get all stakers in the pallet.
	///
	/// include all stakers include those that are no longer working.
	pub fn get_staker_count() -> u64 {
		return Stakers::<T>::iter().count() as u64;
	}
	/// calc staker hash with coinbase,workbase,workcount field.
	///
	/// iswork = false and locked_balance = 0. it is w
	pub fn calc_staker_hash(staker: StakeInfo<T::AccountId,BalanceOf<T>>) -> T::Hash {
		let mut s = staker.clone();
		s.iswork = false;
		s.locked_balance = Zero::zero();
		T::Hashing::hash_of(&s)
	}
	/// calc all reward for stakers by every epoch trigger by watcher.
	pub fn calc_reward_by_epoch() -> BalanceOf<T> {
		let unit = T::RewardUnit::get();
		unit
		// One::one()
	}
	/// get all balance by all staker's staking
	pub fn get_total_staking() -> BalanceOf<T> {
		Stakers::<T>::iter()
			.filter(|(_,val)| val.iswork)
			.map(|(_,val)| val.clone())
			.fold(Zero::zero(),|acc,v|{
				acc + v.locked_balance
			})
	}
	pub fn exist_watcher(watcher: T::AccountId) -> bool {
		Watchers::<T>::get(watcher.clone()).is_one()
	}
	pub fn remove_unused_watcher() {
		let unused: Vec<_> = Watchers::<T>::iter()
			.filter(|&(_, v)| v == 0)
			.map(|(k, _)| k.clone())
			.collect();
		for k in unused { Watchers::<T>::remove(&k); }
	}
	pub fn remove_unused_staker() {
		let unused_keys: Vec<_> = Stakers::<T>::iter()
			.filter(|(_,val)| !val.iswork && val.workcount > 1)
			.map(|(k, _)| k.clone())
			.collect();
		for k in unused_keys { Stakers::<T>::remove(&k); }
	}
	/// get amount of the reward by stake's coinbase.
	pub fn get_staker_reward_by_coinbase(account: T::AccountId) -> BalanceOf<T> {
		Rewards::<T>::get(account)
	}
	pub fn valid_staker(account: T::AccountId) -> bool {
		let v: Vec<(T::AccountId,bool)> = Stakers::<T>::iter()
			.filter(|(_,val)| val.iswork && val.coinbase == account)
			.map(|(_,val)| (val.coinbase.clone(),val.iswork))
			.collect();
		v.len() > 0
	}
	pub fn has_staker_by_coinbase(account: T::AccountId) -> bool {
		let v: Vec<T::AccountId> = Stakers::<T>::iter()
			.filter(|(_,val)| val.coinbase == account)
			.map(|(_,val)| val.coinbase.clone())
			.collect();
		v.len() > 0
	}
	/// First,make all old stakers stopping `iswork=false`.
	/// If the staker which in `old` still in next epoch will be added again.
	/// add the new stakers in to the nulink.
	pub fn update_stakers(new_stakers: Vec<StakeInfo<T::AccountId,BalanceOf<T>>>) -> DispatchResult {
		let staker_count = new_stakers.len();
		let mut uni_stakers: Vec<T::AccountId> = vec![];
		for s in new_stakers.clone() {
			if !uni_stakers.contains(&s.coinbase) {
				uni_stakers.push(s.coinbase.clone());
			}
		}
		ensure!(staker_count == uni_stakers.len(),Error::<T>::RepeatStakerCoinBase);

		let keys = Stakers::<T>::iter()
			.map(|(x, _)| x)
			.collect::<Vec<_>>();

		for key in keys {
			Stakers::<T>::mutate(key.clone(),|value|{
				value.iswork = false;
				value.workcount += 1;
		})}

		for new_staker in new_stakers {
			let new_key = Self::calc_staker_hash(new_staker.clone());
			//if Stakers::<T>::contains_key(new_key.clone()) {
			if Self::has_staker_by_coinbase(new_staker.coinbase.clone()) {
				match Self::get_key_by_coinbase(new_staker.coinbase.clone()) {
					Ok(key) => Stakers::<T>::mutate(key.clone(),|value|{
						value.iswork = true;
						value.workcount = 0;
						value.locked_balance = new_staker.locked_balance;
						value.workbase = new_staker.workbase.clone();
					}),
					Err(e) => continue,
				}
			} else {
				Stakers::<T>::insert(new_key.clone(),new_staker);
			}
		}
		Ok(())
	}
	/// In each epoch, the nulink start to allocate a fixed reward to all stakers, and rewards will
	/// be distributed proportionally according to the stakers' stake.
	/// the stakers can claim their rewards in any time.
	pub fn mint_by_staker(all_reward: BalanceOf<T>) -> DispatchResult {
		let total = Self::get_total_staking();
		let cur_all_reward: Vec<_> = Stakers::<T>::iter()
			.filter(|(_,val)| val.iswork)
			.map(|(_,val)| {
				let reward = (val.locked_balance.clone() * all_reward.clone()) / total.clone();
				(val.coinbase.clone(),reward)
			})
			.collect();

		let count = cur_all_reward.len();
		let mut all: BalanceOf<T> = Zero::zero();
		let mut left: BalanceOf<T> = Zero::zero();

		for i in 0..count {
			if i == count - 1 {
				left = all_reward.saturating_sub(all.clone());
			} else {
				left = cur_all_reward[i].1.clone();
			}
			all = all.saturating_add(left.clone());
			Rewards::<T>::mutate(cur_all_reward[i].0.clone(),|b| -> DispatchResult {
				let amount = b.saturating_add(left.clone());
				*b = amount;
				Ok(())
			}).unwrap();
		}
		Ok(())
	}
	pub fn base_reward(account: T::AccountId,balance: BalanceOf<T>) -> DispatchResult {
		ensure!(balance >= Zero::zero(), Error::<T>::BalanceLow);
		ensure!(Rewards::<T>::contains_key(account.clone()),Error::<T>::AccountNotExist);
		let amount: BalanceOf<T> = balance.clone();

		Rewards::<T>::mutate(account.clone(), |val| -> DispatchResult {
			ensure!(*val >= amount, Error::<T>::BalanceLow);
			ensure!(Self::vault_balance() >= amount, Error::<T>::VaultBalanceLow);

			*val = val.checked_sub(&amount).ok_or(Error::<T>::BalanceLow)?;
			let valut: T::AccountId = Self::account_id();
			T::Currency::transfer(&valut,&account,amount,AllowDeath)
		})
	}
	/// get staker key for staker's coinbase field
	pub fn coinbase_to_staker_key(accounts: Vec<T::AccountId>) -> Vec<T::Hash> {
		let keys: Vec<_> = Stakers::<T>::iter()
			.filter(|(_,val)| {
				accounts.clone().into_iter().find(|x| *x==val.coinbase ).is_some()
			})
			.map(|(x,_)| x )
			.collect();
		keys
	}
	pub fn get_key_by_coinbase(coinbase: T::AccountId) -> Result<T::Hash,DispatchError> {
		let hashs = Self::coinbase_to_staker_key(vec![coinbase]);
		if hashs.len() > 0 {
			return Ok(hashs[0].clone());
		}
		Err(Error::<T>::NoStakerKey.into())
	}
	/// get the policy info from policyID by use the other pallet storage
	pub fn get_policy_by_pallet(pid: PolicyID) -> Result<PolicyInfo<T::AccountId, T::BlockNumber>, DispatchError> {
		T::GetPolicyInfo::get_policy_info_by_pid(pid)
	}
	/// All staker members share the policy rewards.
	pub fn assigned_by_policy_reward(keys: Vec<T::AccountId>, all_amount: BalanceOf<T>) -> DispatchResult {
		let count = keys.len();
		let amount = all_amount / <BalanceOf<T>>::from(count  as u32);
		for i in 0..count {
			Rewards::<T>::mutate(keys[i].clone(),|b| -> DispatchResult {
				let new_amount = b.saturating_add(amount);
				*b = new_amount;
				Ok(())
			}).unwrap();
		}
		Ok(())
	}
	/// calc every policy reward by epoch.
	///
	/// when `useblock == 0`,it means the user was revoke the policy and stop it.
	pub fn calc_reward_in_policy(num: T::BlockNumber,pid: PolicyID) -> DispatchResult {
		ensure!(PolicyReserve::<T>::contains_key(pid), Error::<T>::NoReserve);

		match Self::get_policy_by_pallet(pid) {
			Ok(info) => {
				ensure!(num >= info.policy_start, Error::<T>::LowBlockNumber);
				ensure!(info.period > Zero::zero(), Error::<T>::InValidPeriod);
				let range: u32 = info.period.try_into().map_err(|_| Error::<T>::ConvertFailed)?;

				if let (user,reserve,last) = PolicyReserve::<T>::get(pid) {
					let mut last_assign = last;
					if last_assign == Zero::zero() {
						last_assign = info.policy_start;
					}
					if last_assign < info.policy_stop && reserve > Zero::zero() {
						let mut stop = num;
						if num > info.policy_stop {
							stop = info.policy_stop;
						}
						ensure!(stop >= last_assign, Error::<T>::LowBlockNumber);
						let useblock: u32 = (stop - last_assign).try_into().map_err(|_| Error::<T>::ConvertFailed)?;
						if useblock > 0 {
							let mut all = reserve * <BalanceOf<T>>::from(useblock) / <BalanceOf<T>>::from(range);
							if all > reserve {
								all = reserve;
							}

							Self::assigned_by_policy_reward(info.stackers.clone(),all)?;

							PolicyReserve::<T>::mutate(pid,|x|->DispatchResult {
								let new_amount = x.1.saturating_sub(all);
								x.1 = new_amount;
								x.2 = num;
								Ok(())
							});
						}
						last_assign = num;
					}
					if last_assign >= info.policy_stop || reserve == Zero::zero() {
						// the user stop the policy or Deplete all assets
						if reserve > Zero::zero() {
							Rewards::<T>::mutate(user.clone(), |val| -> DispatchResult {
								let new_amount = val.saturating_add(reserve);
								*val = new_amount;
								Ok(())
							}).unwrap();
							PolicyReserve::<T>::mutate(pid,|x|->DispatchResult {
								x.1 = Zero::zero();
								Ok(())
							}).unwrap();
						}
						return Ok(())
					}
					Ok(())
				} else {
					Err(Error::<T>::NoReserve)?
				}
			},
			Err(e) => Err(e),
		}
	}
	/// Calculate the reward for each user’s policy in every epoch. The reward is used to distribute
	/// to each staker who has processed the policy. After the user has created the policy,
	/// he can terminate the policy at any time and redeem his remaining pledge.
	///
	/// `num`: the reward assignment by the block numbers.
	pub fn reward_in_epoch(num: T::BlockNumber) -> DispatchResult {
		PolicyReserve::<T>::iter().for_each(|(k,_)|{
			Self::calc_reward_in_policy(num,k).unwrap();
		});
		Ok(())
	}

}

impl<T: Config> BasePolicy<T::AccountId,BalanceOf<T>,PolicyID> for Pallet<T> {
	/// policy owner will reserve asset(local asset) to the vault when create policy.
	fn create_policy(who: T::AccountId,amount: BalanceOf<T>,pid: PolicyID,stakers: Vec<T::AccountId>) -> DispatchResult {
		ensure!(!PolicyReserve::<T>::contains_key(pid), Error::<T>::RepeatReserve);
		// stakers.iter().for_each(|s| ensure!(!Self::valid_staker(s),Error::<T>::InvalidStaker));
		for s in stakers {
			if !Self::valid_staker(s) {
				return Err(Error::<T>::InvalidStaker.into());
			}
			// ensure!(Self::valid_staker(s),Error::<T>::InvalidStaker);
		}

		PolicyReserve::<T>::mutate(pid, |val| -> DispatchResult {
			*val = (who.clone(),amount,Zero::zero());
			let valut: T::AccountId = Self::account_id();
			let from = who.clone();
			T::Currency::transfer(&from,&valut,amount,AllowDeath)
		})
	}
}

//! Various basic types for use in the nuproxy pallet.

use super::*;
use frame_support::pallet_prelude::*;

use frame_support::traits::{Currency};
use sp_runtime::{traits::Convert, FixedPointNumber, FixedPointOperand, FixedU128};


pub type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen)]
pub struct StakeInfo<AccountId,Balance> {
    pub(super) coinbase: AccountId,
    pub(super) workbase: [u8;32],
    pub(super) iswork:  bool,               // no hash field
    pub(super) lockedBalance:  Balance,     // no hash field
    pub(super) workcount:   u32,
}

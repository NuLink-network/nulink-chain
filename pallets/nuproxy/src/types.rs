//! Various basic types for use in the nuproxy pallet.

use super::*;
use frame_support::pallet_prelude::*;

use frame_support::traits::{Currency};
// use frame_support::inherent::Vec;
// use sp_runtime::{traits::Convert, FixedPointNumber, FixedPointOperand, FixedU128};


pub type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct StakeInfo<AccountId,Balance> {
    pub(super) coinbase: AccountId,
    pub(super) workbase: Vec<u8>,
    pub(super) iswork:  bool,               // no hash field
    pub(super) locked_balance:  Balance,     // no hash field
    pub(super) workcount:   u32,
}

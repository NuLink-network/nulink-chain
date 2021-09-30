//! Various basic types for use in the nuproxy pallet.

use super::*;
use frame_support::pallet_prelude::*;

use frame_support::traits::{fungible, tokens::BalanceConversion, Currency};
use sp_runtime::{traits::Convert, FixedPointNumber, FixedPointOperand, FixedU128};
use pallet_policy::{PolicyID,PolicyInfo};

type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen)]
pub struct StakeInfo<Balance,AccountId> {
    pub(super) coinbase: AccountId,
    pub(super) workbase: [u8;32],
    pub(super) iswork:  bool,
    pub(super) lockedBalance:  Balance,
    pub(super) workcount:   u32,
}

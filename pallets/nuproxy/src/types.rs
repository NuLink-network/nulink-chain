//! Various basic types for use in the nuproxy pallet.

use super::*;
use frame_support::pallet_prelude::*;

use frame_support::traits::{fungible, tokens::BalanceConversion};
use sp_runtime::{traits::Convert, FixedPointNumber, FixedPointOperand, FixedU128};


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen)]
pub struct StakeInfo<Balance,AccountId> {
    pub(super) coinbase: AccountId,
    pub(super) workbase: [u8;32],
    pub(super) iswork:  bool,
    pub(super) lockedBalance:  Balance,
    pub(super) workcount:   u32,
}

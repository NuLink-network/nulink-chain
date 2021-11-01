use super::*;
use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(NuLinkProxy::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		// assert_eq!(NuLinkProxy::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(
		// 	NuLinkProxy::cause_error(Origin::signed(1)),
		// 	Error::<Test>::NoneValue
		// );
	});
}

#[test]
fn it_works_for_set_watcher() {
	new_test_ext().execute_with(|| {
		assert_ok!(NuLinkProxy::set_watcher(Origin::signed(1)));
		assert_ok!(NuLinkProxy::set_watcher(Origin::signed(2)));
		assert_ok!(NuLinkProxy::set_watcher(Origin::signed(3)));
		assert_ok!(NuLinkProxy::set_watcher(Origin::signed(4)));
		assert_noop!(NuLinkProxy::set_watcher(Origin::signed(2)),Error::<Test>::AlreadyExist);
		assert_noop!(NuLinkProxy::set_watcher(Origin::signed(4)),Error::<Test>::AlreadyExist);

		assert_eq!(NuLinkProxy::exist_watcher(1),true);
		assert_eq!(NuLinkProxy::exist_watcher(3),true);
		assert_eq!(NuLinkProxy::exist_watcher(5),false);
		assert_eq!(NuLinkProxy::exist_watcher(6),false);
	});
}

#[test]
fn it_works_for_calc_staker_hash() {
	new_test_ext().execute_with(|| {
		let staker0 = make_stake_infos(1,100,1);
		let staker1 = make_stake_infos(1,100,1);
		let staker2 = make_stake_infos(1,200,1);
		let staker3 = make_stake_infos(1,200,2);
		let staker4 = make_stake_infos(1,200,2);
		assert_eq!(NuLinkProxy::calc_staker_hash(staker0.clone()),NuLinkProxy::calc_staker_hash(staker1.clone()));
		// lock_balance has no hash field
		assert_eq!(NuLinkProxy::calc_staker_hash(staker0.clone()),NuLinkProxy::calc_staker_hash(staker2.clone()));
		assert_ne!(NuLinkProxy::calc_staker_hash(staker0.clone()),NuLinkProxy::calc_staker_hash(staker4.clone()));
		assert_ne!(NuLinkProxy::calc_staker_hash(staker0.clone()),NuLinkProxy::calc_staker_hash(staker3.clone()));
	});
}

#[test]
fn it_works_for_update_staker() {
	new_test_ext().execute_with(|| {
		let staker1 = make_stake_infos(1,100,1);
		let staker2 = make_stake_infos(2,200,2);
		let staker3 = make_stake_infos(3,300,3);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));
		assert_eq!(NuLinkProxy::get_staker_count(),3);
		assert_eq!(NuLinkProxy::get_total_staking(),600);
		assert_eq!(Stakers::<Test>::get(NuLinkProxy::calc_staker_hash(staker1.clone())).iswork,true);
		// update the stakers
		let staker4 = make_stake_infos(1,100,1);
		let staker5 = make_stake_infos(5,500,1);
		let staker6 = make_stake_infos(6,600,1);
		let stakers2 = vec![staker4.clone(),staker5.clone(),staker6.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers2));
		// the staker1 same as staker4,they equal hash.
		assert_eq!(NuLinkProxy::get_staker_count(),5);
		assert_eq!(NuLinkProxy::get_total_staking(),1200);
		// staker1 still work in the next epoch
		assert_eq!(Stakers::<Test>::get(NuLinkProxy::calc_staker_hash(staker1.clone())).iswork,true);
		assert_eq!(Stakers::<Test>::get(NuLinkProxy::calc_staker_hash(staker2.clone())).iswork,false);
		assert_eq!(Stakers::<Test>::get(NuLinkProxy::calc_staker_hash(staker3.clone())).iswork,false);
	});
}

#[test]
fn it_works_for_reward_in_epoch() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn it_works_for_mint_in_epoch() {
	new_test_ext().execute_with(|| {

	});
}
#[test]
fn it_works_for_claim_reward() {
	new_test_ext().execute_with(|| {

	});
}
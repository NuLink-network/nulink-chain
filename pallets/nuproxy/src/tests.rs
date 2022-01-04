use super::*;
use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use frame_system::RawOrigin;
use std::str::FromStr;
use sp_runtime::testing::H256;
use sp_std::convert::TryFrom;

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
fn it_work_for_init_staker() {
	new_test_ext().execute_with(|| {
		assert_eq!(NuLinkProxy::get_staker_count(),0);
		let empty_key: H256 = H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap();
		assert_eq!(Stakers::<Test>::contains_key(empty_key),false);
		// let s = Stakers::<Test>::get(empty_key);
	});
}

#[test]
fn it_works_for_set_watcher() {
	new_test_ext().execute_with(|| {
		// assert_ok!(NuLinkProxy::register_watcher(Origin::signed(1)));
		// assert_ok!(NuLinkProxy::register_watcher(Origin::signed(2)));
		// assert_ok!(NuLinkProxy::register_watcher(Origin::signed(3)));
		// assert_ok!(NuLinkProxy::register_watcher(Origin::signed(4)));
		// assert_noop!(NuLinkProxy::register_watcher(Origin::signed(2)),Error::<Test>::AlreadyExist);
		// assert_noop!(NuLinkProxy::register_watcher(Origin::signed(4)),Error::<Test>::AlreadyExist);
		//
		// assert_eq!(NuLinkProxy::exist_watcher(1),true);
		// assert_eq!(NuLinkProxy::exist_watcher(3),true);
		// assert_eq!(NuLinkProxy::exist_watcher(5),false);
		// assert_eq!(NuLinkProxy::exist_watcher(6),false);
		// only one watcher
		assert_ok!(NuLinkProxy::register_watcher(Origin::signed(1)));
		assert_noop!(NuLinkProxy::register_watcher(Origin::signed(1)),Error::<Test>::AlreadyExist);
		assert_noop!(NuLinkProxy::register_watcher(Origin::signed(2)),Error::<Test>::OnlyOneWatcher);
		assert_eq!(NuLinkProxy::exist_watcher(1),true);
		assert_eq!(NuLinkProxy::exist_watcher(2),false);
	});
}

#[test]
fn it_works_for_coinbase_to_keys() {
	new_test_ext().execute_with(|| {
		// keep the stakers
		let staker1 = make_stake_infos(1,100,1);
		let staker2 = make_stake_infos(2,200,2);
		let staker3 = make_stake_infos(3,300,3);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));
		let accounts = vec![1 as u64,2,3];
		let h = NuLinkProxy::coinbase_to_staker_key(accounts);
		assert_eq!(h.len(),3);
		println!("{:?}", h);
		let hh1 = NuLinkProxy::coinbase_to_staker_key(vec![1]);
		let h1 = NuLinkProxy::calc_staker_hash(staker1.clone());
		println!("{:?}",h1);
		assert_eq!(hh1[0],h1);

		let hh2 = NuLinkProxy::coinbase_to_staker_key(vec![2]);
		let h2 = NuLinkProxy::calc_staker_hash(staker2.clone());
		println!("{:?}",h2);
		assert_eq!(hh2[0],h2);

		let hh3 = NuLinkProxy::coinbase_to_staker_key(vec![3]);
		let h3 = NuLinkProxy::calc_staker_hash(staker3.clone());
		println!("{:?}",h3);
		assert_eq!(hh3[0],h3);
	});
}

#[test]
fn it_works_for_stakers1() {
	new_test_ext().execute_with(|| {
		// keep the stakers
		let staker1 = make_stake_infos2(1,true);
		let staker2 = make_stake_infos2(2,true);
		let staker3 = make_stake_infos2(3,false);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));
		assert_eq!(NuLinkProxy::valid_staker(1),true);
		assert_eq!(NuLinkProxy::valid_staker(2),true);
		assert_eq!(NuLinkProxy::valid_staker(3),false);
		assert_eq!(NuLinkProxy::valid_staker(4),false);
	});
}

#[test]
fn it_works_for_stakers2() {
	new_test_ext().execute_with(|| {
		// keep the stakers
		let mut staker1 = make_stake_infos2(1,true);
		let mut staker2 = make_stake_infos2(1,true);
		let mut staker3 = make_stake_infos2(2,true);
		let mut staker4 = make_stake_infos2(1,false);
		let stakers0 = vec![staker1.clone(),staker2.clone()];
		assert_noop!(NuLinkProxy::update_stakers(stakers0),Error::<Test>::RepeatStakerCoinBase);
		// set the staker1.iswork was false, it will not works.
		let mut staker1_1 = staker1.clone();
		staker1_1.iswork = false;
		let stakers1 = vec![staker1.clone(),staker1_1.clone()];
		assert_noop!(NuLinkProxy::update_stakers(stakers1),Error::<Test>::RepeatStakerCoinBase);
		// make iswork=false for the staker which account=1
		let stakers2 = vec![staker1.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers2));
		let stakers3 = vec![staker4.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers3));
	});
}

#[test]
fn it_works_for_stakers_and_policy() {
	new_test_ext().execute_with(|| {
		// keep the stakers
		let staker1 = make_stake_infos2(1,true);
		let staker2 = make_stake_infos2(2,true);
		let staker3 = make_stake_infos2(3,false);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));

		assert_ok!(NulinkPolicy::base_create_policy(OWNER.clone(),111,10,10,vec![1,2]));
		assert_ok!(NulinkPolicy::base_create_policy(OWNER.clone(),222,10,10,vec![1,2,2]));
		assert_noop!(NuLinkProxy::create_policy(OWNER.clone(),333,10,vec![1,2,3]),Error::<Test>::InvalidStaker);
		assert_noop!(NuLinkProxy::create_policy(OWNER.clone(),444,10,vec![1,4]),Error::<Test>::InvalidStaker);
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
fn it_works_for_mint_in_epoch() {
	new_test_ext().execute_with(|| {
		let staker1 = make_stake_infos(1,100,1);
		let staker2 = make_stake_infos(2,200,2);
		let staker3 = make_stake_infos(3,300,3);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(staker1.coinbase.clone()),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(staker2.coinbase.clone()),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(staker3.coinbase.clone()),0);
		assert_ok!(NuLinkProxy::mint_by_staker(100));
		let all_staking = NuLinkProxy::get_total_staking();
		let v3 =  staker3.locked_balance * 100 / all_staking;
		let v2 = staker2.locked_balance * 100 / all_staking;
		let v1 = 100 - v2 -v3;
		assert_eq!(v1,NuLinkProxy::get_staker_reward_by_coinbase(staker1.coinbase));
		assert_eq!(v2,NuLinkProxy::get_staker_reward_by_coinbase(staker2.coinbase));
		assert_eq!(v3,NuLinkProxy::get_staker_reward_by_coinbase(staker3.coinbase));
		// mint again
		assert_ok!(NuLinkProxy::mint_by_staker(200));
		let vv3 = staker3.locked_balance * 200 / all_staking;
		let vv2 = staker2.locked_balance * 200 / all_staking;
		let vv1 = 200 - vv2 - vv3;
		assert_eq!(v1+vv1,NuLinkProxy::get_staker_reward_by_coinbase(staker1.coinbase));
		assert_eq!(v2+vv2,NuLinkProxy::get_staker_reward_by_coinbase(staker2.coinbase));
		assert_eq!(v3+vv3,NuLinkProxy::get_staker_reward_by_coinbase(staker3.coinbase));
	});
}
#[test]
fn it_works_for_assigned_by_policy_reward() {
	new_test_ext().execute_with(|| {
		let all_amount:u64 = 100;
		let ids = vec![1 as u64,2,3];
		assert_ok!(NuLinkProxy::assigned_by_policy_reward(ids.clone(),all_amount));
		let unit = all_amount / ids.len() as u64;
		assert_eq!(unit,NuLinkProxy::get_staker_reward_by_coinbase(1));
		assert_eq!(unit,NuLinkProxy::get_staker_reward_by_coinbase(2));
		assert_eq!(unit,NuLinkProxy::get_staker_reward_by_coinbase(3));
	});
}

#[test]
fn it_works_for_reward_by_user_policy() {
	new_test_ext().execute_with(|| {
		let staker1 = make_stake_infos(1,100,1);
		let staker2 = make_stake_infos(2,200,2);
		let staker3 = make_stake_infos(3,300,3);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));
		frame_system::Pallet::<Test>::set_block_number(10);
		// create the policy by owner
		let value = 100;
		let policyid = 1111;
		let stakers0 = vec![1,2];
		// check the owner asset
		assert_eq!(Balances::free_balance(OWNER),1000);
		create_policy(OWNER.clone(),value,50,policyid,stakers0.clone());
		assert_eq!(PolicyReserve::<Test>::contains_key(policyid),true);
		assert_eq!(Balances::free_balance(OWNER),1000-value);
		// set the epoch
		let epoch = 20;
		frame_system::Pallet::<Test>::set_block_number(epoch);
		let num = frame_system::Pallet::<Test>::block_number();
		// check the staker balance
		assert_eq!(Balances::free_balance(1),0);
		assert_eq!(Balances::free_balance(2),0);
		assert_eq!(Balances::free_balance(3),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(1),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(2),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(3),0);

		assert_ok!(NuLinkProxy::reward_in_epoch(num));
		// check the vault
		assert_eq!(NuLinkProxy::vault_balance(),value);
		let alluse = value * (num-11) / 50;
		let unit = alluse / 2;
		// check the reward of the staker
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(1),unit);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(2),unit);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(3),0);
		// now the staker can claim their reward
	});
}

#[test]
fn it_works_for_set_get_from_vault() {
	new_test_ext().execute_with(|| {
		// transfer balance to vault
		let valut = NuLinkProxy::account_id();
		assert_ok!(Balances::transfer(RawOrigin::Signed(A).into(),valut,10));
		assert_eq!(Balances::free_balance(A),90);
		assert_eq!(Balances::free_balance(&valut),10);
		assert_ok!(Balances::transfer(RawOrigin::Signed(B).into(),valut,200));
		assert_eq!(Balances::free_balance(B),800);
		assert_eq!(Balances::free_balance(&valut),210);
		// use from the vault
		assert_ok!(Balances::transfer(RawOrigin::Signed(valut).into(),1,20));
		assert_eq!(Balances::free_balance(1),20);
		assert_eq!(Balances::free_balance(&valut),190);
	});
}
#[test]
fn it_works_for_claim_reward() {
	new_test_ext().execute_with(|| {
		let staker1 = make_stake_infos(1,100,1);
		let staker2 = make_stake_infos(2,200,2);
		let staker3 = make_stake_infos(3,300,3);
		let stakers1 = vec![staker1.clone(),staker2.clone(),staker3.clone()];
		assert_ok!(NuLinkProxy::update_stakers(stakers1));
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(staker1.coinbase.clone()),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(staker2.coinbase.clone()),0);
		assert_eq!(NuLinkProxy::get_staker_reward_by_coinbase(staker3.coinbase.clone()),0);
		assert_ok!(NuLinkProxy::mint_by_staker(100));
		let all_staking = NuLinkProxy::get_total_staking();
		let v3 =  staker3.locked_balance * 100 / all_staking;
		let v2 = staker2.locked_balance * 100 / all_staking;
		let v1 = 100 - v2 -v3;
		assert_eq!(v1,NuLinkProxy::get_staker_reward_by_coinbase(staker1.coinbase));
		assert_eq!(v2,NuLinkProxy::get_staker_reward_by_coinbase(staker2.coinbase));
		assert_eq!(v3,NuLinkProxy::get_staker_reward_by_coinbase(staker3.coinbase));
		let valut = NuLinkProxy::account_id();

		// now the staker can claim the reward
		// there are not enough assets to withdraw
		assert_ok!(Balances::transfer(RawOrigin::Signed(B).into(),valut,0));
		assert_noop!(NuLinkProxy::claim_reward_by_user(RawOrigin::Signed(staker1.coinbase).into(),v1),Error::<Test>::VaultBalanceLow);
		assert_noop!(NuLinkProxy::claim_reward_by_user(RawOrigin::Signed(staker2.coinbase).into(),v2),Error::<Test>::VaultBalanceLow);
		assert_noop!(NuLinkProxy::claim_reward_by_user(RawOrigin::Signed(staker3.coinbase).into(),v3),Error::<Test>::VaultBalanceLow);

		// there are enough assets to withdraw
		assert_ok!(Balances::transfer(RawOrigin::Signed(B).into(),valut,200));
		assert_ok!(NuLinkProxy::claim_reward_by_user(RawOrigin::Signed(staker1.coinbase).into(),v1));
		assert_ok!(NuLinkProxy::claim_reward_by_user(RawOrigin::Signed(staker2.coinbase).into(),v2));
		assert_ok!(NuLinkProxy::claim_reward_by_user(RawOrigin::Signed(staker3.coinbase).into(),v3));

		assert_eq!(v1,Balances::free_balance(staker1.coinbase));
		assert_eq!(v2,Balances::free_balance(staker2.coinbase));
		assert_eq!(v3,Balances::free_balance(staker3.coinbase));
	});
}
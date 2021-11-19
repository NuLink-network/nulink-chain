use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		// assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// // Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(
		// 	TemplateModule::cause_error(Origin::signed(1)),
		// 	Error::<Test>::NoneValue
		// );
	});
}
fn it_works_for_create_policy1() {
	new_test_ext().execute_with(|| {
		// create policy
		let mut stakers = vec![0];
		for i in 1..100 as u64 {
			let pid = (i + 1000) as u128;
			assert_ok!(NulinkPolicy::base_create_policy(i,pid,300,50,stakers.clone()));
			let p_info0 = NulinkPolicy::get_policy_info_by_pid(pid).unwrap();
			assert_eq!(p_info0.pID,pid);
			assert_eq!(p_info0.policyOwner,i);
			assert_eq!(p_info0.stackers,stakers.clone());
			stakers.push(i);
		}
	});
}

#[test]
fn it_works_for_create_policy2() {
	new_test_ext().execute_with(|| {
		// create policy
		let stakers = vec![2,3,4,5];
		assert_ok!(NulinkPolicy::base_create_policy(1,100,100,20,stakers.clone()));
		assert_noop!(NulinkPolicy::get_policy_info_by_pid(999),Error::<Test>::NotFoundPolicyID);
		let p_info0 = NulinkPolicy::get_policy_info_by_pid(100).unwrap();
		assert_eq!(p_info0.pID,100);
		assert_ne!(p_info0.pID,199);
		assert_eq!(p_info0.policyOwner,1);
		assert_eq!(p_info0.period,20);
		assert_eq!(p_info0.stackers,stakers);
		assert_noop!(NulinkPolicy::base_create_policy(1,100,100,20,stakers.clone()),Error::<Test>::RepeatPolicyID);
	});
}

#[test]
fn it_works_for_revoke_policy() {
	new_test_ext().execute_with(|| {
		// create policy and revoke it
		let stakers = vec![2,3,4,5];
		let pid = 100;
		assert_ok!(NulinkPolicy::base_create_policy(1,pid,100,20,stakers.clone()));
		let p_info0 = NulinkPolicy::get_policy_info_by_pid(100).unwrap();
		assert_eq!(p_info0.pID,100);
		assert_eq!(p_info0.policyOwner,1);
		assert_eq!(p_info0.period,20);
		assert_eq!(p_info0.stackers,stakers);
		// the policyStop was 21 block number
		assert_eq!(p_info0.policyStop,21);
		// the was no policy id
		assert_noop!(NulinkPolicy::base_revoke_policy(99,1),Error::<Test>::NotFoundPolicyID);
		// the revoke block number must large the policy.policyStart
		assert_noop!(NulinkPolicy::base_revoke_policy(pid,1),Error::<Test>::PolicyOverPeriod);
		frame_system::Pallet::<Test>::set_block_number(10);
		assert_ok!(NulinkPolicy::base_revoke_policy(pid,1));

		let p_info1 = NulinkPolicy::get_policy_info_by_pid(100).unwrap();
		assert_eq!(p_info1.policyStop,10);
	});
}
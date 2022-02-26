use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_create_policy1() {
	new_test_ext().execute_with(|| {
		// create policy
		let mut stakers = vec![0];
		for i in 1..100 as u64 {
			let pid = (i + 1000) as u128;
			assert_ok!(NulinkPolicy::base_create_policy(i,pid,300,50,stakers.clone()));
			let p_info0 = NulinkPolicy::get_policy_info_by_pid(pid).unwrap();
			assert_eq!(p_info0.p_id,pid);
			assert_eq!(p_info0.policy_owner,i);
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
		assert_eq!(p_info0.p_id,100);
		assert_ne!(p_info0.p_id,199);
		assert_eq!(p_info0.policy_owner,1);
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
		assert_eq!(p_info0.p_id,100);
		assert_eq!(p_info0.policy_owner,1);
		assert_eq!(p_info0.period,20);
		assert_eq!(p_info0.stackers,stakers);
		// the policy_stop was 21 block number
		assert_eq!(p_info0.policy_stop,21);
		// the was no policy id
		assert_noop!(NulinkPolicy::base_revoke_policy(99,1),Error::<Test>::NotFoundPolicyID);
		// the revoke block number must large the policy.policy_start
		assert_noop!(NulinkPolicy::base_revoke_policy(pid,1),Error::<Test>::PolicyOverPeriod);
		frame_system::Pallet::<Test>::set_block_number(10);
		// not the owner of the policy
		assert_noop!(NulinkPolicy::base_revoke_policy(pid,2),Error::<Test>::NotPolicyOwner);

		assert_ok!(NulinkPolicy::base_revoke_policy(pid,1));

		let p_info1 = NulinkPolicy::get_policy_info_by_pid(100).unwrap();
		assert_eq!(p_info1.policy_stop,10);
	});
}
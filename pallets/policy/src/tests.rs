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
#[test]
fn it_works_for_create_policy() {
	new_test_ext().execute_with(|| {
		// create policy
		let stakers = vec![2,3,4,5];
		assert_ok!(NulinkPolicy::base_create_policy(1,100,100,20,stakers));
		assert_noop!(NulinkPolicy::get_policy_info_by_pid(999),Error::<Test>::NotFoundPolicyID);
		let p_info0 = NulinkPolicy::get_policy_info_by_pid(100).unwrap();
		assert_eq!(p_info0.pID,100);
		assert_ne!(p_info0.pID,199);
		assert_eq!(p_info0.policyOwner,1);
		assert_eq!(p_info0.period,20);
	});
}
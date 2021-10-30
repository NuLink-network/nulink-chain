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
fn it_works_for_update_staker() {
	new_test_ext().execute_with(|| {

	});
}

#[test]
fn it_works_for_update_staker2() {
	new_test_ext().execute_with(|| {

	});
}
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use sp_runtime::BoundedVec;

/*
Notice how these tests use the types and configuration as per mock.rs.
*/

/*
Notice how these tests use the types and configuration as per mock.rs.
*/
#[test]
fn register_a_user() {
	new_test_ext().execute_with(|| {
		let name = b"polkadot".to_vec();
		let bio = b"A heterogeneous, sharded network.".to_vec();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Set the balance to 10 DOT - or whatever amount is needed to lock.
		assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), 1, 10));
		// Dispatch a signed extrinsic.
		assert_ok!(Connect::register(RuntimeOrigin::signed(1), name.clone(), bio));
		assert_eq!(Connect::total_registered().unwrap(), 1);

		// Check that the registered user exists
		let bounded_name: BoundedVec<u8, MaxNameLength> = name.try_into().unwrap();
		let user = Connect::names(bounded_name);
		assert!(user.is_some());
		assert_eq!(user.unwrap(), 1);

		// Check if locks are in place
		assert_eq!(Balances::locks(1).len(), 1);
		assert_eq!(Balances::locks(1).get(0).unwrap().amount, 10);

		// Assert that the correct event was deposited
		System::assert_last_event(Event::Registered { id: 1 }.into());
	});
}

#[test]
fn generate_gradient_with_correct_length() {
	let hex = Connect::generate_hex_values(H256([0; 32]));
	println!("{:?}", hex);
	assert_eq!(hex.0, [8, 48, 48]);
	assert_eq!(hex.1, [8, 48, 48]);
}

#[test]
fn balance_too_low() {
	new_test_ext().execute_with(|| {
		let name = b"polkadot".to_vec();
		let bio = b"A heterogeneous, sharded network.".to_vec();
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Set the balance to 9 DOT - too low
		assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), 1, 9));
		// Dispatch a signed extrinsic.
		assert_noop!(
			Connect::register(RuntimeOrigin::signed(1), name.clone(), bio),
			Error::<Test>::LowBalance
		);
	});
}

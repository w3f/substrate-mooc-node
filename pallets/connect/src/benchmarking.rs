//! Benchmarking setup for pallet-connect
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Connect;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_an_account() {
		let caller: T::AccountId = whitelisted_caller();

		T::Currency::make_free_balance_be(&caller, 100000000u32.into());

		let very_long_name = b"123456789".to_vec();
		let very_long_bio = b"USOsEy3cAmZudmWyUEMdlU6wVXsZeMj7Ts8rh7Laur3L1ZpvvorGOcZw17mDGtNhmxqYRnANsOxhhfauuRxJhz1PRtHKoXai0i3lT0cTFqpCGODLvRxk8MOiMmVMdoylxwXYMVMwoYuZJQStM9t8k4m9aESUQ5rcCkH408t9s4Yz3WfyvbZfF5bROFgrHug9uk4Iar7Q".to_vec();

		#[extrinsic_call]
		register(RawOrigin::Signed(caller.clone()), very_long_name, very_long_bio);
		assert!(<RegisteredUsers<T>>::get(&caller).is_some());
	}

	impl_benchmark_test_suite!(Connect, crate::mock::new_test_ext(), crate::mock::Test);
}

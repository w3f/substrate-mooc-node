#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Currency;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub mod weights;
pub use weights::*;

/// GOALS: 
/// 
/// 1. Register user who owns at least 10 DOT, ERROR if not
/// 2. Generate random hex values for each side of gradient 
/// 3. Store metadata about a user, including a name, bio, and their profile picture
/// 4. Store a like and dislike object in association to an account
/// 5. Have a list of friends for each account
/// 6. Store total amount of profiles on a network


/// NOTE: Using dev mode!
#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, ensure, traits::{ReservableCurrency, LockableCurrency}, Blake2_128Concat};
	use frame_system::{pallet_prelude::{*, OriginFor}, ensure_signed};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Using the pallet_balances exposed 'Currency' trait to fetch user balance info
		type Currency: ReservableCurrency<Self::AccountId> + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		/// Minimum amount to lock as part of being in the network
		#[pallet::constant]
		type MinimumLockableAmount: Get<BalanceOf<Self>>;
		/// Maximum amount of characters for a bio
		type MaxBioLength: Get<u32>;
		/// Maximum amount of characters for a name
		type MaxNameLength: Get<u32>;
	}

	/// User Info
	#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone)]
	#[scale_info(skip_type_params(T))]
	pub struct UserMetadata<T: Config> {
		pub name: BoundedVec<u8, T::MaxNameLength>,
		pub bio: BoundedVec<u8, T::MaxBioLength>,
		pub profile_gradient: (u32, u32),
		pub account_id: T::AccountId
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type TotalRegistered<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type RegisteredUsers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, UserMetadata<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Registered { id: T::AccountId }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Balance does not meet the minimum required amount
		InvalidBalance,
		/// Name exceeds MaxNameLength
		NameTooLong,
		/// Bio exceeds MaxBioLength
		BioTooLong,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Registers a user to the network
		#[pallet::call_index(0)]
		pub fn register(origin: OriginFor<T>) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(Self::check_balance(signer), Error::<T>::InvalidBalance);

			Ok(())
		}
	}

	impl <T: Config> Pallet<T> {

		/// Generates hex values for a gradient profile picture
		fn generate_hex_values() -> (u32, u32) {
			// TODO: randomly generate
			let values = (00000, 00000);
			values
		}

		/// Checks that the balance is more than or equal to ten
		/// Err if not
		fn check_balance(account_id: T::AccountId) -> bool {
			// TODO: Check balance, lock that amount!
			true
		}

	}
}

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, LockIdentifier};
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

const LOCK_ID: LockIdentifier = *b"MINDEPOS";

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
	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::{LockableCurrency, ReservableCurrency, WithdrawReasons},
		Blake2_128Concat,
	};
	use frame_system::{
		ensure_signed,
		pallet_prelude::{OriginFor, *},
	};

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
		type Currency: ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		/// Minimum amount to lock as part of being in the network
		#[pallet::constant]
		type MinimumLockableAmount: Get<BalanceOf<Self>>;
		/// Maximum amount of characters for a bio
		type MaxBioLength: Get<u32>;
		/// Maximum amount of characters for a name
		type MaxNameLength: Get<u32>;
	}

	/// User Metadata - note the traits, especially the Encode/Decode which allow for SCALE encoding to occur on this type.
	#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone)]
	#[scale_info(skip_type_params(T))]
	pub struct UserMetadata<T: Config> {
		pub name: BoundedVec<u8, T::MaxNameLength>,
		pub bio: BoundedVec<u8, T::MaxBioLength>,
		pub profile_gradient: (u32, u32),
		pub account_id: T::AccountId,
	}

	/// Total amount of registered users
	#[pallet::storage]
	pub type TotalRegistered<T: Config> = StorageValue<_, u32>;

	/// List of names by hash. This is used to check if a name has already been taken or not.
	/// In this case, we don't mind it being unbounded, as this storage object is only called
	/// When a name is either added or removed.
	#[pallet::storage]
	#[pallet::unbounded]
	pub type Names<T: Config> = StorageValue<_, Vec<BoundedVec<u8, T::MaxNameLength>>>;

	/// Registered users mapped by address
	#[pallet::storage]
	pub type RegisteredUsers<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, UserMetadata<T>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Registered { id: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Balance does not meet the minimum required amount
		LowBalance,
		/// Name exceeds MaxNameLength
		NameTooLong,
		/// Bio exceeds MaxBioLength
		BioTooLong,
		/// Name already registered
		NameInUse
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Registers a user to the network
		#[pallet::call_index(0)]
		pub fn register(origin: OriginFor<T>, name: Vec<u8>, bio: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let balance = T::Currency::free_balance(&sender);

			// Before proceeding - we have to make sure the *free* balance of a user is enough to lock up!
			// Otherwise, we halt this dispatchable with an error.
			ensure!(balance < T::MinimumLockableAmount::get(), Error::<T>::LowBalance);

			// 1. Craft the user metadata out of the given parameters from `register`.
			// Keep in mind we have to cast these to `BoundedVec` using the limits we have defined
			// in our Config (hence why we must access them using our handy `T` generic operator!).
			// Notice the error handling! Other types of error handling are okay too :)

			let name_bounded: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(name.clone()).map_err(|_| Error::<T>::NameTooLong)?;
			let bio_bounded: BoundedVec<u8, T::MaxBioLength> = BoundedVec::try_from(bio).map_err(|_| Error::<T>::BioTooLong)?;

			// 2. Check if the name already exists within the hashed name storage value
			// unwrap_or_default is used, as the default state of a vec is `[]`.
			let mut names = <Names<T>>::get().unwrap_or_default();

			let is_name = names.iter().any(|n| n == &name_bounded);
			ensure!(is_name, Error::<T>::NameInUse);

			// 3. Generate our random profile picture (aka, two hex values which form a gradient)
			let random_pfp = Self::generate_hex_values();

			// 4. Construct our UserMetadata.  Ideally, we could also create an implemention to make this easier to create!
			let user_metadata: UserMetadata<T> = UserMetadata {
				name: name_bounded.clone(),
				bio: bio_bounded,
				profile_gradient: random_pfp,
				account_id: sender.clone()
			};

			// 5. Lock the minimum deposit.  This account will now have this amount locked until they 'de-register'
			T::Currency::set_lock(LOCK_ID, &sender, T::MinimumLockableAmount::get(), WithdrawReasons::RESERVE);

			// 6. Store the user and add to existing names
			<RegisteredUsers<T>>::insert(&sender, user_metadata);
			// Push a new name
			names.push(name_bounded);
			<Names<T>>::put(names);

			// 7. Emit an event
			Self::deposit_event(Event::Registered { id: sender });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Generates hex values for a gradient profile picture
		fn generate_hex_values() -> (u32, u32) {
			// TODO: randomly generate
			let values = (00000, 00000);
			values
		}
	}
}

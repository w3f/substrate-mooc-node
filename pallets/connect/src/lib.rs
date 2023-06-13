//! # Connect Pallet
//!
//! For mostly learning purposes!
//!
//! A sybil-resistant method for a metadata storage system which could be the basis for any social network.
//!
//! - \[`Config`]
//! - \[`Call`]
//! - \[`Pallet`]
//!
//! ## Overview
//!
//! This pallet aims to utilize most basic/common features of FRAME and Substrate in order to demonstrate
//! an application which takes advantage of common traits (Currency, Randomness) in a practical setting.
//!
//! This pallet could be extended to include more aspects of game theory to further prevent adversasial actions,
//! or simply as a basis for a basic social network.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `register` - Registers user metadata in association to the senders account id.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{Currency, LockIdentifier};

/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// Type which shortens the access to the Curreency trait from the Balances pallet.
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// Lock id for locking the minimum lockable amount.
const LOCK_ID: LockIdentifier = *b"LOCKEDUP";

// Hex values for the gradient - right, left.
type Gradient = (Vec<u8>, Vec<u8>);

pub mod weights;
pub use weights::*;

/// Note: Using dev mode! (#[frame_support::pallet(dev_mode)])
#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{
		ensure,
		pallet_prelude::*,
		traits::{LockableCurrency, Randomness, ReservableCurrency, WithdrawReasons},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;

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
		#[pallet::constant]
		type MaxBioLength: Get<u32>;
		/// Maximum amount of characters for a name
		#[pallet::constant]
		type MaxNameLength: Get<u32>;
		/// Randomness!
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	/// User Metadata - note the traits, especially the Encode/Decode which allow for SCALE encoding to occur on this type.
	#[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone)]
	#[scale_info(skip_type_params(T))]
	pub struct UserMetadata<T: Config> {
		/// The name of the user, bounded in total length.
		pub name: BoundedVec<u8, T::MaxNameLength>,
		/// The bio of the user, bounded in total length.
		pub bio: BoundedVec<u8, T::MaxBioLength>,
		/// The user's color hex values for a gradient profile picture, from right to left.
		pub profile_gradient: Gradient,
		/// The associated account_id of the sender at the time of registration.
		pub account_id: T::AccountId,
	}

	/// Total amount of registered users.
	#[pallet::storage]
	pub type TotalRegistered<T: Config> = StorageValue<_, u32>;

	/// A mapping that declares names that are in use by the pallet.
	#[pallet::storage]
	pub type Names<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxNameLength>,
		T::AccountId,
		OptionQuery,
	>;

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
		NameInUse,
		/// Account ID is already registered
		AccountIdAlreadyRegistered,
		/// Integer overflow
		IntegerOverflow,
	}

	/// The extrinsics, or dispatchable functions, for this pallet.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Registers a user to the network. It requires the balance of the sender to have an amount which is greater than, or equal to MinimumLockableAmount.
		/// Locks MinimumLockableAmount as part of the registration process.
		#[pallet::call_index(0)]
		pub fn register(origin: OriginFor<T>, name: Vec<u8>, bio: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let balance = T::Currency::free_balance(&sender);

			// Before proceeding - we have to make sure the *free* balance of a user is enough to lock up!
			// Otherwise, we halt this dispatchable with an error.
			ensure!(balance >= T::MinimumLockableAmount::get(), Error::<T>::LowBalance);

			// 1. Craft the user metadata out of the given parameters from `register`.
			// Keep in mind we have to cast these to `BoundedVec` using the limits we have defined
			// in our Config (hence why we must access them using our handy `T` generic operator!).
			// Notice the error handling! Other types of error handling are okay too :)

			let name_bounded: BoundedVec<u8, T::MaxNameLength> =
				BoundedVec::try_from(name.clone()).map_err(|_| Error::<T>::NameTooLong)?;
			let bio_bounded: BoundedVec<u8, T::MaxBioLength> =
				BoundedVec::try_from(bio).map_err(|_| Error::<T>::BioTooLong)?;

			// 2. Check if the name already exists or user metadata already exists
			ensure!(<Names<T>>::get(&name_bounded).is_none(), Error::<T>::NameInUse);
			ensure!(
				<RegisteredUsers<T>>::get(&sender).is_none(),
				Error::<T>::AccountIdAlreadyRegistered
			);

			// 3. Generate our random profile picture (aka, two hex values which form a gradient)
			// Usually, some increasing nonce is used as a seed. For simplicity, we use the account id as the seed.
			let (value, _) = T::Randomness::random(&sender.encode());
			let random_pfp = Self::generate_hex_values(value);

			// 4. Construct our UserMetadata.  Ideally, we could also create an implemention to make this easier to create!
			let user_metadata: UserMetadata<T> = UserMetadata {
				name: name_bounded.clone(),
				bio: bio_bounded,
				profile_gradient: random_pfp,
				account_id: sender.clone(),
			};

			// 5. Lock the minimum deposit.  This account will now have this amount locked until they 'de-register'.
			T::Currency::set_lock(
				LOCK_ID,
				&sender,
				T::MinimumLockableAmount::get(),
				WithdrawReasons::RESERVE,
			);

			// 6. Store the user, add to existing names, and update total amount of users
			<RegisteredUsers<T>>::insert(&sender, user_metadata);
			<Names<T>>::insert(&name_bounded, sender.clone());

			// Note the use of 'unwrap_or_default' - this is better than just a plain 'unwrap()'
			// The default for 'u32' is 0, meaning an 'unwrap_or(0)' could also work here!

			let total_registered = <TotalRegistered<T>>::get().unwrap_or_default();

			// The use of checked_add() ensures 'safe math' is taking place.
			// Since we never want panic within a runtime, we have to ensure all *possible* errors can be caught.

			<TotalRegistered<T>>::put(
				total_registered.checked_add(1).ok_or(Error::<T>::IntegerOverflow)?,
			);

			// 7. Emit an event to indicate a new user was added to the network
			Self::deposit_event(Event::Registered { id: sender });

			Ok(())
		}
	}

	/// Note how you can also simply utilize the Pallet struct as per normal to declare helper functions, or anything helpful in the context of the pallet.
	impl<T: Config> Pallet<T> {
		/// Generates hex values for a gradient profile picture
		fn generate_hex_values(random_value: T::Hash) -> Gradient {
			let hex = hex::encode(random_value);
			// SCALE encode each hex portion. We don't *really* need to hex-encode here,
			// you could just get the SCALE bytes themselves, but it's useful how an external crate can be used :)
			let right = hex[..2].encode();
			let left = hex[4..6].encode();
			(right, left)
		}
	}
}

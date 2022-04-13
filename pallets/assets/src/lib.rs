#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_std::prelude::*;
use sp_runtime::{traits::{AtLeast32BitUnsigned, One}, ArithmeticError};
use codec::HasCompact;
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, dispatch::DispatchResultWithPostInfo, Blake2_128Concat, Twox64Concat};
	use frame_system::{pallet_prelude::*, Origin};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///	The units in which we record balances
		type Balance: Member 
			+ Parameter
			+ AtLeast32BitUnsigned
			+ MaxEncodedLen
			+ Default
			+ Copy;
		///	The arithmetic type of asset identifier
		type AssetID: Member 
			+ Parameter 
			+ Default
			+ TypeInfo
			+ HasCompact
			+ MaxEncodedLen 
			+ Copy; 
	}

	///	Asset Id 
	#[pallet::storage]
	#[pallet::getter(fn next_id)]
	pub type NextAssetId<T: Config> = StorageValue<_, T::AssetID, ValueQuery>;

	///	Asset Object
	#[pallet::storage]
	#[pallet::getter(fn get_asset)]
	pub type TotalSupply<T: Config> = StorageMap<
		_, Twox64Concat, T::AssetID, T::Balance, ValueQuery>;
	/// The number of units of assets held by any given account
	#[pallet::storage]
	#[pallet::getter(fn get_balances)]
	pub type Balances<T: Config> = StorageMap<
		_, Blake2_128Concat, (T::AssetID, T::AccountId), T::Balance, ValueQuery>;
	/// The inherent asset in this platform
	#[pallet::storage]
	#[pallet::getter(fn get_inherent_asset)]
	pub type PlatformAsset<T: Config> = StorageValue<_, T::AssetID>;
	///	The Price of the asset
	#[pallet::storage]
	#[pallet::getter(fn price)]
	pub type Price<T: Config> = StorageMap<
		_, Twox64Concat, T::AssetID, sp_runtime::FixedI128, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Issued { asset_id: T::AssetID, account_id: T::AccountId, balance: T::Balance },
		Burned { asset_id: T::AssetID, account_id: T::AccountId, balance: T::Balance},
		Transferred { asset_id: T::AssetID, from: T::AccountId, to: T::AccountId, amount: T::Balance}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///	Mint Assets 
		/// Issue new assets in a permissioned way, if permissionless, then with a deposit is required
		#[pallet::weight(0)]
		pub fn mint(origin: OriginFor<T>) -> DispatchResultWithPostInfo { 

			Ok(().into())
		}
		///	Transfer Asset
		/// Move assets between accounts
		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>) -> DispatchResultWithPostInfo { 

			Ok(().into())
		}
		///	Burn Assets
		/// Decrease the asset balance of an account
		#[pallet::weight(0)]
		pub fn burn(origin: OriginFor<T>) -> DispatchResult { 

			Ok(())
		}
 	}
	impl<T: Config> Pallet<T> { 
		fn get_next_id() -> Result<T::AssetID, DispatchError> { 
			NextAssetId::<T>::try_mutate(|id| -> Result<T::AssetID, DispatchError> { 
				let curr_id = *id;
				*id = id.checked_add(One::one())
					.ok_or(ArithmeticError::Overflow)?;
				Ok(curr_id)
			})
		}
	}
	
}

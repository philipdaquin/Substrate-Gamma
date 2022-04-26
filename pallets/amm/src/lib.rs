#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
mod traits;


use assets::{};

#[frame_support::pallet]
pub mod pallet {
	use sp_runtime::traits::{AtLeast32Bit, AtLeast32BitUnsigned};

use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Defines the fees taken out of each trade and sent back to the AMM pool
		type LpFee: Parameter + AtLeast32BitUnsigned + Default + Copy;

	}
	type AssetIdOf<T> = <T as assets::Config>::AssetID;
	type BalanceOf<T> = <T as assets::Config>::Balance;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// [Account, Liquidity Amount, Paired Assets ]
		AddedLiquidity { account_id: T::AccountId, amount: T::Balance, asset_id: T::AssetID },
		/// Removing liquidity event
		/// [Account, Liquidity Amount, Paid Assets] 
		RemovedLiquidity { account_id: T::AccountId, amount: T::Balance, asset_id: T::AssetID}, 
		///	Asset swap event
		///	[Account, Asset A, Liquidity A, 
		/// Asset B, Liquidity B]
		SwappedAssets { account_id: T::AccountId, asset_a: T::AssetID, amount_a: T::Balance,  
			asset_b: T::AssetID, 
			amount_b: T::Balance
		},
		ReserveChanged { asset_id: T::AssetID, amount: T::Balance }

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
		#[pallet::weight(1)]
		#[frame_support::transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>, 
			pair: (T::AssetID, T::AssetID),
			target_amount: (T::Balance, T::Balance),
			minimum_amount: (T::Balance, T::Balance)
		) -> DispatchResult {

			Ok(())
		}
	}
	///	Helper Functions
	impl<T: Config> Pallet<T> { 
	}
}

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use common::{MultiAsset, Oracle};
mod model;
use model::Pool;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::{HasCompact, MaxEncodedLen};
	use frame_support::{pallet_prelude::{*, Member}, Blake2_128Concat};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::AtLeast32BitUnsigned, FixedU128};
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///	The arithmetic type of asset identifier
		type AssetID: Member + Parameter + Default + TypeInfo + AtLeast32BitUnsigned + HasCompact
			+ MaxEncodedLen + Copy;
		///	The units in which we record balances
		type Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen + Default + Copy;
		///	Price Oracle for assets
		type Oracle: Oracle<Self::AssetID, FixedU128>;
		///	MultiAsset Transfer
		type MultiAsset: MultiAsset<Self::AccountId, Self::AssetID, Self::Balance>;
	}
	
	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_pool_info)]
	pub type PoolInfo<T: Config> = StorageMap<
		_, 
		 Twox64Concat, 
		T::AssetID, 
		Pool<T>,
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Supplied Assets to a pool[asset_id]
		Supplied { asset_id: T::AssetID, supplied_amount: T::Balance, account_id: T::AccountId },
		///	Borrowed Assets from a pool[asset_id]
		Borrowed { asset_id: T::AssetID, borrowed_amoutn: T::Balance, account_id: T::AccountId},
		///	Withdrawn Assets from a pool
		Withdrawn { asset_id: T::AssetID, withdrawn_amount: T::Balance, account_id: T::AccountId },
		///	Repaid assets to a pool
		Repaid { asset_id: T::AssetID, repaid_amount: T::Balance, account_id: T::AccountId },
		// Liquidated {}
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
		#[pallet::weight(10)]
		pub fn supply_asset(origin: OriginFor<T>) -> DispatchResult { 
			let user = ensure_signed(origin)?;


			Ok(())
		}
		#[pallet::weight(10)]
		pub fn borrow_asset(origin: OriginFor<T>) -> DispatchResult { 
			let user = ensure_signed(origin)?;


			Ok(())
		}
		#[pallet::weight(10)]
		pub fn withdraw_asset(origin: OriginFor<T>) -> DispatchResult { 
			let user = ensure_signed(origin)?;


			Ok(())
		}
		#[pallet::weight(10)]
		pub fn repay_asset(origin: OriginFor<T>) -> DispatchResult { 
			let user = ensure_signed(origin)?;


			Ok(())
		}
		#[pallet::weight(10)]
		pub fn liquidate(origin: OriginFor<T>) -> DispatchResult { 
			let user = ensure_signed(origin)?;


			Ok(())
		}
	}
}

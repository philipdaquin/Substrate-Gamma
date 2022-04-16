#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use assets;
use sp_runtime::FixedU128;
use frame_support::{pallet_prelude::*, dispatch::TransactionPriority};
use frame_system::{pallet_prelude::{*, BlockNumberFor}, Origin};

// #[cfg(test)]
// mod tests;
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
// #[cfg(test)]
// mod mock;


#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///	The overacrching dispatch call type 
		type Call: From<Call<Self>>;
		///	A configuration for base priority of unsigned transaction 
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
		/// Unsigned Interval 
		#[pallet::constant]
		type UnsignedInterval: Get<Self::BlockNumber>;
	}
	pub type AssetID<T> = <T as assets::Config>::AssetID;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	pub type Price<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewPrice {asset_id: AssetID<T>, price: FixedU128 },
	}
	
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
	
		///	
		#[pallet::weight(0)]
		pub fn submit_price_unsigned(origin: OriginFor<T>,  ) -> DispatchResult { 
			ensure_none(origin);
			
			
			Ok(())
		}
		
	}
	impl<T: Config> Pallet<T> { 
		
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { 
		fn offchain_worker(blocknumber: T::BlockNumber) { 
			log::info!("👋 This is the mutha fuckin offchain worker boiiii 🚀🚀🚀🚀");
			log::debug!("📢 Current blocknumber {:?}", blocknumber);
			let current_price = ;
			log::debug!("{:?}", current_price);
		}
	}
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

} 

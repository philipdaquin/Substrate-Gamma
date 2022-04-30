#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use assets::Pallet ;
use sp_runtime::{FixedPointNumber, FixedU128, offchain::http::Error as HttpError, offchain::http::Request};
use frame_support::{pallet_prelude::*, dispatch::TransactionPriority};
use frame_system::{pallet_prelude::{*, BlockNumberFor}, Origin, 
	offchain::{Signer, SubmitTransaction, CreateSignedTransaction}
};
use sp_io::offchain;
use sp_std::vec::Vec;
use lite_json::json::JsonValue;
// #[cfg(test)]
// mod tests;
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;
// #[cfg(test)]
// mod mock;
const API_URL: &str = "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD";

#[frame_support::pallet]
pub mod pallet {
	use lite_json::Value;

use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + 
		assets::Config + CreateSignedTransaction<Call<Self>> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///	The overacrching dispatch call type 
		type Call: From<Call<Self>>;
		///	A configuration for base priority of unsigned transaction 
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
		/// Unsigned Interval 
		#[pallet::constant]
		type UnsignedInterval: Get<BlockNumberFor<Self>>;
	}
	pub type AssetID<T> = <T as assets::Config>::AssetID;
	pub type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
	///	Defines the block when the next unsigned transaction will be accepted 
	/// To prevent the spam of unsigned and unpaid transaction on the network, 
	/// we have decided to set a constant interval 'T::UnsignedInterval' blocks
	/// This storage entry defines when new transactions is going to be accepted 
	#[pallet::storage]
	#[pallet::getter(fn next_unsigned_at)]
	pub type NextUnsignedAt<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	
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
		pub fn submit_price_unsigned(
			origin: OriginFor<T>, 
			blocknumber: BlockNumberFor<T>, 
			price: u32,
		) -> DispatchResult { 
			ensure_none(origin)?;
		
			let asset_id = <T as assets::Config>::AssetID::from(4u32);
			let current_block = <frame_system::Pallet<T>>::block_number();
			let price = FixedU128::saturating_from_rational(price, 100);
			NextUnsignedAt::<T>::put(current_block + T::UnsignedInterval::get());
			assets::Pallet::<T>::set_price(asset_id, price);
			Self::deposit_event(Event::<T>::NewPrice {asset_id, price});

			Ok(())
		}
		
	}
	impl<T: Config> Pallet<T> { 
		///	Unsigned Transactions
		///	A helper function to fetch the price and send a raw unsigned transaction
		fn fetch_price_and_send_raw_unsigned(blocknumber: BlockNumberFor<T>) -> DispatchResult { 
			let next_unsigned = Self::next_unsigned_at();
			ensure!(next_unsigned > blocknumber, Error::<T>::TooEarlyToSend);

			///	Fetch the current price from an external Api
			let price = Self::fetch_prices().map_err(|_| Error::<T>::FetchError)?;
			let call = Call::submit_price_unsigned { blocknumber, price };
			
			SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
			.map_err(|()| "Unable to submit unsigned transaction.")?;
			
			Ok(())
		}
		fn fetch_prices() -> Result<u32, HttpError> { 
			let (request, deadline) = (Request::get(API_URL), 
				offchain::timestamp().add(sp_runtime::offchain::Duration::from_millis(2000)));
			let pending = request 
				.deadline(deadline)
				.send()
				.map_err(|_| HttpError::IoError)?;
			let response = pending.try_wait(deadline).map_err(|_| HttpError::DeadlineReached)??;
			if response.code != 200 { 
				log::warn!("Unexpected Code {:?}", response.code.clone());
				return Err(HttpError::Unknown);
			}
			//	JSON Bytes
			let body = response.body().collect::<Vec<_>>();
			//	Bytes to str 
			let body_str = sp_std::str::from_utf8(&body).expect("No UTF8 body");
			let price = match Self::parse_price(body_str) { 
				Some(p) => Ok(p),
				_ => { 
					log::warn!("{:?}", body_str);
					Err(HttpError::Unknown)
				}
			}?;
			log::info!("{:?}", price);
			Ok(price)

		}
		/// Parse the price from the given JSON string using lite json 
		fn parse_price(price_str: &str) -> Option<u32> { 
			let val = lite_json::parse_json(price_str)
				.ok()
				.and_then(|price| match price { 
					JsonValue::Object(p) => { 
						let mut chars = "USD".chars();
						p.into_iter()
							.find(|(k, _)| k.iter().all(|k| Some(*k) == chars.next()))
							.and_then(|v| match v.1 { 
								JsonValue::Number(num) => Some(num),
								_ => None
							})
					},
					_ => None
				})?;
				let exp = val.fraction_length.checked_sub(2).unwrap_or(0);
				Some(val.integer as u32 * 100 + (val.fraction / 10_u64.pow(exp)) as u32)
		}
		fn validate_transaction_parameters(
			blocknumber: BlockNumberFor<T>, 
			new_price: u32
		) -> TransactionValidity { 
			let next_unsigned = NextUnsignedAt::<T>::get();
			
			if next_unsigned > blocknumber { 
				return InvalidTransaction::Stale.into();
			}
			let curr_block = <frame_system::Pallet<T>>::block_number();
			if curr_block < blocknumber { 
				return InvalidTransaction::Future.into();
			}
			ValidTransaction::with_tag_prefix("OffChainworker")
				.priority(T::UnsignedPriority::get())
				.and_provides(next_unsigned)
				.longevity(5)
				.propagate(true)
				.build()
		}
	}
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> { 
		fn offchain_worker(blocknumber: BlockNumberFor<T>) { 
			log::info!("👋 This is the mutha fuckin offchain worker boiiii 🚀🚀🚀🚀");
			log::debug!("📢 Current blocknumber {:?}", blocknumber);

			let asset_id = AssetID::<T>::from(4u32);
			let current_price = assets::Price::<T>::get(asset_id);
			
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

		TooEarlyToSend,

		FetchError,
	}

} 

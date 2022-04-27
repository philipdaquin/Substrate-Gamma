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
	use frame_support::traits::Randomness;
use frame_system::WeightInfo;
use sp_runtime::traits::{AtLeast32Bit, AtLeast32BitUnsigned, AccountIdConversion, Zero};
use frame_support::PalletId;
use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Defines the fees taken out of each trade and sent back to the AMM pool
		type LpFee: Parameter + AtLeast32BitUnsigned + Default + Copy;
		///	Weight information for extrinsics 
		type SwapsWeight: WeightInfo;
		///	Pallet Id for this Pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;



	}
	type AssetIdOf<T> = <T as assets::Config>::AssetID;
	type BalanceOf<T> = <T as assets::Config>::Balance;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	///	Pool index storage
	#[pallet::storage]
	#[pallet::getter(fn pool_id)]
	pub type PoolIndex<T> = StorageValue<_, u32, ValueQuery>;

	///	Pool-to-Asset Storage 
	#[pallet::storage]
	#[pallet::getter(fn get_pool_id)]
	pub type PoolAccount<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetID, T::AccountId, OptionQuery>;

	///	Liquidy of each pair pool 
	#[pallet::storage]
	#[pallet::getter(fn get_liquidity)]
	pub type PoolLiquidity<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetID, T::Balance>;


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
		PoolIdError,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		///	origin: AccountId 
		///	pair: [base, quote]
		/// target_amount: [base, quote]
		/// minimum_amount: [base, quote]
		#[pallet::weight(1)]
		#[frame_support::transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>, 
			pair: (T::AssetID, T::AssetID),
			target_amount: (T::Balance, T::Balance),
			minimum_amount: (T::Balance, T::Balance)
		) -> DispatchResult {
			let account_id = ensure_signed(origin)?;
			let (base, quote) = pair;
			let (base_amount, quoted_amount) = target_amount;
			let (min_base, min_quote) = minimum_amount;
			
			let pool_id = Self::gen_new_exchange();
			
			let total_liquidity = Self::get_liquidity(base);
			
			if let Some(liquidity) = total_liquidity { 
				if liquidity.is_zero() { 
					assets::Pallet::<T>::transfer(account_id.clone(), from, value, asset_id)
				}
			}


			Ok(())
		}
	}
	///	Helper Functions
	impl<T: Config> Pallet<T> {
		///	Generate a new exchange address
		/// Create a new exchange account 
		fn gen_new_exchange() -> T::AccountId { 
			let new_id = Self::next_pool_id().expect("Unable To Generate New Pool ID");
			T::PalletId::get().into_sub_account(new_id)
		}
		fn next_pool_id() -> Result<u32, DispatchError> { 
			PoolIndex::<T>::try_mutate(|id| -> Result<u32, DispatchError> { 
				let current_id = *id;
				*id = id.checked_add(1).ok_or(sp_runtime::ArithmeticError::Overflow)?;
				Ok(current_id)
			})
		}

	}
}

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
mod traits;
use common::{MultiAsset, AssetBalance};
use frame_support::traits::Randomness;
use frame_system::WeightInfo;
use sp_runtime::{traits::{AtLeast32Bit, AtLeast32BitUnsigned, AccountIdConversion, Zero}, FixedU128};
use frame_support::PalletId;
use assets::{};


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
		/// Defines the fees taken out of each trade and sent back to the AMM pool
		type LpFee: Parameter + AtLeast32BitUnsigned + Default + Copy;
		///	Weight information for extrinsics 
		type SwapsWeight: WeightInfo;
		///	MultiAsset Trasnsfer
		type MultiAsset: MultiAsset<Self::AccountId, Self::AssetID, Self::Balance>;
		///	Pallet Id for this Pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		///	Standard Protocol Fee
		type Rate: Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		///	Accesses Asset Balance 
		type AssetBalance: AssetBalance<Self::AssetID, Self::AccountId, Self::Balance>;
	}
	type AssetIdOf<T> = <T as assets::Config>::AssetID;
	type BalanceOf<T> = <T as assets::Config>::Balance;

	///	The protocol fee
	#[pallet::storage]
	#[pallet::getter(fn get_fee)]
	pub type ProtocolFee<T: Config> = StorageValue<_, T::Rate, ValueQuery, ()>;


	///	Auxiliarry Storage used to track pool ids
	#[pallet::storage]
	#[pallet::getter(fn pool_id)]
	pub type PoolIndex<T> = StorageValue<_, u32, ValueQuery>;

	///	Accounts of Pools
	#[pallet::storage]
	#[pallet::getter(fn get_pools)]
	pub type Pools<T: Config> = StorageMap<
		_, Blake2_128Concat, 
		T::AssetID, 
		T::AccountId, 
		OptionQuery
	>;
	///	Liquidy of each pair pool 
	/// A bag of liquidity composed by two different assets
	#[pallet::storage]
	#[pallet::getter(fn get_total_liquidity)]
	pub type TotalLiquidity<T: Config> = StorageMap<
		_, 
		Blake2_128Concat, 
		T::AssetID, 
		T::Balance,
	>;

	/// The Liquidity of each AssetID - AccountID	
	#[pallet::storage]
	#[pallet::getter(fn get_lp)]
	pub type PoolLiquidity<T: Config> = StorageDoubleMap<
		_, 
		Blake2_128Concat, T::AssetID, 
		Blake2_128Concat, T::AccountId, 
		T::Balance, 
		OptionQuery
	>;

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
		TransferToFailed,
		PoolNotFound
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
		) -> DispatchResult {
			//	The account to inject liquidity to paired pool 
			let account_id = ensure_signed(origin)?;
			let (base_id, pair_id) = pair;
			let (base_amount, pair_amount) = target_amount;

			//	Create a new pool account 
			let pool_id = Self::gen_new_exchange();
			//	Get the total liquidity of a pool 
			let total_liquidity = Self::get_total_liquidity(base_id);
			
			if let Some(liquidity) = total_liquidity { 
				if liquidity.is_zero() { 
					T::MultiAsset::transfer(account_id.clone(), 
						pool_id.clone(),
						base_id, 
						base_amount).map_err(|_| Error::<T>::TransferToFailed)?;

					T::MultiAsset::transfer(account_id.clone(),
						pool_id.clone(),
						pair_id,
						pair_amount).map_err(|_| Error::<T>::TransferToFailed)?;
					//	Set liquidity of an account in a pool
					Self::insert_liquidity(pair_id.clone(), account_id.clone(), base_amount.clone());
					//	Update the total liquidity of 'pair_id' pool Account 
					Self::update_pool_liquidity(pair_id.clone(), base_amount.clone(), true);
				}  else { 
					let total_base_amount = T::AssetBalance::balance(base_id, pool_id.clone());
					
					//	Transfer base_asset with base amount to the pool exchange account
					T::MultiAsset::transfer(account_id.clone(), 
						pool_id.clone(),
						base_id.clone(),
						base_amount.clone()).map_err(|_| Error::<T>::TransferToFailed)?;
	
					// Transfer the paired asset with pair_amount to the pool exchange account 
					T::MultiAsset::transfer(account_id.clone(), 
						pool_id.clone(), 
						pair_id.clone(),
						pair_amount.clone()).map_err(|_| Error::<T>::TransferToFailed)?;
	
					let minted_lp = liquidity * pair_amount / total_base_amount;
					let pool_liquidity = PoolLiquidity::<T>::get(pair_id.clone(), account_id.clone())
						.expect("Unable to get pool liquidity");
					Self::insert_liquidity(pair_id.clone(), account_id.clone(), pool_liquidity + minted_lp);
					Self::update_pool_liquidity(pair_id.clone(), minted_lp.clone(), true);
				}
				// Update the key pair 
				Pools::<T>::insert(pair_id.clone(), pool_id.clone());
			} 
			//	Get the new paired asset balance
			let total_pair_amount = T::AssetBalance::balance(pair_id, pool_id.clone());
			Self::deposit_event(Event::<T>::ReserveChanged {
				asset_id: pair_id.clone(),
				amount: total_pair_amount.clone()
			});
			//	Get the new updated base balance
			let base_id_amount = T::AssetBalance::balance(base_id.clone(), pool_id.clone());
			Self::deposit_event(Event::<T>::ReserveChanged{ 
				asset_id: base_id.clone(),
				amount: base_id_amount.clone()
			});

			Self::deposit_event(Event::<T>::AddedLiquidity { 
				account_id: account_id, 
				amount: base_amount.clone(), 
				asset_id: pair_id.clone() });

			Ok(())
		}
		///	origin: User 
		/// pair: [base, paired asset]
		/// target_amount : [target_base, target_paired]
		/// min_amount: [min_base, min_paired]
		#[pallet::weight(1)]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pair: (T::AssetID, T::AssetID),
			target_amount: (T::Balance, T::Balance),
			min_amount: (T::Balance, T::Balance)
		) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;
			let (base_id, pair_id) = pair;
			let (base_amount, pair_amount) = target_amount;

			

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
		fn insert_liquidity(asset_id: T::AssetID, account_id: T::AccountId, amount: T::Balance) { 
			PoolLiquidity::<T>::insert(asset_id.clone(), account_id, amount);
			log::info!("Inserting Liquidity into {:?}", asset_id);
		} 
		fn update_pool_liquidity(asset_id: T::AssetID, amount: T::Balance, add_on: bool) { 
			log::info!("Updating pool liquidity");
			let mut total_liquidity = Self::get_total_liquidity(asset_id)
				.expect("");
			if add_on { 
				total_liquidity += amount;
			} else { 
				total_liquidity -= amount;
			}
			TotalLiquidity::<T>::insert(asset_id, total_liquidity);
		}
	}
}

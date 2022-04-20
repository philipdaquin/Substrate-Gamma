#![cfg_attr(not(feature = "std"), no_std)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use common::{MultiAsset, Oracle};
mod model;
pub use model::*;
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use sp_runtime::{FixedU128, FixedI128,FixedPointOperand, traits::{One, Zero}};
use codec::{HasCompact, MaxEncodedLen};
use frame_support::{pallet_prelude::{*, Member, ValueQuery}, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::AtLeast32BitUnsigned};
use scale_info::TypeInfo;
use sp_runtime::FixedPointNumber;


#[frame_support::pallet]
pub mod pallet {

use super::*;
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///	The units in which we record balances
		type Balance: Member + Parameter + AtLeast32BitUnsigned + FixedPointOperand + Default + Copy + MaxEncodedLen + TypeInfo;
		///	The arithmetic type of asset identifier
		type AssetID: Parameter + Default + AtLeast32BitUnsigned + Copy + MaxEncodedLen + TypeInfo;
		///	Price Oracle for assets
		type Oracle: Oracle<Self::AssetID, FixedU128>;
		///	MultiAsset Transfer
		type MultiAsset: MultiAsset<Self::AccountId, Self::AssetID, Self::Balance>;
	}
	type AssetIdOf<T> = <T as Config>::AssetID;
	type BalanceOf<T> = <T as Config>::Balance;
	pub type Pools<T> = Pool<AssetIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::BlockNumber>;

	//	OnChain Database 
	//	AssetID uses Twox64Concat to ensure flexibility on Security and Efficient Queries					
	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_pool)]
	pub type PoolInfo<T: Config> = StorageMap<
		_, 
		Twox64Concat, 
		T::AssetID, 
		Pools<T>,
		OptionQuery,
	>;
	
	//	User debt
	#[pallet::storage]
	#[pallet::getter(fn get_user_debt)]
	pub type UserDebtInfo<T: Config> = StorageDoubleMap<
		_, 
		//	[AssetId, AccountId, UserDebt]
		Twox64Concat, T::AssetID, 
		Blake2_128Concat, T::AccountId, 
		Option<UserDebt<T::AssetID, T::Balance>>, 
		OptionQuery
	>;

	//	User Supply 
	#[pallet::storage]
	#[pallet::getter(fn get_user_asset)]
	pub type UserAssetInfo<T: Config> = StorageDoubleMap<
		_, 
		//	[AssetId, AccountId, UserAsset]
		Twox64Concat, T::AssetID,
		Blake2_128Concat, T::AccountId, 
		Option<UserAssets<T::AssetID, T::Balance>>,
		OptionQuery
	>;

	#[pallet::type_value]
	pub fn LiquidationThreshold<T: Config>() -> FixedU128 { FixedU128::one()}

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
		///	OnChain Does not Exist
		DbPoolNotExist,
		UnableIntoU32
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10)]
		pub fn supply_asset(origin: OriginFor<T>, asset_id: T::AssetID, amount: T::Balance) -> DispatchResult { 
			let user = ensure_signed(origin)?;
			//	Verify OnChain Database
			let mut pool = PoolInfo::<T>::get(asset_id).ok_or(Error::<T>::DbPoolNotExist)?;
			// 


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
	impl<T: Config> Pallet<T> { 
		///	'Accrue Interest' is the interest on an Asset that has accumulated since the principle investment
		fn accrue_interest(pool: &mut Pools<T>) { 
			log::info!("📢 Accruiing User Interst Rate");
			let now = frame_system::Pallet::<T>::block_number();
			
			//	Verify if the pool interst has been updated
			if pool.last_updated == now { 
				return 					
			}
			//	Get the time difference from 'now' - 'last_updated'
			let timespan = now - pool.last_updated;
			//	Convert 'BlockNumber' into u32
			let elapsed_time_in_u32 = TryInto::<u32>::try_into(timespan).map_err(|_| Error::<T>::UnableIntoU32);
			// Get the Supply Rate and then calculate the Supply Interest 
			
		}  
		fn supply_rate_internal(pool: &Pools<T>) -> FixedU128 { 
			//	Check asset supply in the Pool
			if pool.total_supply == T::Balance::zero() { 
				return pool.initial_interest_rate
			}
			let utilization_ratio = FixedU128::saturating_from_rational(pool.total_debt, pool.total_supply);
			Self::debt_rate_interal(pool) * utilization_ratio
		}
		fn debt_rate_internal(pool: &Pools<T>) -> FixedU128 { 
			if pool.total_supply == T::Balance::zero() { 
				return pool.initial_interest_rate
			}
			let utilization_ratio = FixedU128::saturating_from_rational(pool.total_debt, pool.total_supply);
			pool.initial_interest_rate + pool.utilization_factor * utilization_ratio		
		}
	}
}

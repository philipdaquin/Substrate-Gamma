#![cfg_attr(not(feature = "std"), no_std)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
use common::{MultiAsset, Oracle};
mod model;
mod impl_function;
pub use model::*;
use codec::{Decode, Encode};
use frame_support::{RuntimeDebug, PalletId};
use sp_runtime::{FixedU128, FixedI128,FixedPointOperand, traits::{One, Zero, AccountIdConversion}};
use codec::{HasCompact, MaxEncodedLen};
use frame_support::{pallet_prelude::{*, Member, ValueQuery}, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::AtLeast32BitUnsigned};
use scale_info::TypeInfo;
use sp_runtime::FixedPointNumber;
use sp_std::{prelude::*, vec, convert::TryInto};

const PALLET_ID: PalletId = PalletId(*b"Lending2");

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
		type AssetID: Parameter + Default + AtLeast32BitUnsigned + Copy + MaxEncodedLen + TypeInfo + HasCompact;
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
	#[pallet::getter(fn get_user_supply)]
	pub type UserAssetInfo<T: Config> = StorageDoubleMap<
		_, 
		//	[AssetId, AccountId, UserAsset]
		Twox64Concat, T::AssetID,
		Blake2_128Concat, T::AccountId, 
		UserAssets<T::AssetID, T::Balance>,
		OptionQuery
	>;


	//	The set of User's Supply 
	#[pallet::storage]
	#[pallet::getter(fn user_assets)]
	pub(super) type UserAssetSet<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		Vec<T::AssetID>,
		ValueQuery,
	>; 
	//	Set of User's debt
	#[pallet::storage]
	#[pallet::getter(fn user_debts)]
	pub type UserDebtSet<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId, 
		Vec<T::AssetID>,
		ValueQuery
	>;

	#[pallet::type_value]
	pub fn LiquidationThreshold<T: Config>() -> FixedU128 { FixedU128::one()}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Supplied Assets to a pool[asset_id]
		Supplied { asset_id: T::AssetID, amount: T::Balance, account_id: T::AccountId },
		///	Borrowed Assets from a pool[asset_id]
		Borrowed { asset_id: T::AssetID, amount: T::Balance, account_id: T::AccountId},
		///	Withdrawn Assets from a pool
		Withdrawn { asset_id: T::AssetID, withdrawn_amount: T::Balance, account_id: T::AccountId },
		///	Repaid assets to a pool
		Repaid { asset_id: T::AssetID, repaid_amount: T::Balance, account_id: T::AccountId },
		// Liquidated {}
		Liquidated {
			payment_asset: T::AssetID, 
			seized_asset: T::AssetID, 
			arbitrager: T::AccountId, 
			target: T::AccountId,
			price: T::Balance,
			amount_seized: T::Balance
		}
	}
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		///	OnChain Does not Exist
		DbPoolNotExist,
		UnableIntoU32,
		TransferIntoFailed,
		InsufficientSupply,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10)]
		pub fn supply_asset(origin: OriginFor<T>, asset_id: T::AssetID, amount: T::Balance) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;
			//	Verify OnChain Database
			let mut pool = PoolInfo::<T>::get(asset_id).ok_or(Error::<T>::DbPoolNotExist)?;
			//	Accrue Pool Interest
			Self::accrue_interest(&mut pool);
			//	Transfer User Asset to the OnChain Account
			T::MultiAsset::transfer(
				account_id.clone(),
				Self::fund_account_id(),
				asset_id, 
				amount 
			).map_err(|_| Error::<T>::TransferIntoFailed)?;
			//	Update the User Supply Interest 
			Self::update_user_supply_interest(account_id.clone(), asset_id, &pool, amount, true);

			//	Update the Pool Supply interest 
			Self::update_pool_supply(&mut pool, amount.clone(), true);
			Self::deposit_event(Event::<T>::Supplied { 
				asset_id, 
				amount,
				account_id 
			});
			//	Insert the assets to the User Records
			let mut user_assets = Self::user_assets(account_id);
			user_assets.push(asset_id.clone());
			UserAssetSet::<T>::insert(account_id, user_assets);
			//	Update the pool 
			PoolInfo::<T>::insert(asset_id, pool);
			
			Ok(())
		}
		#[pallet::weight(10)]
		pub fn borrow_asset(origin: OriginFor<T>) -> DispatchResult { 
			let user = ensure_signed(origin)?;


			Ok(())
		}
		#[pallet::weight(10)]
		pub fn withdraw_asset(origin: OriginFor<T>, asset_id: T::AssetID, amount: T::Balance ) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;
			//	Check the pool 
			let mut pool = Self::get_pool(asset_id).ok_or(Error::<T>::DbPoolNotExist)?;
			//	Ensure the amount is no greater than the actual witholding assets of the user
			//	else set the amount to the left over supplied amount 
			if let Some(user_assets) = UserAssetInfo::<T>::get(asset_id, account_id) { 
				ensure!(user_assets.supplied_amount >= amount, Error::<T>::InsufficientSupply);
				amount = user_assets.supplied_amount;
			}
			//	Accrue pool interest 
			Self::accrue_interest(&mut pool);
			//	Accrue the user's interest 
			Self::accrue_user_supply(&mut pool, asset_id, account_id, amount);
			
			//	Check Users Collateral, ensure that it would not trigger liquidation process would not be triggered 

			
			//	Get current Price 
			let price = T::Oracle::get_rate(asset_id);
			


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
		///	'Into_Account' converts 'PALLET_ID' into a OnChain Account 
		fn fund_account_id() -> T::AccountId { 
			PALLET_ID.into_account()
		}
		fn block_to_int(block: T::BlockNumber) -> Result<u32, DispatchError> { 
			let into_int: u32 = TryInto::<u32>::try_into(block).ok().expect("");
			Ok(into_int)
		}
		///	'Accrue Interest' is the interest on an Asset that has accumulated since the principle investment
		fn accrue_interest(pool: &mut Pools<T>) { 
			log::info!("📢 Accruing User Interst Rate");
			let now = frame_system::Pallet::<T>::block_number();
			
			//	Verify if the pool interest has been updated
			if pool.last_updated == now { 
				return 					
			}
			//	Get the time difference from 'now' - 'last_updated'
			let timespan = now - pool.last_updated;
			//	Convert 'BlockNumber' into u32
			let elapsed_time_in_u32 = Self::block_to_int(timespan).unwrap();

			// Get the Supply Rate and then calculate the Supply Interest 
			let supply_multiplier = Self::supply_rate_interest(pool) 
				+ FixedU128::one() 
				* FixedU128::saturating_from_integer(elapsed_time_in_u32); 
			let debt_multiplier = Self::borrowing_rate_interest(pool)
				+ FixedU128::one()
				* FixedU128::saturating_from_integer(elapsed_time_in_u32);
			
			pool.total_supply = supply_multiplier.saturating_mul_int(pool.total_debt);
			pool.total_supply_index = pool.total_supply_index * supply_multiplier;

			pool.total_debt = debt_multiplier.saturating_mul_int(pool.total_debt);
			pool.total_supply_index = pool.total_debt_index * debt_multiplier;

			pool.last_updated = now;
			log::info!("Accrued Interest Rate");
		}  
		/// Supply Interest Rate 
		pub(crate) fn supply_rate_interest(pool: &Pools<T>) -> FixedU128 { 
			//	Check asset supply in the Pool
			if pool.total_supply == T::Balance::zero() { 
				return FixedU128::zero();
			}
			
			//	Utilisation Rate = total debt/ total assets
			let utilization_ratio = FixedU128::saturating_from_rational(pool.total_debt, pool.total_supply);
			Self::borrowing_rate_interest(pool) * utilization_ratio
		}
		///	Borrowing Interest Rate
		pub(crate) fn borrowing_rate_interest(pool: &Pools<T>) -> FixedU128 { 
			if pool.total_supply == T::Balance::zero() { 
				return pool.initial_interest_rate
			}
			let utilization_ratio = FixedU128::saturating_from_rational(pool.total_debt, pool.total_supply);
			pool.initial_interest_rate + pool.utilization_factor * utilization_ratio		
		}

		fn update_user_supply_interest(
			account_id: T::AccountId, 
			asset_id: T::AssetID, 
			pool: &Pools<T>,
			amount: T::Balance, 
			add_on: bool 
		) { 
			if let Some(mut user_assets) = Self::get_user_supply(asset_id, account_id.clone()) { 
				//	Calculate the ratio 'total_supply_index' is to 'user_assets.index'
				let pool_to_user_ratio = pool.total_supply_index / user_assets.index;
				//	Update the user's supplied amount: pool_to_user ratio * intitial user_assets ratio
				user_assets.supplied_amount = pool_to_user_ratio.saturating_mul_int(user_assets.supplied_amount);
				//	Update the User's Index to the Current Pool's supply Index 
				user_assets.index = pool.total_supply_index;
				
				//	Recalculate the user_supplied assets on chain 
				if add_on { 
					user_assets.supplied_amount += amount
				} else { 
					user_assets.supplied_amount -= amount 
				}
				//	For security purposes, the system would only access on chain storage if the 
				//	the supplied amount is not NULL
				//	Update the new supplied_amount on chain 
				if user_assets.supplied_amount != T::Balance::zero() { 
					UserAssetInfo::<T>::insert(asset_id, account_id.clone(), user_assets);
					log::info!("Updating On-Chain User-to-Pool Ownership");
				} else { 
					UserAssetInfo::<T>::remove(asset_id, account_id.clone());
					// Update the User Supply Set 
					let mut assets = UserAssetSet::<T>::get(account_id);
					//	Remove the asset equal to Zero 
					assets.retain(|n| *n != asset_id);
					UserAssetSet::<T>::insert(account_id, assets);
				} 
			} else if amount != T::Balance::zero() { 
				log::info!("Updating User's Index");
				let asset_set = UserAssets::<T::AssetID, T::Balance> { 
					asset_id, 
					supplied_amount: amount.clone(), 
					index: pool.total_supply_index 
				};
				//	Update the user's index unique to the pool 
				UserAssetInfo::<T>::insert(
					asset_id, 
					account_id, 
					asset_set	
				);
			}		
		}
		///	Helper Functions to recalculate the supply and debt
		fn update_pool_supply(pool: &mut Pools<T>, amount: T::Balance, add_on: bool) { 
			log::info!("Recalculating pool supply of assets");
			if add_on  { 
				pool.total_supply += amount;
			} else { 
				pool.total_supply -= amount;			
			}
		}
		fn update_pool_debt(pool: &mut Pools<T>, debt: T::Balance, add_on: bool) { 
			log::info!("Recalculating pool supply of debt");
			
			if add_on { 
				pool.total_debt += debt
			} else { 
				pool.total_debt -= debt
			}
		}
		fn accrue_user_supply(pool: &mut Pools<T>, asset_id: T::AssetID, account_id: T::AccountId, amount: T::Balance) { 
			if let Some(mut user_assets) = Self::get_user_supply(asset_id, account_id.clone()) { 
				//	Calculate the ratio 'total_supply_index' is to 'user_assets.index'
				let pool_to_user_ratio = pool.total_supply_index / user_assets.index;
				//	Update the user's supplied amount: pool_to_user ratio * intitial user_assets ratio
				user_assets.supplied_amount = pool_to_user_ratio.saturating_mul_int(user_assets.supplied_amount);
				//	Update the User's Index to the Current Pool's supply Index 
				user_assets.index = pool.total_supply_index;
				UserAssetInfo::<T>::insert(asset_id, account_id, user_assets);
			}
		}
		
		
		pub fn get_user_supply_with_interest(asset_id: T::AssetID, account_id: T::AccountId) -> T::Balance { 
			let mut pool = Self::get_pool(asset_id);
			let now: T::BlockNumber = frame_system::Pallet::<T>::block_number();
			let total_supply_index;
			if let Some(pool_info) = pool { 
				let timespan = Self::block_to_int(now - pool_info.last_updated).unwrap();
				
				let supply_multiplier = FixedU128::one() 
					+ Self::supply_rate_interest(&pool_info) 
					* FixedU128::saturating_from_integer(timespan); 
				total_supply_index = pool_info.total_supply_index * supply_multiplier;
			} else { return T::Balance::zero() }
			
			if let Some(asset) = Self::get_user_supply(asset_id, account_id) { 
				let pool_to_user_ratio = asset.
			}
		}
		///	Runtime Public APIs
		///	Supply Interest Rate of Pool
		pub fn get_supply_interest_rate(asset_id: T::AssetID) -> FixedU128 { 
			if let Some(asset_pool) = Self::get_pool(asset_id) { 
				return Self::supply_rate_interest(&asset_pool)
			} else { 
				log::warn!("📢 Pool Does not Exist!");
				return FixedU128::zero()
			}
		}
		///	Borrowing Interest Rate of Pool
		pub fn get_borrowing_interest_rate(asset_id: T::AssetID) -> FixedU128 { 
			if let Some(asset_pool) = Self::get_pool(asset_id) { 
				return Self::borrowing_rate_interest(&asset_pool)
			} else { 
				log::warn!("📢 Pool Does not Exist!");
				return FixedU128::zero()
			}
		}
		
	}
}

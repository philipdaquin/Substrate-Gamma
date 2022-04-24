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

	use sp_runtime::traits::StaticLookup;

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
		/// Liquidation Threshoold
		type LiquidationThreshold: Get<FixedU128>;
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
	pub type PoolInfo<T: Config> = StorageMap<_, Twox64Concat, T::AssetID, Pools<T>,OptionQuery>;
	
	//	User debt
	#[pallet::storage]
	#[pallet::getter(fn get_user_debt)]
	pub type UserDebtInfo<T: Config> = StorageDoubleMap<
		_, 
		//	[AssetId, AccountId, UserDebt]
		Twox64Concat, T::AssetID, 
		Blake2_128Concat, T::AccountId, 
		UserDebt<T::AssetID, T::Balance>, OptionQuery>;

	//	User asset 
	#[pallet::storage]
	#[pallet::getter(fn get_user_asset)]
	pub type UserAssetInfo<T: Config> = StorageDoubleMap<
		_, 
		//	[AssetId, AccountId, UserAsset]
		Twox64Concat, T::AssetID,
		Blake2_128Concat, T::AccountId, 
		UserAssets<T::AssetID, T::Balance>,
		OptionQuery
	>;

	//	The set of User's asset 
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
		Insufficientasset,
		ReachedLiquidationThreshold,
		InsufficientLiquidity,
		UserHasNoDebt,
		UserPayingZeroAmount
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10)]
		pub fn asset_asset(origin: OriginFor<T>, asset_id: T::AssetID, amount: T::Balance) -> DispatchResult { 
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
			//	Update the User asset Interest 
			Self::update_user_asset(account_id.clone(), asset_id, &pool, amount, true);

			//	Update the Pool asset interest 
			Self::update_pool_assets(&mut pool, amount.clone(), true);
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
		pub fn borrow_asset(origin: OriginFor<T>, asset_id: T::AssetID, amount: T::Balance) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;	

			let mut pool = Self::get_pool(asset_id).ok_or(Error::<T>::DbPoolNotExist)?;

			//	Accrue Pool Interest 
			Self::accrue_interest(&mut pool);
			//	ensure total liquidity is greater than the asking amount 
			ensure!(pool.total_asset - pool.total_debt > amount, Error::<T>::InsufficientLiquidity);
			//	Accrue the user Interet first
			Self::get_user_debt_with_interest(asset_id, account_id);
			
			//	Collaterals
			let (_, current_collateral, needed_collateral) = Self::get_user_balances(account_id.clone());
			let price = T::Oracle::get_rate(asset_id);
			
			let needed_collateral = needed_collateral + price.saturating_mul_int(amount);
			//	Ensure the asset/w interest has not reached pass liquidation threshold 
			ensure!(
				T::LiquidationThreshold::get().saturating_mul_int(needed_collateral) <= current_collateral,
				Error::<T>::ReachedLiquidationThreshold
			);

			//	Transfer assets to user  
			T::MultiAsset::transfer(
				Self::fund_account_id(),
				account_id.clone(),
				asset_id.clone(), 
				amount
			).map_err(|_| Error::<T>::TransferIntoFailed)?;

			//	Update the current user debt: DAdd new debt_amount to the user on chain 
			Self::update_user_debt(account_id, asset_id, &pool, amount, true);
			//	Update the current pool debt: Add new borrowed amount 
			Self::update_pool_debt(&mut pool, amount, true);
			//	Emit borrowed event 
			Self::deposit_event(Event::<T>::Borrowed { 
				asset_id, 
				amount, 
				account_id, 
			});

			//	Update the user's asset debt 
			let mut assets = Self::user_debts(account_id);
			//	Add the debt set to the user's account if it doesnt exist yet
			if !assets.iter().any(|target| *target == asset_id ) { 
				assets.push(asset_id);
				//	Update DB
				UserDebtSet::<T>::insert(account_id, assets);
			} 
			log::info!("🚀 Updated Pool Metrics");
			//	Register new pool state to the onchain db 
			PoolInfo::<T>::insert(asset_id, pool);

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
				ensure!(user_assets.supplied_amount >= amount, Error::<T>::Insufficientasset);
				amount = user_assets.supplied_amount;
			}
			//	Accrue pool interest 
			Self::accrue_interest(&mut pool);
			//	Accrue the user's interest 
			Self::accrue_user_asset(&mut pool, asset_id, account_id.clone());
			
			//	Check Users Collateral, ensure that it would not trigger liquidation process would not be triggered 
			let	(balance, 
				asset_with_interest, debt_balance) = Self::get_user_balances(account_id);			
			// Safe factor 
			let safe_factor = pool.safe_factor.clone();
			//	Get current Price 
			let price = T::Oracle::get_rate(asset_id);
			let converted_asset = 
				asset_with_interest - (price * safe_factor).saturating_mul_int(amount.clone());
			//	Ensure the asset/w interest has not reached pass liquidation threshold 
			ensure!(
				T::LiquidationThreshold::get().saturating_mul_int(debt_balance) <= converted_asset,
				Error::<T>::ReachedLiquidationThreshold
			);
			//	Verify Liquidty inside the pool 
			//	Ensure the asset(w/ interestf) - borrowed is greater than the amount to be withdrawn, else trigger 
			//	'InsufficientLiquidity' Error 
			ensure!(
				pool.total_asset - pool.total_debt > amount, Error::<T>::InsufficientLiquidity);
			
			//	If all Verification passes
			//	Trasnfer the assets of user from the pool 
			T::MultiAsset::transfer(
				Self::fund_account_id(), account_id.clone(), asset_id.clone(), amount
			).map_err(|_| Error::<T>::TransferIntoFailed)?;

			Self::deposit_event(Event::<T>::Withdrawn { asset_id, withdrawn_amount: amount.clone(), account_id });

			//	Update the user records onchain 
			Self::update_user_asset(account_id.clone(), asset_id, &pool, amount, false);

			//	Update the pool assets 
			Self::update_pool_assets(&mut pool, amount, false);

			//	Update the onChaiin Pool database 
			PoolInfo::<T>::insert(asset_id, pool);
			Ok(())
		}
		#[pallet::weight(10)]
		pub fn repay_asset(origin: OriginFor<T>, asset_id: T::AssetID, amount: T::Balance) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;
			let mut pool = Self::get_pool(asset_id).ok_or(Error::<T>::DbPoolNotExist)?;
			//	Accrue the pool interest 
			Self::accrue_interest(&mut pool);
			//  Accrue the user's interest
			Self::accrue_user_debt(&mut pool, asset_id, account_id.clone());
			let mut amount = amount; 
			//	Check if the user owes anything 
			if let Some(user_debt) = Self::get_user_debt(asset_id, account_id) { 
				//	To repay, the repayment must be greater than the debt amount 
				ensure!(user_debt.debt_amount != T::Balance::zero(), Error::<T>::UserHasNoDebt);
				ensure!(amount != T::Balance::zero(), Error::<T>::UserPayingZeroAmount);
				//	Check the amount is enough to pay 
				if amount >= user_debt.debt_amount { 
					amount = user_debt.debt_amount;
				}
			}
			//	Trasnfer the users assets to the pools account 
			T::MultiAsset::transfer(
				account_id,
				Self::fund_account_id(),
				asset_id, 
				amount
			).map_err(|_| Error::<T>::TransferIntoFailed)?;
			//	Update the users debt 
			Self::update_user_debt(account_id, asset_id, &pool, amount, false);
			//	Update the pool's debt 
			Self::update_pool_debt(&mut pool, amount, false);
			//	Deposit event 
			Self::deposit_event(Event::<T>::Repaid { 
				asset_id, 
				repaid_amount: amount,
				account_id
			});

			//	Update on chain database 
			PoolInfo::<T>::insert(asset_id, pool);
			Ok(())
		}
		#[pallet::weight(10)]
		pub fn liquidate(
			origin: OriginFor<T>, 
			source: <T::Lookup as StaticLookup>::Source, 
			payment_id: T::AssetID,
			target_asset_id: T::AssetID, 
			amount: T::Balance
		) -> DispatchResult { 
			let account = ensure_signed(origin)?;

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

			// Get the asset Rate and then calculate the asset Interest 
			let asset_multiplier = Self::asset_rate_interest(pool) 
				+ FixedU128::one() 
				* FixedU128::saturating_from_integer(elapsed_time_in_u32); 
			let debt_multiplier = Self::borrowing_rate_interest(pool)
				+ FixedU128::one()
				* FixedU128::saturating_from_integer(elapsed_time_in_u32);
			
			pool.total_asset = asset_multiplier.saturating_mul_int(pool.total_debt);
			pool.total_asset_index = pool.total_asset_index * asset_multiplier;

			pool.total_debt = debt_multiplier.saturating_mul_int(pool.total_debt);
			pool.total_asset_index = pool.total_debt_index * debt_multiplier;

			pool.last_updated = now;
			log::info!("Accrued Interest Rate");
		}  
		/// asset Interest Rate 
		pub(crate) fn asset_rate_interest(pool: &Pools<T>) -> FixedU128 { 
			//	Check asset asset in the Pool
			if pool.total_asset == T::Balance::zero() { 
				return FixedU128::zero();
			}
			
			//	Utilisation Rate = total debt/ total assets
			//	The utilisation rate represents the percentage of borrows in the total money market 
			let utilization_ratio = FixedU128::saturating_from_rational(pool.total_debt, pool.total_asset);
			Self::borrowing_rate_interest(pool) * utilization_ratio
		}
		///	Borrowing Interest Rate
		pub(crate) fn borrowing_rate_interest(pool: &Pools<T>) -> FixedU128 { 
			if pool.total_asset == T::Balance::zero() { 
				return pool.initial_interest_rate
			}
			let utilization_ratio = FixedU128::saturating_from_rational(pool.total_debt, pool.total_asset);
			pool.initial_interest_rate + pool.utilization_factor * utilization_ratio		
		}

		fn update_user_asset(
			account_id: T::AccountId, 
			asset_id: T::AssetID, 
			pool: &Pools<T>,
			amount: T::Balance, 
			add_on: bool 
		) { 
			if let Some(mut user_assets) = Self::get_user_asset(asset_id, account_id.clone()) { 
				//	Calculate the ratio 'total_asset_index' is to 'user_assets.index'
				let pool_to_user_ratio = pool.total_asset_index / user_assets.index;
				//	Update the user's supplied amount: pool_to_user ratio * intitial user_assets ratio
				user_assets.supplied_amount = pool_to_user_ratio.saturating_mul_int(user_assets.supplied_amount);
				//	Update the User's Index to the Current Pool's asset Index 
				user_assets.index = pool.total_asset_index;
				
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
					log::warn!("📭 User Has Zero Asset, Updating OnChain DB...");

					UserAssetInfo::<T>::remove(asset_id, account_id.clone());
					// Update the User asset Set 
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
					index: pool.total_asset_index 
				};
				//	Update the user's index unique to the pool 
				UserAssetInfo::<T>::insert(
					asset_id, 
					account_id, 
					asset_set	
				);
			}		
		}
		fn update_user_debt(
			account_id: T::AccountId, 
			asset_id: T::AssetID,
			pool: &Pools<T>,
			amount: T::Balance, 
			add_on: bool
		) { 
			if let Some(mut user_debt) = Self::get_user_debt(asset_id, account_id.clone()) { 
				let pool_to_user_ratio = pool.total_debt_index / user_debt.index;
				user_debt.debt_amount = pool_to_user_ratio.saturating_mul_int(user_debt.debt_amount);

				user_debt.index = pool.total_debt_index;

				if add_on { 
					user_debt.debt_amount += amount 
				} else { 
					user_debt.debt_amount -= amount
				}

				if user_debt.debt_amount != T::Balance::zero() { 
					UserDebtInfo::<T>::insert(asset_id, account_id.clone(), user_debt);
				} else { 
					log::warn!("📭 User Has Zero Debt, Updating OnChain DB...");
					//	If the user debt IS NULL
					UserDebtInfo::<T>::remove(asset_id, account_id);
					//	Update the user_set debt 
					let mut debt = Self::user_debts(account_id);
					debt.retain(|x| *x != asset_id );
					UserDebtSet::<T>::insert(account_id, debt);
				}
			} else if amount != T::Balance::zero() { 
				//	Update the user debt index 
				UserDebtInfo::<T>::insert(
					asset_id, 
					account_id,
					UserDebt::<T::AssetID, T::Balance> { 
						asset_id, 
						debt_amount: amount,
						index: pool.total_debt_index
					}
				);
			}
		}

		///	Helper Functions to recalculate the asset and debt
		fn update_pool_assets(pool: &mut Pools<T>, amount: T::Balance, add_on: bool) { 
			log::info!("Recalculating pool asset of assets");
			if add_on  { 
				pool.total_asset += amount;
			} else { 
				pool.total_asset -= amount;			
			}
		}
		fn update_pool_debt(pool: &mut Pools<T>, debt: T::Balance, add_on: bool) { 
			log::info!("Recalculating pool asset of debt");
			
			if add_on { 
				pool.total_debt += debt
			} else { 
				pool.total_debt -= debt
			}
		}
		///	Accrue users assets with Interest 
		fn accrue_user_asset(pool: &mut Pools<T>, asset_id: T::AssetID, account_id: T::AccountId) { 
			if let Some(mut user_assets) = Self::get_user_asset(asset_id, account_id.clone()) { 
				//	Calculate the ratio 'total_asset_index' is to 'user_assets.index'
				let pool_to_user_ratio = pool.total_asset_index / user_assets.index;
				//	Update the user's supplied amount: pool_to_user ratio * intitial user_assets ratio
				user_assets.supplied_amount = pool_to_user_ratio.saturating_mul_int(user_assets.supplied_amount);
				//	Update the User's Index to the Current Pool's asset Index 
				user_assets.index = pool.total_asset_index;
				UserAssetInfo::<T>::insert(asset_id, account_id, user_assets);
			}
		}
		///	Accrue user debt with Interest 
		fn accrue_user_debt(pool: &mut Pools<T>, asset_id: T::AssetID, account_id: T::AccountId) { 
			if let Some(mut user_debt) = Self::get_user_debt(asset_id, account_id) { 
				//	
				let pool_to_user_ratio = pool.total_debt_index / user_debt.index;
				
				user_debt.debt_amount = pool_to_user_ratio.saturating_mul_int(user_debt.debt_amount);
				user_debt.index = pool.total_debt_index;
				
			}
		} 
		
		///	Calculate the value of assets with interest 
		pub fn get_user_asset_with_interest(asset_id: T::AssetID, account_id: T::AccountId) -> T::Balance { 
			//	Get the pool information
			let mut pool = Self::get_pool(asset_id);
			let now: T::BlockNumber = frame_system::Pallet::<T>::block_number();
			let total_asset_index;

			if let Some(pool_info) = pool { 
				let timespan = Self::block_to_int(now - pool_info.last_updated).unwrap();
				
				// 1 + assetInterestRate * (timespan of staking)
				let asset_multiplier = FixedU128::one() 
					+ Self::asset_rate_interest(&pool_info) 
					* FixedU128::saturating_from_integer(timespan); 
				
				total_asset_index = pool_info.total_asset_index * asset_multiplier;
			} else { return T::Balance::zero() }
			
			if let Some(asset) = Self::get_user_asset(asset_id, account_id) { 
				let pool_to_user_ratio = total_asset_index / asset.index;
				pool_to_user_ratio.saturating_mul_int(asset.supplied_amount)
			} else { T::Balance::zero() }
		}
		///	Calculate the value of debt with interest  
		pub fn get_user_debt_with_interest(asset_id: T::AssetID, account_id: T::AccountId) -> T::Balance { 
			let mut pool = Self::get_pool(asset_id);
			let now: T::BlockNumber = frame_system::Pallet::<T>::block_number();
			let total_debt_index: FixedU128;
			if let Some(pool_info) = pool { 
				let timespan = Self::block_to_int(now - pool_info.last_updated).unwrap();
				let debt_multiplier = FixedU128::one()
					+ Self::borrowing_rate_interest(&pool_info) 
					* FixedU128::saturating_from_integer(timespan);
				total_debt_index = pool_info.total_debt_index * debt_multiplier 
			} else { return T::Balance::zero() }

			//	update the user debt
			if let Some(debt) = Self::get_user_debt(asset_id, account_id.clone()) { 
				let pool_to_user_ratio = total_debt_index / debt.index;
				pool_to_user_ratio.saturating_mul_int(debt.debt_amount)
			} else { T::Balance::zero() }
		}

		///	 Total asset Balance
		///  Total convereted supply balance 
		///  Total total debt balance
		fn get_user_balances(account_id: T::AccountId) -> (T::Balance, T::Balance, T::Balance) { 
			let user_assets = Self::user_assets(account_id);
			let (mut balance, mut converted_balance) = (T::Balance::zero(), T::Balance::zero());

			//	Assets with interest 
			for asset_id in user_assets { 
				let amount_interest = Self::get_user_asset_with_interest(asset_id.clone(), account_id.clone());
				let current_price: FixedU128 = T::Oracle::get_rate(asset_id);

				// balance = current_price * accumulated_asset_interest	
				balance += current_price.saturating_mul_int(amount_interest);

				// convertedasset = safefactor * current_price * asset_with_interest 
				let safe_factor = Self::get_pool(asset_id).unwrap().safe_factor;
				converted_balance += (current_price * safe_factor).saturating_mul_int(amount_interest);
			}
			
			//	The amount needed for collateral 
			let mut debt_balance = T::Balance::zero();
			let user_debt = Self::user_debts(account_id);
			for debt in user_debt { 
				let debt_amount = Self::get_user_debt_with_interest(debt, account_id.clone());
				let price = T::Oracle::get_rate(debt);
				debt_balance += price.saturating_mul_int(debt_amount)
			}
			(balance, converted_balance, debt_balance)
		} 

		///	Runtime Public APIs
		///	asset Interest Rate of Pool
		pub fn get_asset_interest_rate(asset_id: T::AssetID) -> FixedU128 { 
			if let Some(asset_pool) = Self::get_pool(asset_id) { 
				return Self::asset_rate_interest(&asset_pool)
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

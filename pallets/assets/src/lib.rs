#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use sp_std::prelude::*;
use sp_runtime::{traits::{AtLeast32BitUnsigned, One, Zero}, ArithmeticError, FixedU128 };
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
	use frame_support::{pallet_prelude::*, dispatch::DispatchResultWithPostInfo, Blake2_128Concat, Twox64Concat, traits::EnsureOrigin};
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
		type Balance: Member + Parameter + AtLeast32BitUnsigned + MaxEncodedLen + Default + Copy;
		///	The arithmetic type of asset identifier
		type AssetID: Member + Parameter + Default + TypeInfo + AtLeast32BitUnsigned + HasCompact 
			+ MaxEncodedLen + Copy;
		//	The origin which may forcibly create or destroy an asset or otherwise alter 
		//	priviledged attributes
		type ForceOrigin: EnsureOrigin<Self::Origin>;
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
	#[pallet::getter(fn get_price)]
	pub type Price<T: Config> = StorageMap<
		_, Twox64Concat, T::AssetID, FixedU128, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Issued { asset_id: T::AssetID, account_id: T::AccountId, balance: T::Balance },
		Burned { asset_id: T::AssetID, account_id: T::AccountId, balance: T::Balance},
		Transferred { asset_id: T::AssetID, from: T::AccountId, to: T::AccountId, amount: T::Balance}, 
		NewPrice {asset_id: T::AssetID, price: FixedU128}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		///	The given asset ID is unknown
		Unknown,
		///	Insufficient Balance
		InsufficientBalance
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///	Mint Assets 
		/// Issue new assets in a permissioned way, if permissionless, then with a deposit is required
		#[pallet::weight(0)]
		pub fn mint(origin: OriginFor<T>, total: T::Balance) -> DispatchResultWithPostInfo { 
			//	User Signs in 
			let account_id = ensure_signed(origin)?;
			
			ensure!(!total.is_zero(), Error::<T>::NoneValue);
			let asset_id = Self::get_next_id().ok().ok_or(Error::<T>::Unknown)?;
			// Increment User Balance 
			Balances::<T>::insert((asset_id, account_id.clone()), total);
			//	Select TotalSupply insert_into AssetId 
			TotalSupply::<T>::insert(asset_id, total);
			//	Events 
			Self::deposit_event(Event::<T>::Issued {asset_id, account_id: account_id, balance: total});
			Ok(().into())
		}
		///	Transfer Asset
		/// Move assets between accounts
		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>, 
			from: T::AccountId, 
			value: T::Balance,
			asset_id: T::AssetID
		) -> DispatchResult { 
			//	User Sign in 
			let account_id = ensure_signed(origin)?;
			Self::transfer_asset(
				account_id.clone(), 
				from.clone(), 
				value, 
				asset_id,
				Event::<T>::Transferred { 
					asset_id,
					from: from,
					to: account_id,
					amount: value 
				}
			)
			
		}
		///	Burn Assets
		/// Decrease the asset balance of an account
		#[pallet::weight(0)]
		pub fn burn(origin: OriginFor<T>, value: T::Balance, asset_id: T::AssetID) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;
			ensure!(!value.is_zero(), Error::<T>::NoneValue);
			let balance = Balances::<T>::take((asset_id, account_id.clone()));

			TotalSupply::<T>::mutate(asset_id, |supply| *supply -= balance);
			Self::deposit_event(Event::<T>::Burned {
				asset_id, 
				account_id, 
				balance
			 });

			Ok(())
		}
		#[pallet::weight(0)]
		pub fn submit_price(origin: OriginFor<T>, asset_id: T::AssetID, price: FixedU128) -> DispatchResult { 
			let account_id = ensure_signed(origin)?;
			Self::add_price( price, asset_id);
			Ok(())
		}
 	}
	impl<T: Config> Pallet<T> { 
		fn get_next_id() ->  Result<T::AssetID, DispatchError>{ 
			let id = Self::next_id();
			NextAssetId::<T>::mutate(|id| *id += One::one());
			Ok(id)
		}
		fn transfer_asset(
			from: T::AccountId, 
			to: T::AccountId, 
			value: T::Balance,
			asset_id: T::AssetID,
			transferred: Event<T>
		) -> DispatchResult { 
			let (target_account, user_account) = 
			((asset_id, to), (asset_id, from));
			//	Get user Balance 
			let user_balance = Self::get_balances(user_account.clone());
			//	Check the balance of sender
			ensure!(user_balance >= value, Error::<T>::InsufficientBalance);
			ensure!(!value.is_zero(), Error::<T>::NoneValue);
			//Balances::<T>::mutate((asset_id, from), |balance| *balance -= value);
			//	Insert: so we can reuse the user_balance stored in cache to recalculate the new balanace
			Balances::<T>::insert(user_account, user_balance - value);
			Balances::<T>::mutate(target_account, |target_balance| *target_balance += value);

			Self::deposit_event(transferred);
			Ok(())
		}
		pub fn add_price(price: FixedU128, asset_id: T::AssetID) -> DispatchResult { 
			Price::<T>::insert(asset_id, price);
			
			Self::deposit_event(Event::<T>::NewPrice {asset_id, price });			
			Ok(())
		}
		pub fn set_price(asset_id: T::AssetID, price: FixedU128) { 
			Price::<T>::insert(asset_id, price);
		}
		//	API Queries 
		//	Query the totalsupply for a specified AssetId 
		fn get_supply_by_id(asset_id: T::AssetID) -> Result<T::Balance, DispatchError> { 
			Ok(TotalSupply::<T>::get(asset_id))
		}
		fn get_balance_by_user(account_id: T::AccountId, asset_id: T::AssetID) -> Result<T::Balance, DispatchError> { 
			Ok(Balances::<T>::get((asset_id, account_id)))
		}
	}
}
 
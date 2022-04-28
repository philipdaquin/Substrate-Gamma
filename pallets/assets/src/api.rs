use super::*;
use common::{MultiAsset, Oracle, AssetBalance};
use frame_support::dispatch::DispatchResult;


impl<T: Config> MultiAsset<T::AccountId, T::AssetID, T::Balance> for Pallet<T> { 
    fn transfer(from: T::AccountId, to: T::AccountId, asset_id: T::AssetID, amount: T::Balance) -> DispatchResult {
        let transferred = Event::<T>::Transferred { 
            asset_id, 
            from: from.clone(), 
            to: to.clone(), 
            amount 
        };
        
        Self::transfer_asset(from, to, amount, asset_id, transferred)
    }
}

impl<T: Config> AssetBalance<T::AssetID, T::AccountId, T::Balance> for Pallet<T> {
    fn balance(asset_id: T::AssetID, account_id: T::AccountId) -> T::Balance {
        Self::get_balances((asset_id, account_id))
    } 
}
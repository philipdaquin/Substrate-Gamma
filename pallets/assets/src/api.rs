use super::*;
use common::{MultiAsset, Oracle};
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
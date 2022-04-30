use super::*;
use common::{MultiAsset, Oracle, AssetBalance, DefaultAsset};
use frame_support::dispatch::DispatchResult;
use sp_std;

impl<T: Config> MultiAsset<T::AccountId, T::AssetID, T::Balance> for Pallet<T> { 
    fn transfer(from: T::AccountId, to: T::AccountId, asset_id: T::AssetID, amount: T::Balance) -> sp_std::result::Result<(), &'static str> {
        Self::transfer_asset(from, to, amount, asset_id)
    }
}

impl<T: Config> AssetBalance<T::AssetID, T::AccountId, T::Balance> for Pallet<T> {
    fn balance(asset_id: T::AssetID, account_id: T::AccountId) -> T::Balance {
        Self::get_balances((asset_id, account_id))
    } 
}

impl<T: Config> DefaultAsset<T::AssetID> for Pallet<T> { 
    fn get_default_asset() -> T::AssetID {
        Self::get_default_asset().expect("")
    }
}
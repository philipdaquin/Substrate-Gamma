#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
pub trait Oracle<AssetID, Rate> { 
	fn get_rate(asset_id: AssetID) -> Rate;
}
pub trait MultiAsset<AccountId, AssetID, Balance> { 
	fn transfer(
		from: AccountId, 
		to: AccountId, 
		asset_id: AssetID, 
		amount: Balance
	) -> sp_std::result::Result<(), &'static str>;
}

pub trait AssetBalance<AssetId, AccountId, Balance> { 
	fn balance(asset_id: AssetId, account_id: AccountId) -> Balance;
}
#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::result::Result;
pub trait Oracle<AssetID, Rate> { 
	fn get_rate(asset_id: AssetID) -> Rate;
}
pub trait MultiAsset<AccountId, AssetID, Balance> { 
	fn transfer(
		from: AccountId, 
		to: AccountId, 
		id: AssetID, 
		amount: Balance
	) -> Result<(), &'static str>;
}
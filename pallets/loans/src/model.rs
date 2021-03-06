use super::*;
use sp_std::vec::Vec;

#[cfg(all(feature = "derive", feature = "max-encoded-len"))]
pub use parity_scale_codec_derive::MaxEncodedLen;


#[derive(PartialEq, RuntimeDebug, Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Pool<AssetID, Balance, BlockNumber> { 
    /// If the pool is enabled
    pub enabled: bool,
    /// If the asset can be enabled as collateral
    pub is_collateral: bool,
    /// The underlying asset
    pub asset: AssetID,
    /// Total supply of the pool 
    pub total_asset: Balance,
    /// Total depoobt of the pool
    pub total_debt: Balance,
    /// Effective index of current total supply
    pub total_asset_index: FixedU128,
    /// Effective index of current total debt
    pub total_debt_index: FixedU128,
    /// The latest timestamp that the pool has accrued interest
    pub last_updated: BlockNumber,
    /// One factor of the linear interest model 
    pub utilization_factor: FixedU128,
    /// Another factor of the linear interest model
    pub initial_interest_rate: FixedU128,
    /// A discount factor for an asset that reduces the its limit 
    pub safe_factor: FixedU128, 
    /// Factor that determines what percentage one arbitrage can seize <= 1
    pub close_factor: FixedU128,
    /// The bonus arbitrager can get when triggering a liquidation 
    pub discount_factor: FixedU128
}

/// Information about the User Supplied Assets
#[derive(PartialEq, RuntimeDebug, Eq, Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct UserAssets<AssetID, Balance> { 
    /// Asset Id Supplied
    pub asset_id: AssetID,
    /// Amount supplied by the User
    pub supplied_amount: Balance,
	///	Index 
	pub index: FixedU128
}
/// Information about the User Debt
#[derive(PartialEq, RuntimeDebug, Eq, Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct UserDebt<AssetID, Balance> { 
    /// AssetId Owed
    pub asset_id: AssetID, 
    /// Debt accounted by the User
    pub debt_amount: Balance,
	///	Index 
	pub index: FixedU128
}

#[derive(PartialEq, RuntimeDebug, Eq, Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct UserSet<AssetID> { 
    pub asset_set: Vec<AssetID>
}


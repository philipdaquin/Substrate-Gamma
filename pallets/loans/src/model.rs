use super::*;
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use sp_runtime::{FixedU128, FixedI128};
use super::Config;

#[derive(Clone, PartialEq,  RuntimeDebug, Encode, Decode)]
pub struct Pool<T: Config> { 
    /// If the pool is enabled
    pub enabled: bool,
    /// If the asset can be enabled as collateral
    pub is_collateral: bool,
    /// The underlying asset
    pub asset: T::AssetID,
    /// Total supply of the pool 
    pub total_supply: T::Balance,
    /// Total debt of the pool
    pub debt: T::Balance,
    /// Effective index of current total supply
    pub total_supply_index: FixedU128,
    /// Effective index of current total debt
    pub total_debt_index: FixedU128,
    /// The latest timestamp that the pool has accrued interest
    pub last_updated: T::BlockNumber,
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
#[derive(PartialEq, RuntimeDebug, Encode, Decode, Clone)]
pub struct UserAssets<T: Config> { 
    /// Asset Id Supplied
    pub asset_id: T::AssetID,
    /// Amount supplied by the User
    pub supplied_amount: T::Balance
}
/// Information about the User Debt
#[derive(Clone, PartialEq, RuntimeDebug, Encode, Decode)]
pub struct UserDebt<T: Config> { 
    /// AssetId Owed
    pub asset_id: T::AssetID, 
    /// Debt accounted by the User
    pub debt_amount: T::Balance
}
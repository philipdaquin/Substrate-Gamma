#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

sp_api::decl_runtime_apis! { 
    pub trait LoansApi<AssetID, FixedU128, AccountId, Balance> where
        AssetID: Codec,
        FixedU128: Codec,
        AccountId: Codec, 
        Balance: Codec
    { 
        fn get_user_balances(account_id: AccountId) -> (u64, u64, u64);
        fn get_asset_interest_rate(asset_id: AssetID) -> FixedU128;
        fn get_borrowing_interest_rate(asset_id: AssetID) -> FixedU128;
        fn get_user_asset_with_interest(asset_id: AssetID, account_id: AccountId) -> Balance;
        fn get_user_debt_with_interest(asset_id: AssetID, account_id: AccountId) -> Balance;
        fn user_assets(account_id: AccountId) -> 
        
    }
}
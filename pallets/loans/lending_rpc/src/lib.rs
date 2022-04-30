


use std::sync::Arc;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
// use pallet_loans_rpc_runtime_api::runtime_decl_for_LoansApi::LoansApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT, FixedU128};
pub use pallet_loans_rpc_runtime_api::LoansApi as LendingApi ;

#[rpc]
pub trait LoansApi<BlockHash, AssetID, FixedU128, AccountId, Balance> { 
    
    #[rpc(name = "getUserBalances")]
    fn get_user_balances(
        &self, 
        account_id: AccountId,
        at: Option<BlockHash> 
    ) -> Result<(u64, u64, u64)>;
}

pub struct Loans<C, B> { 
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>
}

impl<C, B> Loans<C, B> { 
    fn new(client: Arc<C>) -> Self { 
        Self { 
            client,
            _marker: Default::default()
        }
    }
}

impl<C, Block, AssetID, FixedU128, AccountId, Balance> LoansApi<
    <Block as BlockT>::Hash, AssetID, FixedU128, AccountId, Balance> for Loans<C, Block> 
where  
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api : LendingApi<Block, AssetID, FixedU128, AccountId, Balance>,
    AssetID: Codec,
    FixedU128: Codec,
    AccountId: Codec,
    Balance: Codec,
{ 
    fn get_user_balances(
        &self, 
        account_id: AccountId, 
        at:Option<<Block as BlockT>::Hash>
    ) ->Result<(u64, u64, u64)>  {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_res = api.get_user_balances(&at, account_id);
        runtime_api_res 
            .map_err(|f| RpcError { 
                code: ErrorCode::ServerError(9876),
                message: "RPC Error".into(),
                data: Some(format!("{:?}", f).into())
            })
    }
}
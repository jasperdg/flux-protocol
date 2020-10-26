use super::*;
use near_sdk::MockedBlockchain;
use near_sdk::{VMContext, testing_env};
use near_crypto::{InMemorySigner, KeyType, Signer};
use near_runtime_standalone::{init_runtime_and_signer, RuntimeStandalone};
use near_primitives::{
    account::{AccessKey},
    errors::{RuntimeError, TxExecutionError},
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{AccountId, Balance},
};
use std::collections::{HashMap};

use serde_json::json;
use near_sdk::json_types::{U128, U64};

const GAS_STANDARD: u64 = 10000000000000000;

fn fun_token() -> String {
	return "fun_token".to_string();
}

fn flux_protocol() -> String {
	return "flux_protocol".to_string();
}

type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

lazy_static::lazy_static! {
    static ref MARKETS_BYTES: &'static [u8] = include_bytes!("../../../res/flux_protocol.wasm").as_ref();
    static ref FUNGIBLE_TOKEN_BYTES: &'static [u8] = include_bytes!("../../../fungible_token.wasm").as_ref();
}

pub fn ntoy(near_amount: Balance) -> Balance {
    near_amount * 10u128.pow(24)
}

fn outcome_into_result(outcome: ExecutionOutcome) -> TxResult {
    match outcome.status {
        ExecutionStatus::SuccessValue(_) => Ok(outcome),
        ExecutionStatus::Failure(_) => Err(outcome),
        ExecutionStatus::SuccessReceiptId(_) => panic!("Unresolved ExecutionOutcome run runtime.resolve(tx) to resolve the final outcome of tx"),
        ExecutionStatus::Unknown => unreachable!()
    }
}

pub struct ExternalUser {
    account_id: AccountId,
    signer: InMemorySigner,
}

impl ExternalUser {
	pub fn new(account_id: AccountId, signer: InMemorySigner) -> Self {
        Self { account_id, signer }
    }

    pub fn get_account_id(&self) -> AccountId {
        return self.account_id.to_string();
    }

    pub fn deploy_flux_protocol(&self, runtime: &mut RuntimeStandalone) -> TxResult {
        let args = json!({
            "fun_token_account_id": fun_token(),
            "owner": self.get_account_id(),
        }).to_string().as_bytes().to_vec();

        let tx = self
        .new_tx(runtime, flux_protocol())
        .create_account()
        .transfer(99994508400000000000000000)
        .deploy_contract(MARKETS_BYTES.to_vec())
        .function_call("init".into(), args, GAS_STANDARD, 0)
        .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn deploy_fun_token(&self, runtime: &mut RuntimeStandalone, owner_account_id: String, total_supply: U128) -> TxResult {
        // let args = json!({}).to_string().as_bytes().to_vec();
        let args = json!({
            "owner_id": owner_account_id,
            "total_supply": total_supply
        }).to_string().as_bytes().to_vec();

        let tx = self
        .new_tx(runtime, fun_token())
        .create_account()
        .transfer(99994508400000000000000000)
        .deploy_contract(FUNGIBLE_TOKEN_BYTES.to_vec())
        .function_call("new".into(), args, GAS_STANDARD, 0)
        .sign(&self.signer);
        let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    fn new_tx(&self, runtime: &RuntimeStandalone, receiver_id: AccountId) -> Transaction {
        let nonce = runtime
        .view_access_key(&self.account_id, &self.signer.public_key())
        .unwrap()
        .nonce
        + 1;
        Transaction::new(
            self.account_id.clone(),
            self.signer.public_key(),
            receiver_id,
            nonce,
            CryptoHash::default(),
        )
    }

	pub fn create_external(
        &self,
        runtime: &mut RuntimeStandalone,
        new_account_id: AccountId,
        amount: Balance,
    ) -> Result<ExternalUser, ExecutionOutcome> {
        let new_signer = InMemorySigner::from_seed(&new_account_id, KeyType::ED25519, &new_account_id);
        let tx = self
        .new_tx(runtime, new_account_id.clone())
        .create_account()
        .add_key(new_signer.public_key(), AccessKey::full_access())
        .transfer(amount)
        .sign(&self.signer);

        let res = runtime.resolve_tx(tx);

        // TODO: this temporary hack, must be rewritten
        if let Err(err) = res.clone() {
            if let RuntimeError::InvalidTxError(tx_err) = err {
                let mut out = ExecutionOutcome::default();
                out.status = ExecutionStatus::Failure(TxExecutionError::InvalidTxError(tx_err));
                return Err(out);
            } else {
                unreachable!();
            }
        } else {
            outcome_into_result(res.unwrap())?;
            runtime.process_all().unwrap();
            Ok(ExternalUser {
                account_id: new_account_id,
                signer: new_signer,
            })
        }
	}
	
	// flux runtime contract method helpers

	pub fn create_market(
        &self,
        runtime: &mut RuntimeStandalone,
        description: String,
        extra_info: String,
        outcomes: u8,
        outcome_tags: Vec<String>,
        categories: Vec<String>,
        end_time: U64,
        creator_fee_percentage: u32,
        affiliate_fee_percentage: u32,
        api_source: String,
    ) -> TxResult {
        let args = json!({
            "description": description,
            "extra_info": extra_info,
            "outcomes": outcomes,
            "outcome_tags": outcome_tags,
            "categories": categories,
            "end_time": end_time,
            "creator_fee_percentage": creator_fee_percentage,
            "affiliate_fee_percentage": affiliate_fee_percentage,
            "api_source": api_source,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("create_market".into(), args, GAS_STANDARD, 0)
        .sign(&self.signer);
        let res = runtime.resolve_tx(tx).expect("resolving tx failed");
        runtime.process_all().expect("processing tx failed");
        let ans = outcome_into_result(res);
        return ans;
	}

	pub fn place_order(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        outcome: u8,
        shares: U128,
		price: U128,
		affiliate_account_id: Option<String>
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "outcome": outcome,
            "shares": shares,
			"price": price,
			"affiliate_account_id": affiliate_account_id
        })
        .to_string()
        .as_bytes()
        .to_vec();
        			
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("place_order".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn cancel_order(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        outcome: u8,
        price: U128,
        order_id: U128
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "outcome": outcome,
            "price": price,
            "order_id": order_id,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("cancel_order".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn dynamic_market_sell(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        outcome: u8,
        shares: U128,
        min_price: U128
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "outcome": outcome,
            "shares": shares,
            "min_price": min_price
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("dynamic_market_sell".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn resolute_market(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        winning_outcome: Option<u8>,
        stake: U128
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "winning_outcome": winning_outcome,
            "stake": stake,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("resolute_market".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }
    
    pub fn dispute_market(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        winning_outcome: Option<u8>,
        stake: U128
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "winning_outcome": winning_outcome,
            "stake": stake,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("dispute_market".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn finalize_market(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        winning_outcome: Option<u8>,
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "winning_outcome": winning_outcome,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("finalize_market".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn claim_earnings(
        &self,
        runtime: &mut RuntimeStandalone,
		market_id: U64, 
		account_id: String
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "account_id": account_id,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("claim_earnings".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn claim_affiliate_earnings(
        &self,
        runtime: &mut RuntimeStandalone,
        account_id: String
    ) -> TxResult {
        let args = json!({
            "account_id": account_id
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("claim_affiliate_earnings".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn withdraw_dispute_stake(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64,
        dispute_round: U64,
        outcome: Option<u8>
    ) -> TxResult {
        let args = json!({
            "market_id": market_id,
            "dispute_round": dispute_round,
            "outcome": outcome,
        })
        .to_string()
        .as_bytes()
        .to_vec();
        
        let tx = self
        .new_tx(runtime, flux_protocol())
        .function_call("withdraw_dispute_stake".into(), args, 10000000000000000, 0)
        .sign(&self.signer);
		
		let res = runtime.resolve_tx(tx).unwrap();
        runtime.process_all().unwrap();
        let ans = outcome_into_result(res);
        return ans;
    }

    pub fn get_owner(
        &self,
        runtime: &RuntimeStandalone
    ) -> String {
        let res = runtime.view_method_call(
            &(flux_protocol()), 
            "owner", 
            json!({})
        .to_string()
        .as_bytes())
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(res.as_slice()).unwrap();
        let market_price = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return market_price;
    }

    pub fn get_market_price(
        &self, 
        runtime: &RuntimeStandalone, 
        market_id: U64, 
        outcome: u8
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_market_price",
            json!({"market_id": market_id, "outcome": outcome})
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let market_price = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return market_price;
    }

    pub fn get_outcome_share_balance(
        &self, 
        runtime: &RuntimeStandalone, 
        account_id: String, 
        market_id: U64, 
        outcome: u8
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_outcome_share_balance",
            json!({
                "account_id": account_id,
                "market_id": market_id, 
                "outcome": outcome,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let share_balance = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return share_balance;
    }

    pub fn get_market_sell_depth(
        &self, 
        runtime: &RuntimeStandalone, 
        market_id: U64, 
        outcome: u8,
        shares: U128
    ) -> (U128, U128) {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_market_sell_depth",
            json!({
                "market_id": market_id, 
                "outcome": outcome,
                "shares": shares,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }

    // Methods for unit tests
    pub fn get_open_orders_len(
        &self, 
        runtime: &RuntimeStandalone,  
        market_id: U64, 
        outcome: u8
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_open_orders_len",
            json!({
                "market_id": market_id, 
                "outcome": outcome,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }
    
    pub fn get_filled_orders_len(
        &self, 
        runtime: &RuntimeStandalone,  
        market_id: U64, 
        outcome: u8
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_filled_orders_len",
            json!({
                "market_id": market_id, 
                "outcome": outcome,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }
    
    pub fn get_market_volume(
        &self, 
        runtime: &RuntimeStandalone,  
        market_id: U64, 
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_market_volume",
            json!({
                "market_id": market_id, 
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }

    pub fn get_liquidity(
        &self, 
        runtime: &RuntimeStandalone,  
        market_id: U64, 
        outcome: u8,
        price: U128
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_liquidity",
            json!({
                "market_id": market_id, 
                "outcome": outcome,
                "price": price,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }

    pub fn get_depth(
        &self, 
        runtime: &RuntimeStandalone,  
        market_id: U64, 
        outcome: u8,
        spend: U128,
        price: U128
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_depth",
            json!({
                "market_id": market_id, 
                "outcome": outcome,
                "spend": spend,
                "price": price,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }

    pub fn get_claimable(
        &self,
        runtime: &mut RuntimeStandalone,
        market_id: U64, 
		account_id: String
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(flux_protocol()),
            "get_claimable",
            json!({
                "market_id": market_id, 
                "account_id": account_id,
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        //TODO: UPDATE THIS CASTING
        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }


    // external token runtime helper methods
    
    pub fn set_allowance(
		&self,
		runtime: &mut RuntimeStandalone,
        escrow_account_id: AccountId, 
        allowance: U128
	) -> TxResult {
		let args = json!({
            "escrow_account_id": escrow_account_id,
            "allowance": allowance
        })
        .to_string()
        .as_bytes()
        .to_vec();
		
		let tx = self.new_tx(runtime, fun_token())
        .function_call("set_allowance".into(), args, GAS_STANDARD, 0)
        .sign(&self.signer);

		let res = runtime.resolve_tx(tx).expect("processing tx failed");
		let ans = outcome_into_result(res);
		return ans;
    }
    
    pub fn transfer(
		&self,
		runtime: &mut RuntimeStandalone,
        new_owner_id: AccountId, 
        amount: U128
	) -> TxResult {
		let args = json!({
            "new_owner_id": new_owner_id,
            "amount": amount
        })
        .to_string()
        .as_bytes()
        .to_vec();
		
		let tx = self.new_tx(runtime, fun_token())
        .function_call("transfer".into(), args, GAS_STANDARD, 0)
        .sign(&self.signer);

		let res = runtime.resolve_tx(tx).expect("processing tx failed");
		let ans = outcome_into_result(res);
		return ans;
    }
    

    pub fn get_balance(
        &self,
        runtime: &mut RuntimeStandalone,
        account_id: String,
    ) -> U128 {
        let market_price_json = runtime
        .view_method_call(
            &(fun_token()),
            "get_balance",
            json!({
                "owner_id": account_id
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
        .0;

        let data: serde_json::Value = serde_json::from_slice(market_price_json.as_slice()).unwrap();
        let res = serde_json::from_value(serde_json::to_value(data).unwrap()).unwrap();

        return res;
    }

}

pub fn init_markets_contract() -> (RuntimeStandalone, ExternalUser) {
    let (mut runtime, signer) = init_runtime_and_signer(&"flux-dev".into());
    let root = ExternalUser::new("flux-dev".into(), signer);

    root.deploy_flux_protocol(&mut runtime).unwrap();
    
    return (runtime, root);
}
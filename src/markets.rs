use near_sdk::{
	near_bindgen, 
	env, 
	ext_contract, 
	callback, 
	Promise, 
	PromiseOrValue, 
	json_types::{U128, U64},
	PromiseResult,
	collections::{
		UnorderedMap,
		TreeMap,
	}
};
use borsh::{BorshDeserialize, BorshSerialize};

mod market;
type Market = market::Market;
type Order = market::orderbook::order::Order;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Markets {
	creator: String,
	markets: UnorderedMap<u64, Market>,
	nonce: u64,
	max_fee_percentage: u128,
	creation_bond: u128,
	affiliate_earnings: UnorderedMap<String, u128>
}

const SINGLE_CALL_GAS: u64 = 200000000000000;

#[ext_contract]
pub trait FunToken {
    fn transfer_from(&mut self, owner_id: String, new_owner_id: String, amount: U128);
    fn transfer(&mut self, new_owner_id: String, amount: U128);
    fn get_total_supply(&self) -> u128;
    fn get_balance(&self, owner_id: AccountId) -> u128;
}

#[ext_contract]
pub trait FluxProtocol {
    fn proceed_order_placement(&mut self, sender: String, market_id: u64, outcome: u64, amount_of_shares: u128, spend: u128, price: u128, affiliate_account_id: Option<String>);
    fn proceed_market_resolution(&mut self, sender: String, market_id: u64, winning_outcome: Option<u64>, stake: u128);
    fn proceed_market_dispute(&mut self, sender: String, market_id: u64, winning_outcome: Option<u64>, stake: u128);
}

#[near_bindgen]
impl Markets {

	fn dai_token(
		&self
	) -> u128 {
		let base: u128 = 10;
		return base.pow(17)
	}

	fn fun_token_account_id(
		&self
	) -> String {
		return "fun_token".to_string();
	}

	fn assert_self(
		&self
	) {
		assert_eq!(env::current_account_id(), env::predecessor_account_id(), "this method can only be called by the contract itself"); 
	}

	pub fn create_market(
		&mut self, 
		description: String, 
		extra_info: String, 
		outcomes: U64,
		outcome_tags: Vec<String>,
		categories: Vec<String>,
		end_time: U64,
		creator_fee_percentage: U128,
		affiliate_fee_percentage: U128,
		api_source: String
	) -> u64 {
		let outcomes: u64 = outcomes.into();
		let end_time: u64 = end_time.into();
		let creator_fee_percentage: u128 = creator_fee_percentage.into();
		let affiliate_fee_percentage: u128 = affiliate_fee_percentage.into();

		assert!(outcomes > 1);
		assert!(outcomes == 2 || outcomes == outcome_tags.len() as u64);
		assert!(outcomes < 20); // up for change
		assert!(end_time > env::block_timestamp() / 1000000);
		assert!(categories.len() < 6);
		assert!(creator_fee_percentage <= self.max_fee_percentage);
		assert!(affiliate_fee_percentage <= 100);

		if outcomes == 2 {assert!(outcome_tags.len() == 0)}
		// TODO check if end_time hasn't happened yet
		let account_id = env::predecessor_account_id();

		// TODO: Escrow bond account_id creator's account
		let new_market = Market::new(self.nonce, account_id, description, extra_info, outcomes, outcome_tags, categories, end_time, creator_fee_percentage, 1, affiliate_fee_percentage ,api_source);
		let market_id = new_market.id;
		self.markets.insert(&self.nonce, &new_market);
		self.nonce = self.nonce + 1;
		return market_id;
	}

	pub fn place_order(
		&mut self, 
		market_id: U64, 
		outcome: U64, 
		spend: U128, 
		price: U128,
		affiliate_account_id: Option<String>
	) -> Promise {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let spend: u128 = spend.into();
		let price: u128 = price.into();

		let market = self.markets.get(&market_id).expect("market doesn't exist");
		
		assert!(spend >= 10000, "order must be valued at > 10000");
		assert!(price > 0 && price < 100, "price can only be between 0 - 100");
		assert!(outcome < market.outcomes, "invalid outcome");
		assert_eq!(market.resoluted, false, "market has already been resoluted");
		assert!(env::block_timestamp() / 1000000 < market.end_time, "market has already ended");
		
		let amount_of_shares = spend / price;
		let rounded_spend = amount_of_shares * price;

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), rounded_spend.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS)
		.then(
			flux_protocol::proceed_order_placement( 
				env::predecessor_account_id(),
				market_id,
				outcome,
				amount_of_shares,
				rounded_spend,
				price,
				affiliate_account_id,
				&env::current_account_id(), 
				0, 
				SINGLE_CALL_GAS * 3
			)
		);
	}

	pub fn proceed_order_placement(
		&mut self,
		sender: String,
		market_id: u64, 
		outcome: u64,
		amount_of_shares: u128,
		spend: u128, 
		price: u128,
		affiliate_account_id: Option<String>,
	) -> PromiseOrValue<bool> {
		env::log(format!("Order placement proceeding").as_bytes());
		self.assert_self();
		
		let transfer_succeeded = self.is_promise_success();
		if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", spend, env::current_account_id()); }
		
		env::log(format!("transfer success, order placement succeeded").as_bytes());

		let mut market = self.markets.get(&market_id).unwrap();
		market.create_order(sender, outcome, amount_of_shares, spend, price, affiliate_account_id);
		self.markets.insert(&market.id, &market);
		return PromiseOrValue::Value(true);
	}

	pub fn cancel_order(
		&mut self, 
		market_id: U64, 
		outcome: U64, 
		order_id: U128
	) {
	// ) -> Promise {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let order_id: u128 = order_id.into();
		
		let mut market = self.markets.get(&market_id).unwrap();
		assert_eq!(market.resoluted, false);
		let mut orderbook = market.orderbooks.get(&outcome).unwrap();
		let order = orderbook.open_orders.get(&order_id).unwrap();
		assert!(env::predecessor_account_id() == order.creator);
		
		let to_return = orderbook.remove_order(order_id);
		market.orderbooks.insert(&outcome, &orderbook);
		self.markets.insert(&market_id, &market);
		fun_token::transfer(env::predecessor_account_id(), to_return.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
    }

	pub fn resolute_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<U64>,
		stake: U128
	) -> Promise {
		let market_id: u64 = market_id.into();
		let winning_outcome: Option<u64> = match winning_outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};
		let stake_u128: u128 = stake.into();
		let market = self.markets.get(&market_id).expect("market doesn't exist");
		assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, false, "market is already resoluted");
		assert_eq!(market.finalized, false, "market is already finalized");
		assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");


		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, SINGLE_CALL_GAS)
		.then(
			flux_protocol::proceed_market_resolution(
				env::predecessor_account_id(),
				market_id,
				winning_outcome,
				stake_u128,
				&env::current_account_id(),
				0,
				SINGLE_CALL_GAS * 2
			)
		);
	}

	pub fn proceed_market_resolution(
		&mut self,
		market_id: u64,
		winning_outcome: Option<u64>,
		stake: u128,
		sender: String
	) -> PromiseOrValue<bool> {
		self.assert_self();
		env::log(b"attempting to proceed market resolution");
		let transfer_succeeded = self.is_promise_success();
		if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", stake, env::current_account_id()); }
		env::log(format!("parent promise (transfer) was succesfull {}", stake).as_bytes());
		
		let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		let change: u128 = market.resolute(sender.to_string(), winning_outcome, stake).into();
		self.markets.insert(&market_id, &market);
		if change > 0 {
			let prom = fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
			return PromiseOrValue::Promise(prom);
		} else {
			return PromiseOrValue::Value(true);
		}
	}

	pub fn withdraw_dispute_stake(
		&mut self, 
		market_id: U64,
		dispute_round: U64,
		outcome: Option<U64>
	) -> Promise {
		let market_id: u64 = market_id.into();
		let dispute_round: u64 = dispute_round.into();
		let outcome: Option<u64> = match outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};

		let mut market = self.markets.get(&market_id).expect("invalid market");
		let to_return = market.cancel_dispute_participation(dispute_round, outcome);
		self.markets.insert(&market_id, &market);
		if to_return > 0 {
			return fun_token::transfer(env::predecessor_account_id(), U128(to_return), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		} else {
			panic!("user has no participation in this dispute");
		}
	}

	pub fn dispute_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<U64>,
		stake: U128
	) -> Promise {
		let market_id: u64 = market_id.into();
		let winning_outcome: Option<u64> = match winning_outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};
		let stake_u128: u128 = stake.into();
        let market = self.markets.get(&market_id).expect("market doesn't exist");

		assert_eq!(market.resoluted, true, "market isn't resoluted yet");
		assert_eq!(market.finalized, false, "market is already finalized");
        assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");
        assert!(winning_outcome != market.winning_outcome, "same oucome as last resolution");
		let resolution_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("Invalid dispute window unwrap");
		assert_eq!(resolution_window.round, 1, "for this version, there's only 1 round of dispute");
		assert!(env::block_timestamp() / 1000000 <= resolution_window.end_time, "dispute window is closed, market can be finalized");

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
			flux_protocol::proceed_market_dispute(
				env::predecessor_account_id(),
				market_id,
				winning_outcome,
				stake_u128,
				&env::current_account_id(), 
				0, 
				SINGLE_CALL_GAS * 2
			)
		)
	}

	pub fn proceed_market_dispute(		
		&mut self,
		market_id: u64,
		winning_outcome: Option<u64>,
		stake: u128,
		sender: String
	) -> PromiseOrValue<bool> {
		self.assert_self();
		let transfer_succeeded = self.is_promise_success();
		if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", stake, env::current_account_id()); }
        let mut market = self.markets.get(&market_id).expect("market doesn't exist");

		let change = market.dispute(sender.to_string(), winning_outcome, stake);

		self.markets.insert(&market.id, &market);
		if change > 0 {
			return PromiseOrValue::Promise(fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS));
		} else {
			return PromiseOrValue::Value(true);
		}
	}
		

	pub fn finalize_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<U64>
	) {
		let market_id: u64 = market_id.into();
		let winning_outcome: Option<u64> = match winning_outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};

		let mut market = self.markets.get(&market_id).unwrap();
		assert_eq!(market.resoluted, true, "market has to be resoluted before it can be finalized");
		if market.disputed {
			assert_eq!(env::predecessor_account_id(), self.creator, "only the judge can resolute disputed markets");
		} else {
			let dispute_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("no dispute window found, something went wrong");
			assert!(env::block_timestamp() / 1000000 >= dispute_window.end_time || dispute_window.round == 2, "dispute window still open")
		}

		market.finalize(winning_outcome);
		self.markets.insert(&market_id, &market);
	}

	pub fn get_open_orders_len(
		&self, 
		market_id: U64, 
		outcome: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();

		let market = self.markets.get(&market_id).unwrap();
		let orderbook = market.orderbooks.get(&outcome).unwrap();
		return U128(orderbook.open_orders.len() as u128);
	}

	pub fn get_filled_orders_len(
		&self, 
		market_id: U64, 
		outcome: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();

		let market = self.markets.get(&market_id).unwrap();
		let orderbook = market.orderbooks.get(&outcome).unwrap();
		return U128(orderbook.filled_orders.len() as u128);
	}

	pub fn get_claimable(
		&self, 
		market_id: U64, 
		account_id: String
	) -> U128 {
		let market_id: u64 = market_id.into();
		let market = self.markets.get(&market_id).expect("market doesn't exist");

		let (winnings, left_in_open_orders, governance_earnings, _) = market.get_claimable_for(account_id.to_string());
		let market_creator_fee = winnings * market.creator_fee_percentage / 100;
		let resolution_fee = winnings * market.resolution_fee_percentage / 100;

		return (winnings - market_creator_fee - resolution_fee + governance_earnings + left_in_open_orders).into();
	}

	pub fn claim_earnings(
		&mut self, 
		market_id: U64, 
		account_id: String
	) -> Promise {
		let market_id: u64 = market_id.into();
		let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		let market_creator = market.creator.to_string();
		assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, true);
		assert_eq!(market.finalized, true);
		
		let (winnings, left_in_open_orders, governance_earnings, affiliates) = market.get_claimable_for(account_id.to_string());
		let mut market_creator_fee = winnings * market.creator_fee_percentage / 100;
		let creator_fee_percentage = market.creator_fee_percentage;
		let resolution_fee = winnings * market.resolution_fee_percentage / 100;
		let affiliate_fee_percentage = market.affiliate_fee_percentage;
		let mut paid_to_affiliates = 0;

		market.reset_balances_for(account_id.to_string());
		market.delete_resolution_for(account_id.to_string());

		for (affiliate_account_id, amount_owed) in affiliates {
			let affiliate_owed = amount_owed * affiliate_fee_percentage * creator_fee_percentage / 10000;
			paid_to_affiliates += affiliate_owed;
			market_creator_fee -= affiliate_owed;
			let affiliate_earnings = self.affiliate_earnings.get(&affiliate_account_id).unwrap_or(0);
			self.affiliate_earnings.insert(&affiliate_account_id, &(affiliate_earnings + affiliate_owed));
		}

		let total_fee = market_creator_fee + paid_to_affiliates + resolution_fee;
		let to_claim = winnings + governance_earnings + left_in_open_orders;		
		let earnings = to_claim - total_fee;
		
		if earnings == 0 {panic!("can't claim 0 tokens")}

		self.markets.insert(&market_id, &market);
		if market_creator_fee > 0 {
			return fun_token::transfer(account_id.to_string(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
				fun_token::transfer(market_creator, U128(market_creator_fee), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS)
			);
		} else {
			return fun_token::transfer(account_id.to_string(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		}
		
	}

	pub fn claim_affiliate_earnings(
		&mut self,
		account_id: String
	) -> Promise {
		let affiliate_earnings = self.affiliate_earnings.get(&account_id).expect("account doesn't have any affiliate fees to collect");
		if affiliate_earnings > 0 {
			self.affiliate_earnings.insert(&account_id, &0);
			return fun_token::transfer(account_id.to_string(), U128(affiliate_earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		} else {
			panic!("account doesn't have any affiliate fees to collect");
		}	
	}

	pub fn get_market_volume(
		&self,
		market_id: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		return self.markets
		.get(&market_id)
		.expect("market doesn't exist")
		.filled_volume.into();
	}

	pub fn dynamic_market_sell(
		&mut self,
		market_id: U64,
		outcome: U64,
		shares: U128,
	// ) -> Promise{
	) {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let shares: u128 = shares.into();

		assert!(shares > 0, "can't sell no shares");

		let mut market = self.markets.get(&market_id).expect("non existent market");
		let earnings = market.dynamic_market_sell_internal(outcome, shares);
		self.markets.insert(&market_id, &market);
		
		let market_creator_fee = earnings * market.creator_fee_percentage / 100;
		let resolution_fee = earnings * market.resolution_fee_percentage / 100;
		let fees = market_creator_fee + resolution_fee;

		fun_token::transfer(env::predecessor_account_id(), U128(earnings - fees), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
	}

	pub fn get_market_sell_depth(
		&self, 
		market_id: U64,
		outcome: U64,
		shares: U128,
	) -> (U128, U128) {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let shares: u128 = shares.into();

		let market = self.markets.get(&market_id).expect("non existent market");
		let (spendable, shares_fillabe) = market.get_dynamic_market_sell_offer(outcome, shares);

		return (spendable.into(), shares_fillabe.into());
	}

	pub fn get_outcome_share_balance(
		&self,
		account_id: String,
		market_id: U64,
		outcome: U64,
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();

		let market = self.markets.get(&market_id).expect("non existent market");
		let orderbook = market.orderbooks.get(&outcome).expect("non existent outcome");
		return orderbook.get_share_balance(account_id).into();
	}

	pub fn get_depth(
		&self, 
		market_id: U64, 
		outcome: U64, 
		spend: U128, 
		price: U128
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let spend: u128 = spend.into();
		let price: u128 = price.into();
		
		let market = self.markets.get(&market_id).unwrap();
		return market.get_liquidity_available(outcome, spend, price).into();
	}

	pub fn get_liquidity(
		&self, 
		market_id: U64, 
		outcome: U64, 
		price: U128
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let price: u128 = price.into();

		let market = self.markets.get(&market_id).unwrap();
		let orderbook = market.orderbooks.get(&outcome).unwrap();

		return orderbook.get_liquidity_at_price(price).into();
	}

	pub fn get_owner(
		&self
	) -> String {
		return self.creator.to_string();
	}

	pub fn get_market_price(
		&self, 
		market_id: U64, 
		outcome: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();

		let market = self.markets.get(&market_id).unwrap();
		return market.get_market_price_for(outcome).into();
	}

	pub fn is_promise_success(&self) -> bool {
		assert_eq!(
			env::promise_results_count(),
			1,
			"Contract expected a result on the callback"
		);
		match env::promise_result(0) {
			PromiseResult::Successful(_) => true,
			_ => false,
		}
	}
	
}

impl Default for Markets {
	fn default() -> Self {
		Self {
			creator: "flux-dev".to_string(),
			markets: UnorderedMap::new(b"markets".to_vec()),
			nonce: 0,
			max_fee_percentage: 5,
			creation_bond: 0,
			affiliate_earnings: UnorderedMap::new(b"affiliate_earnings".to_vec())
		}
	}
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use super::*;
	mod utils;
	use utils::{ntoy, ExternalUser, init_markets_contract};
    use near_sdk::MockedBlockchain;
    use near_sdk::{VMContext, testing_env};
	use near_runtime_standalone::{RuntimeStandalone};
	use near_primitives::transaction::{ExecutionStatus, ExecutionOutcome};

	fn to_dai(amt: u128) -> u128 {
		let base = 10 as u128;
		return amt * base.pow(17);
	}

	fn flux_protocol() -> String {
		return "flux_protocol".to_string();
	}

	fn judge() -> String {
		return "flux-dev".to_string();
	}

	fn affiliate() -> String {
		return "affiliate".to_string();
	}

	fn alice() -> String {
		return "alice.near".to_string();
	}

	fn carol() -> String {
		return "carol.near".to_string();
	}

	fn bob() -> String {
		return "bob.near".to_string();
	}

	fn empty_string() -> String {
		return "".to_string();
	}

	fn categories () -> Vec<String> {
		return vec![];
	}

	fn outcome_tags(
		number_of_outcomes: u64
	) -> Vec<String> {
		let mut outcomes: Vec<String> = vec![];
		for _ in 0..number_of_outcomes {
			outcomes.push(empty_string());
		}
		return outcomes;
	}

	fn current_block_timestamp() -> u64 {
		return 123789;
	}
	
	fn market_creation_timestamp() -> u64 {
		return 12378;
	}
	fn market_end_timestamp_ns() -> u64 {
		return 12379000000;
	}
	fn market_end_timestamp_ms() -> u64 {
		return 12379;
	}

	fn get_context(
		predecessor_account_id: String, 
		block_timestamp: u64
	) -> VMContext {

		VMContext {
			current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
			block_index: 0,
			epoch_height: 0,
            account_balance: 0,
			is_view: false,
            storage_usage: 0,
			block_timestamp: block_timestamp,
			account_locked_balance: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(12),
            random_seed: vec![0, 1, 2],
            output_data_receivers: vec![],
		}
	}

	fn init_runtime_env() -> (RuntimeStandalone, ExternalUser, Vec<ExternalUser>) {
		let (mut runtime, root) = init_markets_contract();


		let mut accounts: Vec<ExternalUser> = vec![];
		for acc_no in 0..2 {
			let acc = if let Ok(acc) =
				root.create_external(&mut runtime, format!("account_{}", acc_no), ntoy(100))
			{
				acc
			} else {
				break;
			};
			accounts.push(acc);
		}

		root.deploy_fun_token(&mut runtime, accounts[0].get_account_id(), U128(ntoy(100))).unwrap();

		return (runtime, root, accounts);
	}

	mod init_tests;
	mod market_order_tests;
	mod binary_order_matching_tests;
	mod categorical_market_tests;
	mod market_depth_tests;
	mod market_resolution_tests;
	mod claim_earnings_tests;
	mod market_dispute_tests;
	mod fee_payout_tests;
	mod order_sale_tests;
}

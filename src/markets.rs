use near_sdk::{near_bindgen, env, ext_contract, callback, Promise, PromiseOrValue, PromiseResult};
use near_sdk::json_types::{U128, U64};
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize};

mod market;
type Market = market::Market;
type Order = market::orderbook::order::Order;
type ResolutionWindow = market::ResolutionWindow;

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
struct Markets {
	creator: String,
	markets: BTreeMap<u64, Market>,
	nonce: u64,
	fdai_balances: HashMap<String, u128>, // Denominated in 1e18
	fdai_circulation: u128,
	fdai_in_protocol: u128,
	fdai_outside_escrow: u128,
	user_count: u64,
	max_fee_percentage: u128,
	creation_bond: u128,
	affiliate_earnings: HashMap<String, u128>
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
    // fn grant_fdai(&mut self, from: String);
    // fn check_sufficient_balance(&mut self, spend: U128);
    // fn update_fdai_metrics_claim(&mut self);
    // fn update_fdai_metrics_subtract(&mut self, amount: u128);
    // fn update_fdai_metrics_add(&mut self, amount: u128);
    // fn purchase_shares(&mut self, from: String, market_id: u64, outcome: u64, spend: U128, price: U128);
    // fn resolute_approved(&mut self, market_id: u64, winning_outcome: Option<u64>, stake: U128);
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

	// This is a demo method, it mints a currency to interact with markets until we have NDAI
	pub fn add_to_creators_funds(
		&mut self, 
		amount: u128
	) {
		let account_id = env::predecessor_account_id();
		assert_eq!(account_id, self.creator);

		*self.fdai_balances.get_mut(&account_id).unwrap() += amount;

		// Monitoring total supply - just for testnet
		self.fdai_circulation = self.fdai_circulation + amount as u128;
		self.fdai_outside_escrow = self.fdai_outside_escrow + amount as u128;
	}

	// This is a demo method, it mints a currency to interact with markets until we have NDAI
	pub fn claim_fdai(
		&mut self
	) {
		let can_claim = self.fdai_balances.get(&env::predecessor_account_id()).is_none();
		assert!(can_claim, "user has already claimed fdai");

		let claim_amount = 100 * self.dai_token();
		self.fdai_balances.insert(env::predecessor_account_id(), claim_amount);

		// Monitoring total supply - just for testnet
		self.fdai_circulation = self.fdai_circulation + claim_amount as u128;
		self.fdai_outside_escrow = self.fdai_outside_escrow + claim_amount as u128;
		self.user_count = self.user_count + 1;
	}

	pub fn get_fdai_balance(&self, account_id: String) -> u128 {
		return *self.fdai_balances.get(&account_id).unwrap_or(&0);
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
		self.markets.insert(self.nonce, new_market);
		self.nonce = self.nonce + 1;
		return market_id;
	}

	pub fn delete_market(
		&mut self,
		market_id: u64
	) {
		let account_id = env::predecessor_account_id();
		assert_eq!(account_id, self.creator, "markets can only be deleted by the market creator");
		self.markets.remove(&market_id);
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
		
		assert!(spend > 0, "order must be valued at > 0");
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

		let market = self.markets.get_mut(&market_id).unwrap();
		market.create_order(sender, outcome, amount_of_shares, spend, price, affiliate_account_id);
		return PromiseOrValue::Value(true);
	}

	pub fn cancel_order(
		&mut self, 
		market_id: U64, 
		outcome: U64, 
		order_id: U128
	) -> Promise {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let order_id: u128 = order_id.into();

		let market = self.markets.get_mut(&market_id).unwrap();
		assert_eq!(market.resoluted, false);
		let mut orderbook = market.orderbooks.get_mut(&outcome).unwrap();
		let order = orderbook.open_orders.get(&order_id).unwrap();
		assert!(env::predecessor_account_id() == order.creator);

		let to_return = orderbook.remove_order(order_id);
		env::log(format!("canceled order, refunding: {}", to_return).as_bytes());
		return fun_token::transfer(env::predecessor_account_id(), to_return.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
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
		let market = self.markets.get_mut(&market_id).expect("market doesn't exist");
		assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, false, "market is already resoluted");
		assert_eq!(market.finalized, false, "market is already finalized");
		assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");
		
		// this assertion shouldn't ever be needed because of the market.resolution check, 
		// TODO: confirm in tests
		// let resolution_window = market.resolution_windows.last_mut().expect("no resolute window exists, something went wrong at creation");
		// assert_eq!(resolution_window.round, 0, "can only resolute once"); 

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
		
		let market = self.markets.get_mut(&market_id).expect("market doesn't exist");
		let change: u128 = market.resolute(sender.to_string(), winning_outcome, stake).into();
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

		let market = self.markets.get_mut(&market_id).expect("invalid market");
		let to_return = market.cancel_dispute_participation(dispute_round, outcome);
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
        let market = self.markets.get_mut(&market_id).expect("market doesn't exist");

		assert_eq!(market.resoluted, true, "market isn't resoluted yet");
		assert_eq!(market.finalized, false, "market is already finalized");
        assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");
        assert!(winning_outcome != market.winning_outcome, "same oucome as last resolution");
		let resolution_window = market.resolution_windows.last_mut().expect("Invalid dispute window unwrap");
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
        let market = self.markets.get_mut(&market_id).expect("market doesn't exist");

		let change = market.dispute(sender.to_string(), winning_outcome, stake);

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

		let market = self.markets.get_mut(&market_id).unwrap();
		assert_eq!(market.resoluted, true, "market has to be resoluted before it can be finalized");
		if market.disputed {
			assert_eq!(env::predecessor_account_id(), self.creator, "only the judge can resolute disputed markets");
		} else {
			let dispute_window = market.resolution_windows.last().expect("no dispute window found, something went wrong");
			assert!(env::block_timestamp() / 1000000 >= dispute_window.end_time || dispute_window.round == 2, "dispute window still open")
		}

        market.finalize(winning_outcome);
	}

	fn subtract_balance(
		&mut self, 
		amount: u128
	) {
		let account_id = env::predecessor_account_id();
		let balance = self.fdai_balances.get(&account_id).unwrap();
		assert!(*balance >= amount, "sender has unsufficient balance");
		let new_balance = *balance - amount;
		self.fdai_balances.insert(account_id, new_balance);

		// For monitoring supply - just for testnet
		self.fdai_outside_escrow = self.fdai_outside_escrow - amount as u128;
		self.fdai_in_protocol= self.fdai_outside_escrow + amount as u128;
	}

	fn add_balance(
		&mut self, 
		amount: u128,
		account_id: String
	) {
		let one_dai = self.dai_token();
		self.fdai_balances.entry(account_id).and_modify(|balance| {
			*balance += amount;
		}).or_insert(100 * one_dai + amount);

		// For monitoring supply - just for testnet
		self.fdai_outside_escrow = self.fdai_outside_escrow + amount as u128;
		self.fdai_in_protocol= self.fdai_outside_escrow - amount as u128;
	}

	pub fn get_active_resolution_window(
		&self,
		market_id: u64
	) -> Option<&ResolutionWindow> {
		let market = self.markets.get(&market_id).expect("market doesn't exist");
		if !market.resoluted {
			return None;
		}
		return Some(market.resolution_windows.last().expect("invalid dispute window"));

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
	// ) -> Promise {
	) {
		let market_id: u64 = market_id.into();
		let market = self.markets.get_mut(&market_id).expect("market doesn't exist");
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
			self.affiliate_earnings
			.entry(affiliate_account_id)
			.and_modify(|balance| {
				*balance += amount_owed;
			})
			.or_insert(amount_owed);
		}

		
		let earnings = winnings - market_creator_fee - paid_to_affiliates - resolution_fee + governance_earnings + left_in_open_orders;
		if earnings == 0 {panic!("can't claim 0 tokens")}
		// make sure that everything is claimed successfuly in test coverage
		// Best way to solve this is have both these transfers in batch - if one fails both shoud revert.
		// If reverted this tx should revert all changes too 

		if market_creator_fee > 0 {
			fun_token::transfer(account_id.to_string(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
				fun_token::transfer(account_id.to_string(), U128(market_creator_fee), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS)
			);
		} else {
			fun_token::transfer(account_id.to_string(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		}
		
	}

	// TODO
	pub fn claim_affiliate_earnings(
		&mut self,
		account_id: String
	) {

	}

	pub fn get_all_markets(
		&self
	) -> &BTreeMap<u64, Market> {
		return &self.markets;
	}

	pub fn get_markets_by_id(
		&self, 
		market_ids: Vec<u64>
	) -> BTreeMap<u64, &Market> {
		let mut markets = BTreeMap::new();
		for market_id in market_ids {
			markets.insert(market_id, self.markets.get(&market_id).unwrap());
		}
		return markets;
	}

	pub fn get_specific_markets(
		&self, 
		market_ids: Vec<u64>
	) -> BTreeMap<u64, &Market> {
		let mut markets = BTreeMap::new();
		for market_id in 0..market_ids.len() {
			markets.insert(market_id as u64, self.markets.get(&(market_id as u64)).unwrap());
		}
		return markets;
	}
	
	fn dynamic_market_sell(
		&mut self,
		market_id: u64,
		outcome: u64,
		shares: u128,
	) {
		assert!(shares > 0, "can't sell no shares");
		let market = self.markets.get_mut(&market_id).expect("non existent market");
		let earnings = market.dynamic_market_sell(outcome, shares);
		self.add_balance(earnings, env::predecessor_account_id());
	}

	fn get_market_sell_depth(
		&self, 
		market_id: u64,
		outcome: u64,
		shares: u128,
	) -> (u128, u128) {
		let market = self.markets.get(&market_id).expect("non existent market");
		return market.get_dynamic_market_sell_offer(outcome, shares);
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

	pub fn get_market(
		&self, 
		id: u64
	) -> &Market {
		let market = self.markets.get(&id);
		return market.unwrap();
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

	pub fn get_best_prices(
		&self, 
		market_id: u64
	) -> BTreeMap<u64, u128> {
		let market = self.markets.get(&market_id).unwrap();
		return market.get_market_prices_for();
	}

	pub fn get_fdai_metrics(
		&self
	) -> (u128, u128, u128, u64) {
		return (self.fdai_circulation, self.fdai_in_protocol, self.fdai_outside_escrow, self.user_count);
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
			markets: BTreeMap::new(),
			nonce: 0,
			fdai_balances: HashMap::new(),
			fdai_circulation: 0,
			fdai_in_protocol: 0,
			fdai_outside_escrow: 0,
			user_count: 0,
			max_fee_percentage: 5,
			creation_bond: 0,
			affiliate_earnings: HashMap::new()
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

	// mod init_tests;
	// mod market_order_tests;
	// mod binary_order_matching_tests;
	// mod categorical_market_tests;
	// mod market_depth_tests;
	// mod claim_earnings_tests;
	mod market_dispute_tests;
	// mod market_resolution_tests;
	// mod fee_payout_tests;
	// mod order_sale_tests;
}

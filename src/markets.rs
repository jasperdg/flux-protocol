use near_sdk::{
	near_bindgen, 
	env, 
	ext_contract, 
 	Promise, 
	PromiseOrValue, 
	json_types::{U128, U64},
	PromiseResult,
	collections::{
		UnorderedMap,
		Vector,
	}
};
use borsh::{BorshDeserialize, BorshSerialize};
use serde_json::json;

use crate::market;
use crate::order;

type Market = market::Market;
type Order = order::Order;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Markets {
	creator: String,
	markets: UnorderedMap<u64, Market>,
	nonce: u64,
	max_fee_percentage: u128,
	creation_bond: u128,
	affiliate_earnings: UnorderedMap<String, u128>,
	fun_token_account_id: String,
}

const SINGLE_CALL_GAS: u64 = 100000000000000;

#[ext_contract]
pub trait FunToken {
    fn transfer_from(&mut self, owner_id: String, new_owner_id: String, amount: U128);
    fn transfer(&mut self, new_owner_id: String, amount: U128);
    fn get_total_supply(&self) -> u128;
    fn get_balance(&self, owner_id: AccountId) -> u128;
}


#[ext_contract]
pub trait FluxProtocol {
    fn market_creation(&mut self, sender: String, market_id: u64, outcome: u64, amount_of_shares: u128, spend: u128, price: u128, affiliate_account_id: Option<String>);
    fn proceed_order_placement(&mut self, sender: String, market_id: u64, outcome: u64, shares: u128, spend: u128, price: u128, affiliate_account_id: Option<String>);
    fn proceed_market_resolution(&mut self, sender: String, market_id: u64, winning_outcome: Option<u64>, stake: u128);
	fn proceed_market_dispute(&mut self, sender: String, market_id: u64, winning_outcome: Option<u64>, stake: u128);
	fn proceed_market_creation(&mut self, sender: String, description: String, extra_info: String, outcomes: u64, outcome_tags: Vec<String>, categories: Vec<String>, end_time: u64, creator_fee_percentage: u128, resolution_fee_percentage: u128, affiliate_fee_percentage: u128, api_source: String);
}

impl Default for Markets {
    fn default() -> Self {
        panic!("Flux protocol should be initialized before usage")
    }
}

#[near_bindgen]
impl Markets {

	#[init]
	pub fn init(fun_token_account_id: String) -> Self {
		Self {
			creator: "flux-dev".to_string(),
			markets: UnorderedMap::new(b"markets".to_vec()),
			nonce: 0,
			max_fee_percentage: 500,
			creation_bond: 25e18 as u128 / 100,
			affiliate_earnings: UnorderedMap::new(b"affiliate_earnings".to_vec()), 
			fun_token_account_id
		}
	}

	fn dai_token(
		&self
	) -> u128 {
		let base: u128 = 10;
		return base.pow(18)
	}

	fn fun_token_account_id(
		&self
	) -> String {
		return self.fun_token_account_id.to_string();
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
	) -> Promise {
	// ) {
		let outcomes: u64 = outcomes.into();
		let end_time: u64 = end_time.into();
		let creator_fee_percentage: u128 = creator_fee_percentage.into();
		let affiliate_fee_percentage: u128 = affiliate_fee_percentage.into();

		for outcome_tag in &outcome_tags {
			assert!(outcome_tag.chars().count() < 20, "outcome tag can't be more than 20 chars");
		}

		for category in &categories {
			assert!(category.chars().count() < 20, "category tag can't be more than 20 chars");
		}

		assert!(description.chars().count() < 201, "description can't than 200 characters");
		assert!(extra_info.chars().count() < 401, "extra_info can't than 400 characters");
		assert!(outcomes > 1, "need to have more than 2 outcomes");
		assert!(outcomes == 2 || outcomes == outcome_tags.len() as u64, "invalid outcomes");
		assert!(outcomes < 20, "can't have more than 8 outcomes"); // up for change
		assert!(end_time > env::block_timestamp() / 1000000, "end_time has to be greater than NOW");
		assert!(categories.len() < 6, "can't have more than 6 categories");
		assert!(creator_fee_percentage <= self.max_fee_percentage, "creator_fee_percentage too high");
		assert!(affiliate_fee_percentage <= 100, "affiliate_fee_percentage can't be higher than 100");

		if outcomes == 2 {assert!(outcome_tags.len() == 0)}

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), self.creation_bond.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
			flux_protocol::proceed_market_creation(
				env::predecessor_account_id(), 
				description,
				extra_info,
				outcomes,
				outcome_tags,
				categories,
				end_time,
				creator_fee_percentage, 
				100,
				affiliate_fee_percentage,
				api_source,
				&env::current_account_id(),
				0,
				SINGLE_CALL_GAS
			)
		);
	}

	pub fn proceed_market_creation(
		&mut self, 
		sender: String, 
		description: String, 
		extra_info: String, 
		outcomes: u64, 
		outcome_tags: Vec<String>, 
		categories: Vec<String>, 
		end_time: u64, 
		creator_fee_percentage: u128, 
		resolution_fee_percentage: u128, 
		affiliate_fee_percentage: u128, 
		api_source: String
	) -> PromiseOrValue<u64> {
		self.assert_self();
		
		let transfer_succeeded = self.is_promise_success();
		if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", self.creation_bond, env::current_account_id()); }

		env::log(
			json!({
				"type": "market_creation".to_string(),
				"params": {
					"id": U64(self.nonce),
					"creator": sender,
					"description": description,
					"extra_info": extra_info,
					"outcomes": U64(outcomes),
					"outcome_tags": outcome_tags,
					"categories": categories,
					"end_time": U64(end_time),
					"creator_fee_percentage": U128(creator_fee_percentage),
					"resolution_fee_percentage": U128(resolution_fee_percentage),
					"affiliate_fee_percentage": U128(affiliate_fee_percentage),
					"api_source": api_source,
				}
			})
			.to_string()
			.as_bytes()
		);

		let new_market = Market::new(self.nonce, sender, description, extra_info, outcomes, outcome_tags, categories, end_time, creator_fee_percentage, resolution_fee_percentage, affiliate_fee_percentage ,api_source);
		let market_id = new_market.id;
		self.markets.insert(&self.nonce, &new_market);
		self.nonce = self.nonce + 1;
		
		
		return PromiseOrValue::Value(market_id);
	}

	pub fn place_order(
		&mut self, 
		market_id: U64, 
		outcome: U64,
		shares: U128,
		price: U128,
		affiliate_account_id: Option<String>
	) -> Promise {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let price: u128 = price.into();
		let shares: u128 = shares.into();

		let rounded_spend = shares * price;

		let market = self.markets.get(&market_id).expect("market doesn't exist");

		assert!(rounded_spend >= 10000, "order must be valued at > 10000");
		assert!(price > 0 && price < 100, "price can only be between 0 - 100");
		assert!(outcome < market.outcomes, "invalid outcome");
		assert_eq!(market.resoluted, false, "market has already been resoluted");
		assert!(env::block_timestamp() / 1000000 < market.end_time, "market has already ended");
		

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), rounded_spend.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 10)
		.then(
			flux_protocol::proceed_order_placement( 
				env::predecessor_account_id(),
				market_id,
				outcome,
				shares,
				rounded_spend,
				price,
				affiliate_account_id,
				&env::current_account_id(), 
				0, 
				SINGLE_CALL_GAS * 2 - SINGLE_CALL_GAS / 10
			)
		);
	}

	pub fn proceed_order_placement(
		&mut self,
		sender: String,
		market_id: u64, 
		outcome: u64,
		shares: u128,
		spend: u128, 
		price: u128,
		affiliate_account_id: Option<String>,
	) -> PromiseOrValue<bool> {
		self.assert_self();
		
		let transfer_succeeded = self.is_promise_success();
		if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", spend, env::current_account_id()); }
		
		let mut market = self.markets.get(&market_id).unwrap();
		market.place_order_internal(sender, outcome, shares, spend, price, affiliate_account_id);
		self.markets.insert(&market.id, &market);
		return PromiseOrValue::Value(true);
	}

	pub fn cancel_order(
		&mut self, 
		market_id: U64, 
		outcome: U64,
		price: U128,
		order_id: U128
	) {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let order_id: u128 = order_id.into();
		let price: u128 = price.into();
		
		let mut market = self.markets.get(&market_id).unwrap();
		assert_eq!(market.resoluted, false);
		let mut orderbook = market.orderbooks.get(&outcome).unwrap();
		let price_data = orderbook.price_data.get(&price).expect("order at this price doesn't exist");
		let order = price_data.orders.get(&order_id).expect("order with this id doesn't exist");
		assert!(env::predecessor_account_id() == order.creator, "not this user's order");

		let to_return = orderbook.cancel_order(order);
		market.orderbooks.insert(&outcome, &orderbook);
		self.markets.insert(&market_id, &market);
		fun_token::transfer(env::predecessor_account_id(), to_return.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
    }

	// pub fn resolute_market(
	// 	&mut self, 
	// 	market_id: U64, 
	// 	winning_outcome: Option<U64>,
	// 	stake: U128
	// ) -> Promise {
	// 	let market_id: u64 = market_id.into();
	// 	let winning_outcome: Option<u64> = match winning_outcome {
	// 		Some(outcome) => Some(outcome.into()),
	// 		None => None
	// 	};
	// 	let stake_u128: u128 = stake.into();
	// 	let market = self.markets.get(&market_id).expect("market doesn't exist");
	// 	assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
	// 	assert_eq!(market.resoluted, false, "market is already resoluted");
	// 	assert_eq!(market.finalized, false, "market is already finalized");
	// 	assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");

	// 	return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2)
	// 	.then(
	// 		flux_protocol::proceed_market_resolution(
	// 			env::predecessor_account_id(),
	// 			market_id,
	// 			winning_outcome,
	// 			stake_u128,
	// 			&env::current_account_id(),
	// 			0,
	// 			SINGLE_CALL_GAS
	// 		)
	// 	);
	// }

	// pub fn proceed_market_resolution(
	// 	&mut self,
	// 	market_id: u64,
	// 	winning_outcome: Option<u64>,
	// 	stake: u128,
	// 	sender: String
	// ) -> PromiseOrValue<bool> {
	// 	self.assert_self();
	// 	let transfer_succeeded = self.is_promise_success();
	// 	if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", stake, env::current_account_id()); }
		
	// 	let mut market = self.markets.get(&market_id).expect("market doesn't exist");
	// 	let change: u128 = market.resolute(sender.to_string(), winning_outcome, stake).into();
	// 	self.markets.insert(&market_id, &market);
	// 	if change > 0 {
	// 		let prom = fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2);
	// 		return PromiseOrValue::Promise(prom);
	// 	} else {
	// 		return PromiseOrValue::Value(true);
	// 	}
	// }

	// pub fn withdraw_dispute_stake(
	// 	&mut self, 
	// 	market_id: U64,
	// 	dispute_round: U64,
	// 	outcome: Option<U64>
	// ) -> Promise {
	// 	let market_id: u64 = market_id.into();
	// 	let dispute_round: u64 = dispute_round.into();
	// 	let outcome: Option<u64> = match outcome {
	// 		Some(outcome) => Some(outcome.into()),
	// 		None => None
	// 	};

	// 	let mut market = self.markets.get(&market_id).expect("invalid market");
	// 	let to_return = market.cancel_dispute_participation(dispute_round, outcome);
	// 	self.markets.insert(&market_id, &market);
	// 	if to_return > 0 {
	// 		env::log(
	// 			json!({
	// 				"type": "withdrawn_unbounded_dispute_stake".to_string(),
	// 				"params": {
	// 					"market_id": U64(market_id),
	// 					"sender": env::predecessor_account_id(),
	// 					"dispute_round": U64(dispute_round),
	// 					"outcome": outcome,
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);
	// 		return fun_token::transfer(env::predecessor_account_id(), U128(to_return), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
	// 	} else {
	// 		panic!("user has no participation in this dispute");
	// 	}
	// }

	// pub fn dispute_market(
	// 	&mut self, 
	// 	market_id: U64, 
	// 	winning_outcome: Option<U64>,
	// 	stake: U128
	// ) -> Promise {
	// 	let market_id: u64 = market_id.into();
	// 	let winning_outcome: Option<u64> = match winning_outcome {
	// 		Some(outcome) => Some(outcome.into()),
	// 		None => None
	// 	};
	// 	let stake_u128: u128 = stake.into();
    //     let market = self.markets.get(&market_id).expect("market doesn't exist");

	// 	assert_eq!(market.resoluted, true, "market isn't resoluted yet");
	// 	assert_eq!(market.finalized, false, "market is already finalized");
    //     assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");
    //     assert!(winning_outcome != market.winning_outcome, "same oucome as last resolution");
	// 	let resolution_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("Invalid dispute window unwrap");
	// 	assert_eq!(resolution_window.round, 1, "for this version, there's only 1 round of dispute");
	// 	assert!(env::block_timestamp() / 1000000 <= resolution_window.end_time, "dispute window is closed, market can be finalized");

	// 	return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2).then(
	// 		flux_protocol::proceed_market_dispute(
	// 			env::predecessor_account_id(),
	// 			market_id,
	// 			winning_outcome,
	// 			stake_u128,
	// 			&env::current_account_id(), 
	// 			0, 
	// 			SINGLE_CALL_GAS
	// 		)
	// 	)
	// }

	// pub fn proceed_market_dispute(		
	// 	&mut self,
	// 	market_id: u64,
	// 	winning_outcome: Option<u64>,
	// 	stake: u128,
	// 	sender: String
	// ) -> PromiseOrValue<bool> {
	// 	self.assert_self();
	// 	let transfer_succeeded = self.is_promise_success();
	// 	if !transfer_succeeded { panic!("transfer failed, make sure the user has a higher balance than: {} and sufficient allowance set for {}", stake, env::current_account_id()); }
    //     let mut market = self.markets.get(&market_id).expect("market doesn't exist");

	// 	let change = market.dispute(sender.to_string(), winning_outcome, stake);

	// 	self.markets.insert(&market.id, &market);
	// 	if change > 0 {
	// 		return PromiseOrValue::Promise(fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2));
	// 	} else {
	// 		return PromiseOrValue::Value(true);
	// 	}
	// }
		

	// pub fn finalize_market(
	// 	&mut self, 
	// 	market_id: U64, 
	// 	winning_outcome: Option<U64>
	// ) {
	// 	let market_id: u64 = market_id.into();
	// 	let winning_outcome: Option<u64> = match winning_outcome {
	// 		Some(outcome) => Some(outcome.into()),
	// 		None => None
	// 	};

	// 	let mut market = self.markets.get(&market_id).unwrap();
	// 	assert_eq!(market.resoluted, true, "market has to be resoluted before it can be finalized");
	// 	if market.disputed {
	// 		assert_eq!(env::predecessor_account_id(), self.creator, "only the judge can resolute disputed markets");
	// 	} else {
	// 		let dispute_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("no dispute window found, something went wrong");
	// 		assert!(env::block_timestamp() / 1000000 >= dispute_window.end_time || dispute_window.round == 2, "dispute window still open")
	// 	}

	// 	market.finalize(winning_outcome);
	// 	self.markets.insert(&market_id, &market);
	// }

	// pub fn get_open_orders_len(
	// 	&self, 
	// 	market_id: U64, 
	// 	outcome: U64
	// ) -> U128 {
	// 	let market_id: u64 = market_id.into();
	// 	let outcome: u64 = outcome.into();

	// 	let market = self.markets.get(&market_id).unwrap();
	// 	let orderbook = market.orderbooks.get(&outcome).unwrap();
	// 	return U128(orderbook.open_orders.len() as u128);
	// }

	// pub fn get_filled_orders_len(
	// 	&self, 
	// 	market_id: U64, 
	// 	outcome: U64
	// ) -> U128 {
	// 	let market_id: u64 = market_id.into();
	// 	let outcome: u64 = outcome.into();

	// 	let market = self.markets.get(&market_id).unwrap();
	// 	let orderbook = market.orderbooks.get(&outcome).unwrap();
	// 	return U128(orderbook.filled_orders.len() as u128);
	// }

	// pub fn get_claimable(
	// 	&self, 
	// 	market_id: U64, 
	// 	account_id: String
	// ) -> U128 {
		
	// 	let market_id: u64 = market_id.into();
	// 	let market = self.markets.get(&market_id).expect("market doesn't exist");
	// 	let claimed_earnings = market.claimed_earnings.get(&account_id);
	// 	assert_eq!(claimed_earnings.is_none(), true, "user already claimed earnings");
	// 	if claimed_earnings.is_some() { return U128(0); }
	// 	let mut validity_bond = 0;
	// 	if account_id == market.creator && market.validity_bond_claimed == false && market.winning_outcome != None {
	// 		validity_bond = self.creation_bond;
	// 	}
	// 	let (winnings, left_in_open_orders, governance_earnings, _) = market.get_claimable_for(account_id.to_string());
	// 	let total_fee_percentage = market.creator_fee_percentage + market.resolution_fee_percentage;
	// 	let fee = (winnings * total_fee_percentage + 10000 - 1) / 10000;
		
	// 	return (winnings - fee + governance_earnings + left_in_open_orders + validity_bond).into();
	// }

	// pub fn claim_earnings(
	// 	&mut self, 
	// 	market_id: U64, 
	// 	account_id: String
	// ) -> Promise {
	// 	let market_id: u64 = market_id.into();
	// 	let mut market = self.markets.get(&market_id).expect("market doesn't exist");
	// 	let market_creator = market.creator.to_string();
	// 	let claimed_earnings = market.claimed_earnings.get(&account_id);
	// 	assert_eq!(claimed_earnings.is_none(), true, "user already claimed earnings");
	// 	assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
	// 	assert_eq!(market.resoluted, true, "market isn't resoluted yet");
	// 	assert_eq!(market.finalized, true, "market isn't finalized yet");

	// 	market.claimed_earnings.insert(&account_id, &true);
	// 	let (winnings, left_in_open_orders, governance_earnings, affiliates) = market.get_claimable_for(account_id.to_string());
	// 	let mut market_creator_fee = (winnings * market.creator_fee_percentage + 10000 - 1) / 10000;
	// 	let creator_fee_percentage = market.creator_fee_percentage;
	// 	let resolution_fee = (winnings * market.resolution_fee_percentage + 10000 - 1) / 10000;
	// 	let affiliate_fee_percentage = market.affiliate_fee_percentage;
	// 	let mut paid_to_affiliates = 0;
		
	// 	let mut validity_bond = 0;
	// 	if account_id == market.creator && market.validity_bond_claimed == false && market.winning_outcome != None {
	// 		validity_bond = self.creation_bond;
	// 		market.validity_bond_claimed = true;
	// 	}

	// 	for (affiliate_account_id, amount_owed) in affiliates {	
	// 		let affiliate_owed = (amount_owed * affiliate_fee_percentage * creator_fee_percentage + 1000000 - 1) / 1000000;
	// 		paid_to_affiliates += affiliate_owed;
	// 		let affiliate_earnings = self.affiliate_earnings.get(&affiliate_account_id).unwrap_or(0);

	// 		env::log(
	// 			json!({
	// 				"type": "added_to_affiliate_earnings".to_string(),
	// 				"params": {
	// 					"market_id": U64(market_id),
	// 					"affiliate": affiliate_account_id,
	// 					"earned": U128(affiliate_earnings + affiliate_owed),
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);
	// 		market_creator_fee -= affiliate_owed;
	// 		self.affiliate_earnings.insert(&affiliate_account_id, &(affiliate_earnings + affiliate_owed));
	// 	}

	// 	let total_fee = market_creator_fee + paid_to_affiliates + resolution_fee;
	// 	let to_claim = winnings + governance_earnings + left_in_open_orders;

	// 	let earnings = to_claim - total_fee + validity_bond;
		
	// 	if earnings == 0 {panic!("can't claim 0 tokens")}

	// 	env::log(
	// 		json!({
	// 			"type": "earnings_claimed".to_string(),
	// 			"params": {
	// 				"market_id": U64(market_id),
	// 				"account_id": account_id,
	// 				"earned": U128(earnings),
	// 			}
	// 		})
	// 		.to_string()
	// 		.as_bytes()
	// 	);		
		
	// 	self.markets.insert(&market_id, &market);
	// 	if market_creator_fee > 0 {
	// 		return fun_token::transfer(account_id.to_string(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
	// 			fun_token::transfer(market_creator, U128(market_creator_fee), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS)
	// 		);
	// 	} else {
	// 		return fun_token::transfer(account_id.to_string(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
	// 	}
		
	// }

	pub fn claim_affiliate_earnings(
		&mut self,
		account_id: String
	) -> Promise {
		let affiliate_earnings = self.affiliate_earnings.get(&account_id).expect("account doesn't have any affiliate fees to collect");
		if affiliate_earnings > 0 {
			env::log(
				json!({
					"type": "affiliate_earnings_claimed".to_string(),
					"params": {
						"account_id": account_id,
						"earned": U128(affiliate_earnings),
					}
				})
				.to_string()
				.as_bytes()
			);		
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

	pub fn get_market_price(
		&self,
		market_id: U64,
		outcome: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		return U128(self.markets
			.get(&market_id)
			.expect("market doesn't exist")
			.get_market_price(outcome));
	}

	// pub fn dynamic_market_sell(
	// 	&mut self,
	// 	market_id: U64,
	// 	outcome: U64,
	// 	shares: U128,
	// // ) -> Promise{
	// ) {
	// 	let market_id: u64 = market_id.into();
	// 	let outcome: u64 = outcome.into();
	// 	let shares: u128 = shares.into();
		
	// 	assert!(shares > 0, "can't sell no shares");
		
	// 	let mut market = self.markets.get(&market_id).expect("non existent market");
	// 	let has_claimed = market.claimed_earnings.get(&env::predecessor_account_id());
	// 	assert_eq!(has_claimed.is_none(), true, "can't sell shares after claim");
	// 	let earnings = market.dynamic_market_sell_internal(outcome, shares);
	// 	assert!(earnings > 0, "no matching orders");
	// 	self.markets.insert(&market_id, &market);
		
	// 	let market_creator_fee = earnings * market.creator_fee_percentage / 10000;
	// 	let resolution_fee = earnings * market.resolution_fee_percentage / 10000;
	// 	let fees = market_creator_fee + resolution_fee;

	// 	fun_token::transfer(env::predecessor_account_id(), U128(earnings - fees), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
	// }

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
		let user_data = orderbook.user_data.get(&account_id);

		if user_data.is_none() {return U128(0)}

		return U128(user_data.unwrap().balance);
	}

	pub fn get_owner(
		&self
	) -> String {
		return self.creator.to_string();
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
		return amt * 1e18 as u128;
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

		root.deploy_fun_token(&mut runtime, accounts[0].get_account_id(), U128(to_dai(100))).unwrap();

		return (runtime, root, accounts);
	}

	mod binary_order_matching_tests;
	mod categorical_market_tests;
	mod init_tests; 
	mod market_order_tests;
	
	// mod market_depth_tests;
	// mod market_resolution_tests; 
	// mod claim_earnings_tests;
	// mod market_dispute_tests;
	// mod fee_payout_tests;
	// mod order_sale_tests; 
	// mod validity_bond_tests;
}

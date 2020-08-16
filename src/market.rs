use std::cmp;
use near_sdk::{
	near_bindgen, 
	env,
	json_types::{
		U64,
		U128
	},
	collections::{
		UnorderedMap,
		TreeMap,
		Vector
	}
};
use serde_json::json;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ResolutionWindow {
	pub round: u64,
	pub participants_to_outcome_to_stake: UnorderedMap<String, UnorderedMap<u64, u128>>, // Account to outcome to stake
	pub required_bond_size: u128,
	pub staked_per_outcome: UnorderedMap<u64, u128>, // Staked per outcome
	pub end_time: u64,
	pub outcome: Option<u64>,
}

use crate::orderbook::{
	Orderbook
};


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
	pub id: u64,
	pub description: String,
	pub extra_info: String,
	pub creator: String,
	pub outcomes: u64,
	pub outcome_tags: Vector<String>,
	pub categories: Vector<String>,
	pub creation_time: u64,
	pub end_time: u64,
	pub orderbooks: UnorderedMap<u64, Orderbook>,
	pub winning_outcome: Option<u64>, // invalid has outcome id: self.outcomes
	pub resoluted: bool,
	pub resolute_bond: u128,
	pub filled_volume: u128,
	pub disputed: bool,
	pub finalized: bool,
	pub creator_fee_percentage: u128,
	pub resolution_fee_percentage: u128,
	pub affiliate_fee_percentage: u128,
	pub claimable_if_valid: UnorderedMap<String, u128>,
	pub api_source: String,
	pub resolution_windows: Vector<ResolutionWindow>,
	pub validity_bond_claimed: bool,
	pub claimed_earnings: UnorderedMap<String, bool>
}

impl Market {
	pub fn new(
		id: u64, 
		account_id: String, 
		description: String, 
		extra_info: String, 
		outcomes: u64, 
		outcome_tags: Vec<String>, 
		categories: Vec<String>, 
		end_time: u64, 
		creator_fee_percentage: u128, 
		resolution_fee_percentage: u128, 
		affiliate_fee_percentage: u128,
		api_source: String,
	) -> Self {

		let mut outcome_tags_vector: Vector<String> = Vector::new(format!("market:{}:outcome_tags", id).as_bytes().to_vec());
		let mut categories_vector: Vector<String> = Vector::new(format!("market:{}:categories", id).as_bytes().to_vec());

		for outcome_tag in &outcome_tags {
			assert!(outcome_tag.chars().count() < 20, "outcome tag can't be more than 20 chars");
			outcome_tags_vector.push(outcome_tag);
		}

		for category in &categories {
			categories_vector.push(category);
		}

		let mut empty_orderbooks = UnorderedMap::new(format!("market:{}:orderbooks", id).as_bytes().to_vec());

		for i in 0..outcomes {
			empty_orderbooks.insert(&i, &Orderbook::new(id, i));
		}

		let base: u128 = 10;
		let mut resolution_windows = Vector::new("market:{}:resolution_windows".as_bytes().to_vec());
		let base_resolution_window = ResolutionWindow {
			round: 0,
			participants_to_outcome_to_stake: UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:0", id).as_bytes().to_vec()),
			required_bond_size: 5 * base.pow(18),
			staked_per_outcome: UnorderedMap::new(format!("market:{}:staked_per_outcome:{}", id, 0).as_bytes().to_vec()), // Staked per outcome
			end_time: end_time,
			outcome: None,
		};

		resolution_windows.push(&base_resolution_window);

		env::log(
			json!({
				"type": "new_resolution_window".to_string(),
				"params": {
					"market_id": U64(id),
					"round": U64(base_resolution_window.round),
					"required_bond_size": U128(base_resolution_window.required_bond_size),
					"end_time": U64(base_resolution_window.end_time),
				}
			})
			.to_string()
			.as_bytes()
		);

		Self {
			id,
			description,
			extra_info,
			creator: account_id,
			outcomes,
			outcome_tags: outcome_tags_vector,
			categories: categories_vector,
			creation_time: env::block_timestamp() / 1000000,
			end_time,
			orderbooks: empty_orderbooks,
			winning_outcome: None,
			resoluted: false,
			resolute_bond: 5 * base.pow(18),
			filled_volume: 0,
			disputed: false,
			finalized: false,
			creator_fee_percentage,
			resolution_fee_percentage,
			affiliate_fee_percentage,
			claimable_if_valid: UnorderedMap::new(format!("market:{}:claimable_if_valid", id).as_bytes().to_vec()),
			api_source,
			resolution_windows,
			validity_bond_claimed: false,
			claimed_earnings: UnorderedMap::new(format!("market:{}:claimed_earnings_for", id).as_bytes().to_vec()),
		}
	}

	// pub fn dynamic_market_sell_internal(
	// 	&mut self,
	// 	outcome: u64,
	// 	shares_to_sell: u128
	// ) -> u128 {
	// 	let mut orderbook = self.orderbooks.get(&outcome).unwrap();
	// 	let share_balance = orderbook.get_share_balance(env::predecessor_account_id());
	// 	let mut claimable_if_valid = 0 ;
	// 	assert!(shares_to_sell <= share_balance, "user doesn't have enough balance to sell these shares");
	// 	let mut best_price = orderbook.best_price.unwrap_or(0);
		
	// 	if best_price == 0 { return 0; }

	// 	let mut liq_at_price = orderbook.liquidity_by_price.get(&best_price).expect("no liquidity");
	// 	let mut spendable = 0;
	// 	let mut shares_fillable = shares_to_sell;

	// 	while best_price > 0 && shares_fillable > 0 {
	// 		let shares_sought = liq_at_price / best_price;
	// 		if shares_sought > shares_fillable {
	// 			spendable += shares_fillable * best_price;
	// 			claimable_if_valid += orderbook.subtract_shares(shares_fillable, best_price);
	// 			let volume_filled = orderbook.fill_best_orders(shares_to_sell);
	// 			self.filled_volume += volume_filled;
	// 		} else {
	// 			shares_fillable -= shares_sought;
	// 			spendable += liq_at_price;
	// 			claimable_if_valid += orderbook.subtract_shares(shares_sought, best_price);
	// 		}
			
	// 		let next_price = orderbook.liquidity_by_price.lower(&best_price).unwrap_or(0);
	// 		best_price = next_price;
	// 		liq_at_price = orderbook.liquidity_by_price.get(&best_price).unwrap_or(0);
	// 	}

	// 	if claimable_if_valid > 0 {
	// 		let claimable_if_valid_before = self.claimable_if_valid.get(&env::predecessor_account_id()).unwrap_or(0);
	// 		self.claimable_if_valid.insert(&env::predecessor_account_id(), &(claimable_if_valid + claimable_if_valid_before));
	// 		env::log(
	// 			json!({
	// 				"type": "increased_claimable_if_valid".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"sender": env::predecessor_account_id(),
	// 					"claimable_if_valid": U128(claimable_if_valid),
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);	
	// 	}

	// 	let volume_filled = orderbook.fill_best_orders(shares_to_sell - shares_fillable);
	// 	self.orderbooks.insert(&outcome, &orderbook);
	// 	self.filled_volume += volume_filled;

	// 	return spendable - claimable_if_valid;
	// }

	// pub fn get_dynamic_market_sell_offer(
	// 	&self, 
	// 	outcome: u64,
	// 	shares_to_sell: u128
	// ) -> (u128, u128) {
	// 	let orderbook = self.orderbooks.get(&outcome).unwrap();
	// 	let mut best_price = orderbook.best_price.unwrap_or(0);
	// 	if best_price == 0 { return (0, 0); }
	// 	let mut liq_at_price = orderbook.liquidity_by_price.get(&best_price).expect("no liquidity");
	// 	let mut spendable = 0;
	// 	let mut shares_fillable = shares_to_sell;

	// 	while best_price > 0 && shares_fillable > 0 {
	// 		let shares_sought = liq_at_price / best_price;
	// 		if shares_sought > shares_fillable {
	// 			spendable += shares_fillable * best_price;
	// 			return (spendable, 0);
	// 		} else {
	// 			shares_fillable -= shares_sought;
	// 			spendable += liq_at_price;
	// 		}
					
	// 		let next_price = orderbook.liquidity_by_price.lower(&best_price).unwrap_or(0);
	// 		best_price = next_price;
	// 		liq_at_price = orderbook.liquidity_by_price.get(&next_price).unwrap_or(0);
	// 	}

	// 	return (spendable, shares_to_sell - shares_fillable);
	// }

	pub fn place_order_internal(
		&mut self, 
		account_id: String, 
		outcome: u64, 
		shares: u128, 
		spend: u128, 
		price: u128,
		affiliate_account_id: Option<String>
	) {
		let (spent, shares_filled) = self.fill_matches(outcome, spend, price);
		env::log(format!("we spent {} on {:?} shares", spent, shares_filled).to_string().as_bytes());

		self.filled_volume += shares_filled * 100;
		let mut orderbook = self.orderbooks.get(&outcome).unwrap();
		orderbook.new_order(
			account_id,
			outcome,
			spend,
			shares,
			price,
			spent,
			shares_filled,
			affiliate_account_id,
		);
		self.orderbooks.insert(&outcome, &orderbook);
	}

	fn fill_matches(
		&mut self, 
		outcome: u64,
		to_spend: u128, 
		price: u128
	) -> (u128, u128) {
		let (mut market_price, mut share_depth) = self.get_market_price_and_min_liquidty(outcome);
		env::log(format!("avail liquidity for a market price of {} there is minimal liquidity of: {:?}", market_price, share_depth).to_string().as_bytes());

		if market_price > price { return (0, 0) }

		let mut shares_filled = 0;
		let mut spent = 0;
		let mut spendable = to_spend;
		
		while spendable > 100 && market_price <= price {
			let shares_to_fill_at_market_price = cmp::min(spendable / market_price, share_depth.expect("expected there to be share depth"));
			
			for orderbook_id in  0..self.outcomes {
				if orderbook_id == outcome {continue;}
				let mut orderbook = self.orderbooks.get(&orderbook_id).expect("orderbook doens't exist where it should");

				if orderbook.price_data.max().is_some() {
					orderbook.fill_best_orders(shares_to_fill_at_market_price);
					self.orderbooks.insert(&orderbook_id, &orderbook); 
				}
			}

			spendable -= shares_to_fill_at_market_price * market_price;
			shares_filled += shares_to_fill_at_market_price;
			spent += shares_to_fill_at_market_price * market_price;
			let (updated_market_price, updated_share_depth) = self.get_market_price_and_min_liquidty(outcome);
			market_price = updated_market_price;
			share_depth = updated_share_depth;
		}

		return (spent, shares_filled);
	}

	pub fn get_market_price(
		&self, 
		outcome: u64
	) -> u128 {
		let mut market_price = 100;

 		for (orderbook_id, orderbook) in self.orderbooks.iter() {
			if orderbook_id == outcome {continue};

			let best_price = orderbook.price_data.max().unwrap_or(0);
			if best_price == 0 {continue;}

			market_price -= best_price;
		}
		return market_price;
	}

	pub fn get_market_price_and_min_liquidty(
		&self, 
		outcome: u64
	) -> (u128, Option<u128>) {
		let mut market_price = 100;
		let mut min_liquidity = None;

 		for (orderbook_id, orderbook) in self.orderbooks.iter() {
			if orderbook_id == outcome {continue};

			let best_price = orderbook.price_data.max().unwrap_or(0);
			if best_price == 0 {continue;}
			let liq_at_price = orderbook.price_data
				.get(&best_price)
				.expect("there should be an entry at best price but there isn't")
				.share_liquidity;

			if min_liquidity.is_none() || min_liquidity.unwrap() > liq_at_price {
				min_liquidity = Some(liq_at_price);
			}

			market_price -= best_price;
		}
		return (market_price, min_liquidity);
	}



	// pub fn resolute(
	// 	&mut self,
	// 	sender: String,
	// 	winning_outcome: Option<u64>, 
	// 	stake: u128
	// ) -> u128 {
	// 	let outcome_id = self.to_numerical_outcome(winning_outcome);
	// 	let mut resolution_window = self.resolution_windows.get(self.resolution_windows.len() - 1).expect("Something went wrong during market creation");
		
	// 	let mut to_return = 0;
	// 	let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).unwrap_or(0);

	// 	if stake + staked_on_outcome >= self.resolute_bond {
	// 		to_return = stake + staked_on_outcome - self.resolute_bond;
	// 		self.winning_outcome = winning_outcome;
	// 		self.resoluted = true;
	// 	} 

	// 	let mut sender_stake_per_outcome = resolution_window.participants_to_outcome_to_stake
	// 	.get(&sender)
	// 	.unwrap_or(UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}:{}", self.id, resolution_window.round, sender).as_bytes().to_vec()));
	// 	let stake_in_outcome = sender_stake_per_outcome
	// 	.get(&outcome_id)
	// 	.unwrap_or(0);
	// 	let new_stake = stake_in_outcome + stake - to_return;
	// 	sender_stake_per_outcome.insert(&outcome_id, &new_stake);
	// 	resolution_window.participants_to_outcome_to_stake.insert(&sender, &sender_stake_per_outcome);

	// 	let staked_on_outcome = resolution_window.staked_per_outcome
	// 	.get(&outcome_id)
	// 	.unwrap_or(0);
	// 	let new_stake_on_outcome = staked_on_outcome + stake - to_return;
	// 	resolution_window.staked_per_outcome.insert(&outcome_id, &new_stake_on_outcome);


		
	// 	if self.resoluted {

	// 		resolution_window.outcome = winning_outcome;
	// 		let new_resolution_window = ResolutionWindow {
	// 			round: resolution_window.round + 1,
	// 			participants_to_outcome_to_stake: UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}", self.id, resolution_window.round + 1).as_bytes().to_vec()), // Staked per outcome
	// 			required_bond_size: resolution_window.required_bond_size * 2,
	// 			staked_per_outcome: UnorderedMap::new(format!("market:{}:staked_per_outcome:{}", self.id, resolution_window.round + 1).as_bytes().to_vec()), // Staked per outcome
	// 			end_time: env::block_timestamp() / 1000000 + 43200000, // dispute time is 12 hours for first release
	// 			outcome: None,
	// 		};


	// 		env::log(
	// 			json!({
	// 				"type": "market_resoluted".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"sender": sender,
	// 					"round": U64(resolution_window.round),
	// 					"staked": U128(stake - to_return),
	// 					"outcome": U64(outcome_id),
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);

	// 		env::log(
	// 			json!({
	// 				"type": "new_resolution_window".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"round": U64(new_resolution_window.round),
	// 					"required_bond_size": U128(new_resolution_window.required_bond_size),
	// 					"end_time": U64(new_resolution_window.end_time),	
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);
	// 		self.resolution_windows.push(&new_resolution_window);
	// 	}  else {
	// 		env::log(
	// 			json!({
	// 				"type": "staked_on_resolution".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"sender": sender,
	// 					"round": U64(resolution_window.round),
	// 					"staked": U128(stake - to_return),
	// 					"outcome": U64(outcome_id),
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);
	// 	}
		
	// 	self.resolution_windows.replace(resolution_window.round, &resolution_window);

	// 	return to_return;
	// }

	// pub fn dispute(
	// 	&mut self, 
	// 	sender: String,
	// 	winning_outcome: Option<u64>,
	// 	stake: u128
	// ) -> u128 {

	// 	let outcome_id = self.to_numerical_outcome(winning_outcome);
	// 	let mut resolution_window = self.resolution_windows.get(self.resolution_windows.len() - 1).expect("Invalid dispute window unwrap");
	// 	let full_bond_size = resolution_window.required_bond_size;
	// 	let mut bond_filled = false;
	// 	let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).unwrap_or(0);
	// 	let mut to_return = 0;

	// 	if staked_on_outcome + stake >= full_bond_size  {
	// 		bond_filled = true;
	// 		to_return = staked_on_outcome + stake - full_bond_size;
	// 		self.disputed = true; // Only as long as Judge exists
	// 		self.winning_outcome = winning_outcome;
	// 	}

	// 	let mut sender_stake_per_outcome = resolution_window.participants_to_outcome_to_stake
	// 	.get(&sender)
	// 	.unwrap_or(UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}:{}", self.id, resolution_window.round, sender).as_bytes().to_vec()));
	// 	let stake_in_outcome = sender_stake_per_outcome
	// 	.get(&outcome_id)
	// 	.unwrap_or(0);
	// 	let new_stake = stake_in_outcome + stake - to_return;
	// 	sender_stake_per_outcome.insert(&outcome_id, &new_stake);
	// 	resolution_window.participants_to_outcome_to_stake.insert(&sender, &sender_stake_per_outcome);

	// 	resolution_window.staked_per_outcome.insert(&outcome_id, &(staked_on_outcome + stake - to_return));

		
	// 	// Check if this order fills the bond
	// 	if bond_filled {
	// 		// Set last winning outcome
	// 		resolution_window.outcome = winning_outcome;

	// 		let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).expect("This can't be None");
	// 		assert_eq!(staked_on_outcome, full_bond_size, "the total staked on outcome needs to equal full bond size if we get here");

	// 		let next_resolution_window = ResolutionWindow{
	// 			round: resolution_window.round + 1,
	// 			participants_to_outcome_to_stake: UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}", self.id, resolution_window.round + 1).as_bytes().to_vec()), // Staked per outcome
	// 			required_bond_size: resolution_window.required_bond_size * 2,
	// 			staked_per_outcome: UnorderedMap::new(format!("market:{}:staked_per_outcome:{}", self.id, resolution_window.round + 1).as_bytes().to_vec()), // Staked per outcome
	// 			end_time: env::block_timestamp() / 1000000 + 43200000,
	// 			outcome: None,
	// 		};

	// 		env::log(
	// 			json!({
	// 				"type": "resolution_disputed".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"sender": sender,
	// 					"round": U64(resolution_window.round),
	// 					"staked": U128(stake - to_return),
	// 					"outcome": U64(outcome_id)
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);
	// 		env::log(
	// 			json!({
	// 				"type": "new_resolution_window".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"round": U64(next_resolution_window.round),
	// 					"required_bond_size": U128(next_resolution_window.required_bond_size),
	// 					"end_time": U64(next_resolution_window.end_time),	
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);

	// 		self.resolution_windows.push(&next_resolution_window);
	// 	} else {
	// 		env::log(
	// 			json!({
	// 				"type": "staked_on_dispute".to_string(),
	// 				"params": {
	// 					"market_id": U64(self.id),
	// 					"sender": sender,
	// 					"round": U64(resolution_window.round),
	// 					"staked": U128(stake - to_return),
	// 					"outcome": U64(outcome_id)
	// 				}
	// 			})
	// 			.to_string()
	// 			.as_bytes()
	// 		);
	// 	}

	// 	self.resolution_windows.replace(resolution_window.round, &resolution_window);

	// 	return to_return;
	// }

	// pub fn finalize(
	// 	&mut self, 
	// 	winning_outcome: Option<u64>
	// ) {
	// 	assert_eq!(self.resoluted, true, "market isn't resoluted yet");
	// 	assert!(winning_outcome == None || winning_outcome.unwrap() < self.outcomes, "invalid outcome");
	
	//     if self.disputed {
    //         self.winning_outcome = winning_outcome;
	// 	}

	// 	env::log(
	// 		json!({
	// 			"type": "market_finalized".to_string(),
	// 			"params": {
	// 				"market_id": U64(self.id),
	// 				"winning_outcome": U64(self.to_numerical_outcome(self.winning_outcome))
	// 			}
	// 		})
	// 		.to_string()
	// 		.as_bytes()
	// 	);
		
	//     self.finalized = true;
	// }

	// pub fn get_claimable_for(
	// 	&self, 
	// 	account_id: String
	// ) -> (u128, u128, u128, HashMap<String, u128>) {
	// 	let invalid = self.winning_outcome.is_none();
	// 	let mut winnings = 0;
	// 	let mut in_open_orders = 0;
	// 	let mut affiliates: HashMap<String, u128> = HashMap::new();
	// 	// Claiming payouts
	// 	if invalid {
	// 		for (_, orderbook) in self.orderbooks.iter() {
	// 			in_open_orders += orderbook.get_open_order_value_for(account_id.to_string());
	// 			let spent = orderbook.get_spend_by(account_id.to_string());
	// 			winnings += spent; // market creator forfits his fee when market resolutes to invalid
	// 		}
	// 		winnings -= in_open_orders;
	// 	} else {
	// 		for (_, orderbook) in self.orderbooks.iter() {
	// 			in_open_orders += orderbook.get_open_order_value_for(account_id.to_string());
	// 		}

	// 		let winning_orderbook = self.orderbooks.get(&self.to_numerical_outcome(self.winning_outcome)).unwrap();
	// 		let (winning_value, affiliate_map) = winning_orderbook.calc_claimable_amt(account_id.to_string());
	// 		affiliates = affiliate_map;
	// 		let claimable_if_valid = self.claimable_if_valid.get(&account_id.to_string()).unwrap_or(0);
	// 		winnings += winning_value + claimable_if_valid;
	// 	}

	// 	// Claiming Dispute Earnings
	
    //     let governance_earnings = self.get_dispute_earnings(account_id.to_string());
	// 	return (winnings, in_open_orders, governance_earnings, affiliates);
	// }

	// pub fn cancel_dispute_participation(
	// 	&mut self,
	// 	round: u64,
		
	// 	outcome: Option<u64>
	// ) -> u128{
	// 	let outcome_id = self.to_numerical_outcome(outcome);
	// 	let mut resolution_window = self.resolution_windows.get(round).expect("dispute round doesn't exist");
	// 	assert_ne!(outcome, resolution_window.outcome, "you cant cancel dispute stake for bonded outcome");
	// 	let mut sender_particiaption = resolution_window.participants_to_outcome_to_stake.get(&env::predecessor_account_id()).expect("user didn't paritcipate in this dispute round");
	// 	let to_return = sender_particiaption.get(&outcome_id).expect("sender didn't pariticipate in this outcome resolution");
	// 	assert!(to_return > 0, "sender canceled their dispute participation");

	// 	sender_particiaption.insert(&outcome_id, &0);
	// 	resolution_window.participants_to_outcome_to_stake.insert(&env::predecessor_account_id(), &sender_particiaption);

	// 	self.resolution_windows.replace(resolution_window.round, &resolution_window);
	// 	return to_return;
	// }

	// fn get_dispute_earnings(
	// 	&self, 
	// 	account_id: String
	// ) -> u128 {
	// 	let mut user_correctly_staked = 0;
	// 	let mut resolution_reward = 0;
	// 	let mut total_correctly_staked = 0;
	// 	let mut total_incorrectly_staked = 0;

	// 	let winning_outcome_id = self.to_numerical_outcome(self.winning_outcome);
			
	// 	for window in self.resolution_windows.iter() {
	// 		// check if round - round 0 - which is the resolution round
	// 		if window.round == 0 {

	// 			// Calculate how much the total fee payout will be 
	// 			let total_resolution_fee = self.resolution_fee_percentage * self.filled_volume / 10000;
	// 			// Check if the outcome that a resolution bond was staked on coresponds with the finalized outcome
	// 			if self.winning_outcome == window.outcome {
	// 				// check if the user participated in this outcome
	// 				let resolution_participation = !window.participants_to_outcome_to_stake.get(&account_id).is_none();
					
	// 				if resolution_participation {
	// 					// Check how much of the bond the user participated
	// 					let correct_outcome_participation = window.participants_to_outcome_to_stake
	// 					.get(&account_id)
	// 					.unwrap()
	// 					.get(&self.to_numerical_outcome(self.winning_outcome))
	// 					.unwrap_or(0);

	// 					if correct_outcome_participation > 0 {
	// 						// calculate his relative share of the total_resolution_fee relative to his participation
	// 						resolution_reward += total_resolution_fee * correct_outcome_participation * 100 / window.required_bond_size / 100 + correct_outcome_participation;
	// 					}
						
	// 				} 
	// 			} else {
	// 				// If the initial resolution bond wasn't staked on the correct outcome, devide the resolution fee amongst disputors
	// 				total_incorrectly_staked += total_resolution_fee + window.required_bond_size;
	// 			}
	// 		} else {
	// 			// If it isn't the first round calculate according to escalation game
	// 			let window_outcome_id = self.to_numerical_outcome(window.outcome);

	// 			if window_outcome_id == winning_outcome_id {
	// 				let round_participation = window.participants_to_outcome_to_stake
	// 				.get(&account_id)
	// 				.unwrap_or(UnorderedMap::new(format!("market:{}:staked_per_outcome:{}:{}", self.id, window.round + 1, account_id).as_bytes().to_vec()))
	// 				.get(&winning_outcome_id)
	// 				.unwrap_or(0);

	// 				user_correctly_staked += round_participation;
	// 				total_correctly_staked += window.required_bond_size;
	// 			} else if window.outcome.is_some() {
	// 				total_incorrectly_staked += window.required_bond_size;
				 
	// 			}
	// 		}
	// 	}

	// 	if total_correctly_staked == 0 {return resolution_reward}
	
	// 	let percentage_earnigns = user_correctly_staked * 100 / total_correctly_staked;
	// 	let profit = percentage_earnigns * total_incorrectly_staked / 100;
	// 	return profit + user_correctly_staked + resolution_reward;
	// }

    // // Updates the best price for an order once initial best price is filled
	// fn update_next_best_price(
	// 	&self, 
	// 	inverse_orderbook_ids: &Vec<u64>, 
	// 	first_iteration: &bool, 
	// 	outcome_to_price_share_pointer: &mut HashMap<u64, (u128, u128)>, 
	// 	best_order_exists: &mut bool, 
	// 	market_price: &mut u128, 
	// 	lowest_liquidity: &u128
	// ) {
	//     for orderbook_id in inverse_orderbook_ids {
    //         let orderbook = self.orderbooks.get(&orderbook_id).unwrap();
    //         if !first_iteration {
    //             if outcome_to_price_share_pointer.get_mut(orderbook_id).is_none() {continue}
    //             outcome_to_price_share_pointer.get_mut(orderbook_id).unwrap().1 -= lowest_liquidity;
    //             let price_liquidity = outcome_to_price_share_pointer.get(orderbook_id).unwrap();
    //             let liquidity = price_liquidity.1;

    //             if liquidity == 0 {
    //                 // get next best price
    //                 let next_best_price_prom = orderbook.orders_by_price.lower(&price_liquidity.0);

    //                 if next_best_price_prom.is_none() {
    //                     outcome_to_price_share_pointer.remove(orderbook_id);
    //                     continue;
    //                 }
    //                 *best_order_exists = true;
    //                 let next_best_price = next_best_price_prom.unwrap();
    //                 let add_to_market_price =  price_liquidity.0 - next_best_price;
    //                 *market_price += add_to_market_price;
    //                 outcome_to_price_share_pointer.insert(*orderbook_id, (next_best_price, orderbook.get_liquidity_at_price(next_best_price)));
    //             }
    //         }
    //     }
	// }

    // // Updates the lowest liquidity available amongst best prices
	// fn update_lowest_liquidity(
	// 	&self, 
	// 	inverse_orderbook_ids: &Vec<u64>, 
	// 	first_iteration: &bool, 
	// 	lowest_liquidity: &mut u128, 
	// 	outcome_to_price_share_pointer: &mut HashMap<u64, (u128, u128)>, 
	// 	best_order_exists: &mut bool
	// ) {
	//     *best_order_exists = false;
	//     for orderbook_id in inverse_orderbook_ids {
    //         // Get lowest liquidity at new price
    //         let orderbook = self.orderbooks.get(&orderbook_id).unwrap();
    //         if *first_iteration {
    //             let price = orderbook.best_price;
    //             if price.is_none() {continue}
    //             *best_order_exists = true;
    //             let liquidity = orderbook.get_liquidity_at_price(price.unwrap());
    //             outcome_to_price_share_pointer.insert(*orderbook_id, (price.unwrap(), liquidity));
    //         }
    //         if outcome_to_price_share_pointer.get(orderbook_id).is_none() {continue}
    //         let liquidity = outcome_to_price_share_pointer.get(orderbook_id).unwrap().1;
    //         if *lowest_liquidity == 0 {*lowest_liquidity = liquidity}
    //         else if *lowest_liquidity > liquidity { *lowest_liquidity = liquidity}

    //     }
	// }

	fn to_loggable_winning_outcome(
		&self, 
		winning_outcome: Option<u64>
	) -> Option<U64> {
		return match winning_outcome {
			Some(outcome) => Some(U64(outcome)),
			None => None
		};
	}

	pub fn to_numerical_outcome(
		&self, 
		outcome: Option<u64>, 
	) -> u64 {
		return outcome.unwrap_or(self.outcomes);
	}
}

impl Default for Market {
	fn default() -> Self {
		panic!("No default state available init with ::new");
	}
}
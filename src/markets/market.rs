use near_sdk::{near_bindgen, env};
use near_sdk::collections::{UnorderedMap, TreeMap};
use near_sdk::json_types::{U128, U64};
use borsh::{BorshDeserialize, BorshSerialize};
use std::string::String;
use std::collections::{BTreeMap, HashMap};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ResolutionWindow {
	pub round: U64,
	pub participants_to_outcome_to_stake: UnorderedMap<String, UnorderedMap<U64, U128>>, // Account to outcome to stake
	pub required_bond_size: U128,
	pub staked_per_outcome: UnorderedMap<U64, U128>, // Staked per outcome
	pub end_time: U64,
	pub outcome: Option<U64>,
}

pub mod orderbook;
type Orderbook = orderbook::Orderbook;
type Order = orderbook::Order;

// TODO: possible to change to orderbooks to treemap since there is some sort of ordering
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
	pub id: U64,
	pub description: String,
	pub extra_info: String,
	pub creator: String,
	pub outcomes: U64,
	pub outcome_tags: Vec<String>,
	pub categories: Vec<String>,
	pub last_price_for_outcomes: UnorderedMap<U64, U128>,
	pub creation_time: U64,
	pub end_time: U64,
	pub orderbooks: UnorderedMap<U64, orderbook::Orderbook>,
	pub winning_outcome: Option<U64>, // invalid has outcome id: self.outcomes
	pub resoluted: bool,
	pub resolute_bond: U128,
	pub filled_volume: U128,
	pub disputed: bool,
	pub finalized: bool,
	pub creator_fee_percentage: U128,
	pub resolution_fee_percentage: U128,
	pub affiliate_fee_percentage: U128,
	pub claimable_if_valid: HashMap<String, U128>,
	pub api_source: String,
	pub resolution_windows: Vec<ResolutionWindow>
}

#[near_bindgen]
impl Market {
	pub fn new(
		id: U64, 
		account_id: String, 
		description: String, 
		extra_info: String, 
		outcomes: U64, 
		outcome_tags: Vec<String>, 
		categories: Vec<String>, 
		end_time: U64, 
		creator_fee_percentage: U128, 
		resolution_fee_percentage: U128, 
		affiliate_fee_percentage: U128,
		api_source: String
	) -> Self {
		let mut empty_orderbooks = UnorderedMap::new(vec![5]);
		let outcomes_native = outcomes.into();
		for i in 0..outcomes_native {
			empty_orderbooks.insert(&i.into(), &Orderbook::new(i));
		}

		let base: u128 = 10;
		let base_resolution_window = ResolutionWindow {
			round: 0.into(),
			participants_to_outcome_to_stake: UnorderedMap::new(vec![2]),
			required_bond_size: (5 * base.pow(17)).into(),
			staked_per_outcome: UnorderedMap::new(vec![3]), // Staked per outcome
			end_time: end_time.into(),
			outcome: None,
		};

		Self {
			id: id.into(),
			description,
			extra_info,
			creator: account_id,
			outcomes,
			outcome_tags,
			categories,
			last_price_for_outcomes: UnorderedMap::new(vec![4]),
			creation_time: (env::block_timestamp() / 1000000).into(),
			end_time: end_time.into(),
			orderbooks: empty_orderbooks,
			winning_outcome: None,
			resoluted: false,
			resolute_bond: (5 * base.pow(17)).into(),
			filled_volume: 0.into(),
			disputed: false,
			finalized: false,
			creator_fee_percentage: creator_fee_percentage.into(),
			resolution_fee_percentage: resolution_fee_percentage.into(),
			affiliate_fee_percentage: affiliate_fee_percentage.into(),
			claimable_if_valid: HashMap::new(),
			api_source,
			resolution_windows: vec![base_resolution_window]
		}
	}

	// pub fn dynamic_market_sell(
	// 	&mut self,
	// 	outcome: u64,
	// 	shares_to_sell: u128
	// ) -> u128 {
	// 	let orderbook = self.orderbooks.get_mut(&outcome).unwrap();
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
	// 			orderbook.fill_best_orders(shares_to_sell);
	// 		} else {
	// 			shares_fillable -= shares_sought;
	// 			spendable += liq_at_price;
	// 			claimable_if_valid += orderbook.subtract_shares(shares_sought, best_price);
	// 		}
			
	// 		let (next_price, liq_at_next_price) = orderbook.liquidity_by_price.range(0..best_price).next_back().unwrap_or((&0, &0));
	// 		best_price = *next_price;
	// 		liq_at_price = liq_at_next_price;
	// 	}

	// 	self.claimable_if_valid
	// 	.entry(env::predecessor_account_id())
	// 	.and_modify(|claimable| {
	// 		*claimable += claimable_if_valid;
	// 	})
	// 	.or_insert(claimable_if_valid);

	// 	orderbook.fill_best_orders(shares_to_sell - shares_fillable);
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
					
	// 		let (next_price, liq_at_next_price) = orderbook.liquidity_by_price.range(0..best_price).next_back().unwrap_or((&0, &0));
	// 		best_price = *next_price;
	// 		liq_at_price = liq_at_next_price;
	// 	}

	// 	return (spendable, shares_to_sell - shares_fillable);
	// }

	pub fn create_order(
		&mut self, 
		account_id: String, 
		outcome: U64, 
		amt_of_shares: u128, 
		spend: u128, 
		price: u128,
		affiliate_account_id: Option<String>
	) {
		let spend_u128: u128 = spend.into();
		let price_u128: u128 = price.into();
		let filled_volume_u128: u128 = self.filled_volume.into();
		assert!(spend_u128 > 0);
		assert!(price_u128 > 0 && price_u128 < 100);
		assert_eq!(self.resoluted, false);
		assert!(env::block_timestamp() / 1000000 < self.end_time.into());
		let (spend_left, shares_filled) = self.fill_matches(outcome.into(), spend_u128, price_u128);
		let total_spend = spend_u128 - spend_left;
		self.filled_volume = (filled_volume_u128 + shares_filled * 100).into();
		let orderbook = self.orderbooks.get(&outcome).expect("this outcome does not exist");
		// orderbook.place_order(account_id, outcome.into(), spend, amt_of_shares, price, total_spend, shares_filled, affiliate_account_id);
	}

	fn fill_matches(
		&mut self, 
		outcome: u64, 
		spend: u128, 
		price: u128
	) -> (u128, u128) {
		let mut market_price: u128 = self.get_market_price_for(outcome.into()).into();
		if market_price > price { return (spend,0) }
		let orderbook_ids = self.get_matching_orderbook_ids(outcome.into());

		let mut shares_filled = 0;
		let mut spendable = spend;

		while spendable > 100 && market_price <= price {
			let mut shares_to_fill = spendable / market_price;
			let shares_fillable = self.get_min_shares_fillable(outcome);
			self.last_price_for_outcomes.insert(&outcome.into(), &market_price.into());

			if shares_fillable < shares_to_fill {
				shares_to_fill = shares_fillable;
            }
			for orderbook_id in &orderbook_ids {
				let mut orderbook = self.orderbooks.get(&orderbook_id).unwrap();
				if !orderbook.best_price.is_none() {
					let best_price = orderbook.get_best_price();
					self.last_price_for_outcomes.insert(orderbook_id, &best_price.into());
					orderbook.fill_best_orders(shares_to_fill);
				}
			}

			spendable -= shares_to_fill * market_price;
			shares_filled += shares_to_fill;
			market_price = self.get_market_price_for(outcome.into()).into();
		}

		return (spendable, shares_filled);
	}

	fn get_min_shares_fillable(
		&self, 
		outcome: u64
	) -> u128 {
		let mut shares = None;
		let orderbook_ids = self.get_matching_orderbook_ids(outcome.into());
		for orderbook_id in orderbook_ids {
			let orderbook = self.orderbooks.get(&orderbook_id).unwrap();
			if !orderbook.best_price.is_none() {
				let best_price_liquidity = orderbook.get_liquidity_at_price(orderbook.best_price.unwrap());
				if shares.is_none() || shares.unwrap() > best_price_liquidity {shares = Some(best_price_liquidity)}
			}
		}
		return shares.unwrap();
	}

	// pub fn get_market_prices_for(
	// 	&self
	// ) -> BTreeMap<u64, u128> {
	// 	let mut market_prices: BTreeMap<u64, u128> = BTreeMap::new();
	// 	for outcome in 0..self.outcomes {
	// 		let market_price = self.get_market_price_for(outcome);
	// 		market_prices.insert(outcome, market_price);
	// 	}
	// 	return market_prices;
	// }

	pub fn get_market_price_for(
		&self, 
		outcome: U64
	) -> U128 {
		let orderbook_ids = self.get_matching_orderbook_ids(outcome.into());
		let mut market_price = 100;

 		for orderbook_id in orderbook_ids {
			let orderbook = self.orderbooks.get(&orderbook_id).unwrap();
			let best_price = orderbook.best_price;

			if !best_price.is_none() {
				market_price -= best_price.unwrap();
			}
		}
		return market_price.into();
	}

	fn get_matching_orderbook_ids(
		&self, 
		principle_outcome: U64
	) -> Vec<U64> {
		let mut orderbooks = vec![];

		for (outcome, _) in self.orderbooks.iter() {
			if outcome != principle_outcome {
				orderbooks.push(outcome);
			}
		}

		return orderbooks;
	}

	fn to_numerical_outcome(
		&self, 
		outcome: Option<U64>, 
	) -> U64 {
		return outcome.unwrap_or(self.outcomes);
	}

	pub fn resolute(
		&mut self, 
		winning_outcome: Option<U64>, 
		stake: U128
	) -> u128 {
		let end_time: u64 = self.end_time.into();
		let outcomes: u64 = self.outcomes.into();
		let stake: u128 = stake.into();
		assert!(env::block_timestamp() / 1000000 >= end_time, "market hasn't ended yet");
		assert_eq!(self.resoluted, false, "market is already resoluted");
		assert_eq!(self.finalized, false, "market is already finalized");
		let winning_outcome_numerical: u64 = self.to_numerical_outcome(winning_outcome).into();
		assert!(winning_outcome == None || winning_outcome_numerical < outcomes, "invalid winning outcome");
		let outcome_id = self.to_numerical_outcome(winning_outcome);
		let resolution_window = self.resolution_windows.last_mut().expect("no resolute window exists, something went wrong at creation");
		let round: u64 = resolution_window.round.into();
		assert_eq!(round, 0, "can only resolute once");
		
		let mut to_return = 0;
		let staked_on_outcome: u128 = resolution_window.staked_per_outcome.get(&outcome_id.into()).unwrap_or(0.into()).into();
		let resolution_bond: u128 = self.resolute_bond.into();
		let total_stake_incl: u128 = stake + staked_on_outcome;

		if total_stake_incl >= self.resolute_bond.into() {
			to_return = total_stake_incl - resolution_bond;
			self.winning_outcome = winning_outcome;
			self.resoluted = true;
		} 

		// Add to users resolution participation
		let user_participation = resolution_window.participants_to_outcome_to_stake.get(&env::predecessor_account_id());

		let mut participation_in_outcomes = match user_participation {
			Some(outcomes_to_stake) => outcomes_to_stake,
			None => UnorderedMap::new(vec![7]),
		};


		match participation_in_outcomes.get(&winning_outcome_numerical.into()) {
			Some(participation) => {
				let participation: u128 = participation.into();
				participation_in_outcomes.insert(&winning_outcome_numerical.into(), &(participation + stake - to_return).into())
			},
			None => participation_in_outcomes.insert(&winning_outcome_numerical.into(), &(stake - to_return).into()),
		};

		resolution_window.participants_to_outcome_to_stake.insert(&env::predecessor_account_id(), &participation_in_outcomes);

		// Add to total staked in round
		let staked = match resolution_window.staked_per_outcome.get(&winning_outcome_numerical.into()) {
			Some(staked) => {
				let staked: u128 = staked.into();
				return staked + stake - to_return;
			},
			None => stake - to_return,
		}; 

		resolution_window.staked_per_outcome.insert(&winning_outcome_numerical.into(), &staked.into());
		
		if self.resoluted {
			resolution_window.outcome = winning_outcome;
			let current_round: u64 = resolution_window.round.into();
			let next_round: u64 = current_round + 1;

			let current_bond_size: u128 = resolution_window.required_bond_size.into();
			let next_bond_size: u128 = current_bond_size * 2;

			let next_round_end_time: u64 = env::block_timestamp() / 1000000 + 1800000;

			let new_resolution_window = ResolutionWindow {
				round: next_round.into(),
				participants_to_outcome_to_stake: UnorderedMap::new(vec![8]),
				required_bond_size: next_bond_size.into(),
				staked_per_outcome: UnorderedMap::new(vec![9]), // Staked per outcome
				end_time: next_round_end_time.into(),
				outcome: None,
			};
			self.resolution_windows.push(new_resolution_window);
		} 

		return to_return;
	}

	// pub fn dispute(
	// 	&mut self, 
	// 	winning_outcome: Option<u64>,
	// 	stake: u128
	// ) -> u128 {
	// 	assert_eq!(self.resoluted, true, "market isn't resoluted yet");
	// 	assert_eq!(self.finalized, false, "market is already finalized");
    //     assert!(winning_outcome == None || winning_outcome.unwrap() < self.outcomes, "invalid winning outcome");
    //     assert!(winning_outcome != self.winning_outcome, "same oucome as last resolution");
	
	// 	let outcome_id = self.to_numerical_outcome(winning_outcome);
	// 	let resolution_window = self.resolution_windows.last_mut().expect("Invalid dispute window unwrap");
	// 	assert_eq!(resolution_window.round, 1, "for this version, there's only 1 round of dispute");
	// 	assert!(env::block_timestamp() / 1000000 <= resolution_window.end_time, "dispute window is closed, market can be finalized");

	// 	let full_bond_size = resolution_window.required_bond_size;
	// 	let mut bond_filled = false;
	// 	let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).unwrap_or(&0);
	// 	let mut to_return = 0;

	// 	if staked_on_outcome + stake >= full_bond_size  {
	// 		bond_filled = true;
	// 		to_return = staked_on_outcome + stake - full_bond_size;
	// 		self.disputed = true; // Only as long as Judge exists
	// 		self.winning_outcome = winning_outcome;
	// 	}

	// 	// Add to disputors stake
	// 	resolution_window.participants_to_outcome_to_stake
	// 	.entry(env::predecessor_account_id())
	// 	.or_insert(HashMap::new())
	// 	.entry(outcome_id)
	// 	.and_modify(|staked| { *staked += stake - to_return })
	// 	.or_insert(stake);

	// 	// Add to total staked on outcome
	// 	resolution_window.staked_per_outcome
	// 	.entry(outcome_id)
	// 	.and_modify(|total_staked| {*total_staked += stake - to_return})
	// 	.or_insert(stake);
		
	// 	// Check if this order fills the bond
	// 	if bond_filled {
	// 		// Set last winning outcome
	// 		resolution_window.outcome = winning_outcome;

	// 		//
	// 		resolution_window.staked_per_outcome
	// 		.entry(outcome_id)
	// 		.and_modify(|total_staked| {*total_staked = full_bond_size})
	// 		.or_insert(stake);

	// 		let next_resolution_window = ResolutionWindow{
	// 			round: resolution_window.round + 1,
	// 			participants_to_outcome_to_stake: HashMap::new(),
	// 			required_bond_size: resolution_window.required_bond_size * 2,
	// 			staked_per_outcome: HashMap::new(), // Staked per outcome
	// 			end_time: env::block_timestamp() / 1000000 + 1800000,
	// 			outcome: None,
	// 			// invalid: false
	// 		};

	// 		self.resolution_windows.push(next_resolution_window);
	// 	}

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
		
	//     self.finalized = true;
	// }

	// // TODO: claimable should probably be renamed to something like: dispute earnings
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
	// 		    let spent = orderbook.get_spend_by(account_id.to_string());
	// 			winnings += spent; // market creator forfits his fee when market resolutes to invalid
	// 		}
	// 	} else {
	// 		for (_, orderbook) in self.orderbooks.iter() {
	// 			in_open_orders += orderbook.get_open_order_value_for(account_id.to_string());
	// 		}

	// 		let winning_orderbook = self.orderbooks.get(&self.to_numerical_outcome(self.winning_outcome)).unwrap();
	// 		let (winning_value, affiliate_map) = winning_orderbook.calc_claimable_amt(account_id.to_string());
	// 		affiliates = affiliate_map;
	// 		winnings += winning_value;
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
	// 	let resolution_window = self.resolution_windows.get_mut(round as usize).expect("dispute round doesn't exist");
	// 	assert_ne!(outcome, resolution_window.outcome, "you cant cancel dispute stake for bonded outcome");
	// 	assert_ne!(outcome, self.winning_outcome, "you cant cancel dispute stake for winning outcome");
	// 	let mut to_return = 0;
	// 	resolution_window.participants_to_outcome_to_stake
	// 	.entry(env::predecessor_account_id())
	// 	.or_insert(HashMap::new())
	// 	.entry(outcome_id)
	// 	.and_modify(|staked| { 
	// 		to_return = *staked;
	// 		*staked = 0 ;
	// 	})
	// 	.or_insert(0);

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
			
	// 	for window in &self.resolution_windows {
	// 		// check if round - round 0 - which is the resolution round
	// 		if window.round == 0 {

	// 			// Calculate how much the total fee payout will be 
	// 			let total_resolution_fee = self.resolution_fee_percentage * self.filled_volume / 100;
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
	// 					.unwrap_or(&0);

	// 					if correct_outcome_participation > &0 {
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
	// 			let empty_map = HashMap::new();
	// 			let window_outcome_id = self.to_numerical_outcome(window.outcome);
	// 			let round_participation = window.participants_to_outcome_to_stake
	// 			.get(&account_id)
	// 			.unwrap_or(&empty_map)
	// 			.get(&winning_outcome_id)
	// 			.unwrap_or(&0);
				
	// 			let correct_stake = window.staked_per_outcome
	// 			.get(&winning_outcome_id)
	// 			.unwrap_or(&0);


	// 			let incorrect_stake = window.staked_per_outcome
	// 			.get(&window_outcome_id)
	// 			.unwrap_or(&0);

	// 			user_correctly_staked += round_participation;
	// 			total_correctly_staked += correct_stake;
	// 			total_incorrectly_staked += incorrect_stake;

	// 		}
	// 	}

	// 	if total_correctly_staked == 0 {return resolution_reward}
		
    //     return user_correctly_staked * 100 / total_correctly_staked * total_incorrectly_staked / 100 + resolution_reward;
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
    //                 let next_best_price_prom = orderbook.orders_by_price.range(0..price_liquidity.0 - 1).next();

    //                 if next_best_price_prom.is_none() {
    //                     outcome_to_price_share_pointer.remove(orderbook_id);
    //                     continue;
    //                 }
    //                 *best_order_exists = true;
    //                 let next_best_price = *next_best_price_prom.unwrap().0;
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

	// // TODO: Add get_liquidity function that doesn't need the spend argument
	// pub fn get_liquidity_available(
	// 	&self, 
	// 	outcome: u64, 
	// 	spend: u128, 
	// 	price: u128
	// ) -> u128 {
	// 	let inverse_orderbook_ids = self.get_inverse_orderbook_ids(outcome);
	// 	// Mapped outcome to price and liquidity left
	// 	let mut outcome_to_price_share_pointer: HashMap<u64,  (u128, u128)> = HashMap::new();
	// 	let mut max_spend = 0;
	// 	let mut max_shares = 0;
	// 	let mut market_price = self.get_market_price_for(outcome);
	// 	let mut best_order_exists = true;
	// 	let mut lowest_liquidity = 0;
	// 	let mut first_iteration = true;

	// 	while max_spend < spend && market_price <= price && best_order_exists {
	// 		self.update_next_best_price(&inverse_orderbook_ids,
	// 		&first_iteration,
	// 		&mut outcome_to_price_share_pointer,
	// 		&mut best_order_exists,
	// 		&mut market_price,
	// 		&lowest_liquidity);

	// 		lowest_liquidity = 0;
	// 		if market_price <= price {
	// 			self.update_lowest_liquidity(
	// 				&inverse_orderbook_ids,
	// 				&first_iteration,
	// 				&mut lowest_liquidity,
    //             	&mut outcome_to_price_share_pointer,
	// 				&mut best_order_exists
	// 			);
	// 			max_spend += lowest_liquidity * market_price;
	// 			max_shares += lowest_liquidity;
	// 		}
	// 		first_iteration = false;
	// 	}

	// 	return max_spend;
	// }


	// pub fn reset_balances_for(
	// 	&mut self, 
	// 	account_id: String
	// ) {
	// 	for orderbook_id in 0..self.outcomes {
	// 		let orderbook = self.orderbooks.get_mut(&orderbook_id).unwrap();
	// 		orderbook.delete_orders_for(account_id.to_string());
	// 	}
	// }

	// pub fn delete_resolution_for(
	// 	&mut self,
	// 	account_id: String,
	// ) {
	// 	let outcome_id = self.to_numerical_outcome(self.winning_outcome);
	// 	for window in &mut self.resolution_windows {
	// 		window.participants_to_outcome_to_stake
	// 		.entry(account_id.to_string())
	// 		.or_insert(HashMap::new())
	// 		.entry(outcome_id)
	// 		.and_modify(|staked| {
	// 			*staked = 0
	// 		})
	// 		.or_insert(0);
	// 	}
	// }
}
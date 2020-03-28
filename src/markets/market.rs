use std::string::String;
use std::collections::BTreeMap;
use std::collections::HashMap;
use near_bindgen::{near_bindgen};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub mod orderbook;
type Orderbook = orderbook::Orderbook;
type Order = orderbook::Order;

// TODO: Endtime has to be in blocks instead of ms / s
// TODO: Add resolution_url
// TODO: When filling an order for a price > market price the tx fails
// TODO: Share denomination seems off by 1 decimal - don't know if frontend or backend fix
#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
pub struct Market {
	pub id: u64,
	pub description: String,
	pub extra_info: String,
	pub creator: String,
	pub outcomes: u64,
	pub outcome_tags: Vec<String>,
	pub categories: Vec<String>,
	pub last_price_for_outcomes: HashMap<u64, u128>,
	// creation_time: u128
	pub end_time: u64,
	pub orderbooks: BTreeMap<u64, orderbook::Orderbook>,
	pub winning_outcome: Option<u64>,
	pub resoluted: bool,
	pub liquidity: u128
}

impl Market {
	pub fn new(id: u64, from: String, description: String, extra_info: String, outcomes: u64, outcome_tags: Vec<String>, categories: Vec<String>, end_time: u64) -> Self {
		let mut empty_orderbooks = BTreeMap::new();
		// TODO get blocktime at creation

		for i in 0..outcomes {
			empty_orderbooks.insert(i, Orderbook::new(i));
		}

		Self {
			id,
			description,
			extra_info,
			creator: from,
			outcomes,
			outcome_tags,
			categories,
			last_price_for_outcomes: HashMap::new(),
			end_time,
			orderbooks: empty_orderbooks,
			winning_outcome: None,
			resoluted: false,
			liquidity: 0
		}
	}

	pub fn place_order(&mut self, from: String, outcome: u64, amt_of_shares: u128, spend: u128, price_per_share: u128) {
		assert!(spend > 0);
		assert!(price_per_share > 0 && price_per_share < 100);
		assert_eq!(self.resoluted, false);
		let (spend_filled, shares_filled) = self.fill_matches(outcome, spend, price_per_share, 0);
		let total_spend = spend - spend_filled;
		self.liquidity += spend;
		let shares_filled = shares_filled;
		let orderbook = self.orderbooks.get_mut(&outcome).unwrap();
		orderbook.place_order(from, outcome, spend, amt_of_shares, price_per_share, total_spend, shares_filled);
	}

	fn fill_matches(&mut self, outcome: u64, mut spend: u128, price_per_share: u128, mut shares_filled: u128) -> (u128, u128) {
		let market_price = self.get_market_price(outcome);
		let mut shares_to_fill = spend / market_price;
		if price_per_share < market_price || spend < 100 { return (spend, shares_filled); }
		let orderbook_ids = self.get_inverse_orderbook_ids(outcome);
		let shares_fillable = self.get_min_shares_fillable(outcome);
		self.last_price_for_outcomes.insert(outcome, market_price);

		if shares_fillable < shares_to_fill {
			shares_to_fill = shares_fillable;
		}
		
		for orderbook_id in orderbook_ids {
			let orderbook = self.orderbooks.get_mut(&orderbook_id).unwrap();
			if !orderbook.market_order.is_none() {
				let market_order_price = orderbook.get_market_order_price();
				self.last_price_for_outcomes.insert(orderbook_id, market_order_price);
				orderbook.fill_market_order(shares_to_fill);
			}
		}
		spend -= shares_to_fill * market_price;
		shares_filled += shares_to_fill;

		return (spend, shares_filled);
	}

	pub fn get_min_shares_fillable(&self, outcome: u64) -> u128 {
		let mut shares = None;
		let orderbook_ids = self.get_inverse_orderbook_ids(outcome);
		for orderbook_id in orderbook_ids {
			let orderbook = self.orderbooks.get(&orderbook_id).unwrap();

            if let Some((market_order_price_per_share, market_order_map)) = orderbook.open_orders.iter().next() {
                let mut left_to_fill = 0;
                let mut shares_to_fill = 0;
                for (_, order) in market_order_map.iter() {
                    left_to_fill += order.spend - order.filled;
                    shares_to_fill += left_to_fill / market_order_price_per_share;
                }
                if shares.is_none() || shares_to_fill < shares.unwrap() {
                    shares = Some(shares_to_fill);
                }
            }
		}
		return shares.unwrap();
	}

	pub fn get_market_prices(&self) -> BTreeMap<u64, u128> {
		let mut market_prices: BTreeMap<u64, u128> = BTreeMap::new();
		for outcome in 0..self.outcomes {
			let market_price = self.get_market_price(outcome);
			market_prices.insert(outcome, market_price);
		}
		return market_prices;
	}

	pub fn get_market_price(&self, outcome: u64) -> u128 {
		let orderbook_ids = self.get_inverse_orderbook_ids(outcome);
		let mut market_price = 100;

 		for orderbook_id in orderbook_ids {
			let orderbook = self.orderbooks.get(&orderbook_id).unwrap();
			let market_order_price_per_share = orderbook.market_order;

			if !market_order_price_per_share.is_none() {
				market_price -= market_order_price_per_share.unwrap();
			}
		}
		return market_price;
	}

	fn get_inverse_orderbook_ids(&self, principle_outcome: u64) -> Vec<u64> {
		let mut orderbooks = vec![];

		for i in 0..self.outcomes {
			if i != principle_outcome {
				orderbooks.push(i);
			}
		}

		return orderbooks;
	}

	pub fn resolute(&mut self,from: String, winning_outcome: Option<u64>) {
		// TODO: Make sure market can only be resoluted after end time
		assert_eq!(self.resoluted, false);
		assert_eq!(from, self.creator);
		assert!(winning_outcome == None || winning_outcome.unwrap() < self.outcomes);
		self.winning_outcome = winning_outcome;
		self.resoluted = true;
	}
	
	pub fn get_claimable(&self, from: String) -> u128 {
		assert_eq!(self.resoluted, true);
		let invalid = self.winning_outcome.is_none();
		let mut claimable = 0;

		if invalid {
			for (_, orderbook) in self.orderbooks.iter() {
				claimable += orderbook.get_spend_by(from.to_string());
			}
		} else {
			for (_, orderbook) in self.orderbooks.iter() {
			    // If person not in orderbook, continue
			    if orderbook.orders_by_user.get(&from).is_none() {
			        continue;
			    }
				claimable += orderbook.get_open_order_value_for(from.to_string());
			}
			let winning_orderbook = self.orderbooks.get(&self.winning_outcome.unwrap()).unwrap();
			if !winning_orderbook.orders_by_user.get(&from).is_none() {
                claimable += winning_orderbook.calc_claimable_amt(from);
            }
		}
		return claimable;
	}

	pub fn delete_orders_for(&mut self, from: String) {
		for orderbook_id in 0..self.outcomes {
			let orderbook = self.orderbooks.get_mut(&orderbook_id).unwrap();
			orderbook.delete_orders_for(from.to_string());
		}
	}
}


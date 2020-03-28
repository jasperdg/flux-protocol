use std::collections::{BTreeMap};
use std::cmp;
use borsh::{BorshDeserialize, BorshSerialize};
use near_bindgen::{near_bindgen};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub mod order;
pub type Order = order::Order;

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug)]
pub struct Orderbook {
	pub root: Option<u128>,
	pub market_order: Option<u128>,
	pub open_orders: BTreeMap<u128, BTreeMap<u128, Order>>,
	pub filled_orders: BTreeMap<u128, BTreeMap<u128, Order>>,
	pub spend_by_user: BTreeMap<String, u128>,
	pub orders_by_user: BTreeMap<String, Vec<String>>,
	pub nonce: u128,
	pub outcome_id: u64
}
impl Orderbook {
	pub fn new(outcome: u64) -> Self {
		Self {
			root: None,
			open_orders: BTreeMap::new(),
			filled_orders: BTreeMap::new(),
			spend_by_user: BTreeMap::new(),
			orders_by_user: BTreeMap::new(),
			market_order: None,
			nonce: 0,
			outcome_id: outcome,
		}
	}

    // Grabs latest nonce
	fn new_order_id(&mut self) -> u128 {
		let id = self.nonce;
		self.nonce = self.nonce + 1;
		return id;
	}

    // Places order in orderbook
	pub fn place_order(&mut self, from: String, outcome: u64, spend: u128, amt_of_shares: u128, price_per_share: u128, filled: u128, shares_filled: u128) {
		let order_id = self.new_order_id();
		let new_order = Order::new(from.to_string(), outcome, order_id, spend, amt_of_shares, price_per_share, filled, shares_filled);
		*self.spend_by_user.entry(from.to_string()).or_insert(0) += spend;

        // If all of spend is filled, state order is fully filled
        let left_to_spend = spend - filled;
		if left_to_spend < 100 {
			self.filled_orders.entry(price_per_share).or_insert(BTreeMap::new()).insert(order_id, new_order);
			return;
		}

        // If there is a remaining order, set this new order as the new market rate
		self.set_market_order(price_per_share);

        // Insert order into order map
		self.open_orders.entry(price_per_share).or_insert(BTreeMap::new()).insert(order_id, new_order);

		// Insert order into orders_by_user
		let account_value = format!("{outcome_id}::{price_per_share}::{order_id}", outcome_id=self.outcome_id, price_per_share=price_per_share, order_id=order_id);
		self.orders_by_user.entry(from.to_string()).or_insert(Vec::new()).push(account_value);
	}

    // Updates current market order price
	fn set_market_order(&mut self, price_per_share: u128) {
		let current_market_order_price = self.market_order;
		if current_market_order_price.is_none() {
			self.market_order = Some(price_per_share);
		} else {
			if let Some((current_market_price, _ )) = self.open_orders.iter().next() {
			    if price_per_share < *current_market_price {
                    self.market_order = Some(price_per_share);
                }
			}
		}
	}

    // Remove order from orderbook -- added price_per_share - if invalid order id passed behaviour undefined
	pub fn remove_order(&mut self, order_id: u128, price_per_share: u128) -> u128 {
		// Get orders at price
		let order_map = self.open_orders.get_mut(&price_per_share).unwrap();
        let order = order_map.get(&order_id).unwrap();
        let outstanding_spend = order.spend - order.filled;
        *self.spend_by_user.get_mut(&order.creator).unwrap() -= outstanding_spend;

        // Add back to filled if eligible, remove from user map if not
        if order.shares_filled > 0 {
            self.filled_orders.entry(price_per_share).or_insert(BTreeMap::new()).insert(order.id, order.clone());
        } else {
            let order_by_user_vec = self.orders_by_user.get_mut(&order.creator).unwrap();
            order_by_user_vec.swap_remove(order_id.try_into().unwrap());
            if order_by_user_vec.is_empty() {
                self.orders_by_user.remove(&order.creator);
            }
        }

        // Remove from order map
        order_map.remove(&order_id);
        if order_map.is_empty() {
            self.open_orders.remove(&price_per_share);
            if let Some((min_key, _ )) = self.open_orders.iter().next() {
                self.market_order = Some(*min_key);
            }
        }
        return outstanding_spend;
	}

	// TODO: Should catch these rounding errors earlier, right now some "dust" will be lost.
	pub fn fill_market_order(&mut self, mut amt_of_shares_to_fill: u128) {
	    let mut to_remove : Vec<(u128, u128)> = vec![];

		if let Some(( _ , current_order_map)) = self.open_orders.iter_mut().next() {
		    // Iteratively fill market orders until done
            for (order_id, order) in current_order_map.iter_mut() {
                if amt_of_shares_to_fill > 0 {
                    let shares_remaining_in_order = order.amt_of_shares - order.shares_filled;
                    let filling = cmp::min(shares_remaining_in_order, amt_of_shares_to_fill);

                    order.shares_filled += filling;
                    order.filled += filling * order.price_per_share;

                    if order.spend - order.filled < 100 { // some rounding errors here might cause some stack overflow bugs that's why this is build in.
                        to_remove.push((*order_id, order.price_per_share));
                        self.filled_orders.entry(order.price_per_share).or_insert(BTreeMap::new()).insert(order.id, order.clone());
                    }
                    amt_of_shares_to_fill -= filling;
                } else {
                    break;
                }
            }
		}

		for entry in to_remove {
		    self.remove_order(entry.0, entry.1);
		}
	}

	pub fn calc_claimable_amt(&self, from: String) -> u128 {
		let mut claimable = 0;
		let orders_by_user_vec = self.orders_by_user.get(&from).unwrap();

        // order_location = [outcome, price_per_share, order_id]
		for i in 0..orders_by_user_vec.len() {
		    let order_location: Vec<&str> = orders_by_user_vec[i].rsplit("::").collect();
		    // Try open orders
		    if !self.open_orders.get(&order_location[1].parse::<u128>().unwrap()).is_none() {
		        let open_order_map = self.open_orders.get(&order_location[1].parse::<u128>().unwrap()).unwrap();
		        if !open_order_map.get(&order_location[2].parse::<u128>().unwrap()).is_none() {
		            let order = open_order_map.get(&order_location[2].parse::<u128>().unwrap()).unwrap();
		            claimable += order.shares_filled * 100;
		        }
		    }

		    // Try filled orders
		    if !self.filled_orders.get(&order_location[1].parse::<u128>().unwrap()).is_none() {
		        let filled_order_map = self.filled_orders.get(&order_location[1].parse::<u128>().unwrap()).unwrap();
		        if !filled_order_map.get(&order_location[2].parse::<u128>().unwrap()).is_none() {
		            let filled_order = filled_order_map.get(&order_location[2].parse::<u128>().unwrap()).unwrap();
                    claimable += filled_order.shares_filled * 100;
                }
             }
        }
		return claimable;
	}

	pub fn delete_orders_for(&mut self, from: String) {
	    let mut to_delete : Vec<(u128, u128)> = vec![];
        self.spend_by_user.insert(from.to_string(), 0);
        let orders_by_user_vec = self.orders_by_user.get(&from).unwrap();

        for entry in orders_by_user_vec {
            let order_location: Vec<&str> = entry.rsplit("::").collect();
            let price_per_share = order_location[1].parse::<u128>().unwrap();
            let order_id = order_location[2].parse::<u128>().unwrap();
            to_delete.push((order_id, price_per_share));
        }

        for entry in to_delete {
            self.remove_order(entry.0, entry.1);
            self.remove_filled_order(entry.0, entry.1);
        }
	}

    fn remove_filled_order(&mut self, order_id : u128, price_per_share : u128) {
        // Get filled orders at price
        let filled_order_map = self.filled_orders.get_mut(&price_per_share).unwrap();
        let order = filled_order_map.get(&order_id).unwrap();
        // Remove order from user map
        let order_by_user_map = self.orders_by_user.get_mut(&order.creator).unwrap();
        order_by_user_map.remove(order_id.try_into().unwrap());
        if order_by_user_map.is_empty() {
            self.orders_by_user.remove(&order.creator);
        }
        filled_order_map.remove(&order_id);
        if filled_order_map.is_empty() {
            self.filled_orders.remove(&price_per_share);
        }
        return;
    }

	pub fn get_market_order_price(&self) -> u128 {
		return self.market_order.unwrap();
	}

	pub fn get_open_order_value_for(&self, from: String) -> u128 {
		let mut claimable = 0;
		let orders_by_user_vec = self.orders_by_user.get(&from).unwrap();

        // order_location = [outcome, price_per_share, order_id]
        for i in 0..orders_by_user_vec.len() {
            let order_location: Vec<&str> = orders_by_user_vec[i].rsplit("::").collect();
            // Try open orders
            if !self.open_orders.get(&order_location[1].parse::<u128>().unwrap()).is_none() {
                let open_order_map = self.open_orders.get(&order_location[1].parse::<u128>().unwrap()).unwrap();
                let order = open_order_map.get(&order_location[2].parse::<u128>().unwrap()).unwrap();
                claimable += order.shares_filled * 100;
            }
        }
		return claimable;
	}

	pub fn get_spend_by(&self, from: String) -> u128 {
		return *self.spend_by_user.get(&from).unwrap_or(&0);
	}
}
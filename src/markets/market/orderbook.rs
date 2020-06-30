use near_sdk::{
	near_bindgen, 
	env,
	collections::{
		UnorderedMap,
		TreeMap,
		Vector
	}
};
use std::cmp;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub mod order;
pub type Order = order::Order;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Orderbook {
	pub market_id: u64,
	pub root: Option<u128>,
	pub best_price: Option<u128>,
	pub open_orders: UnorderedMap<u128, Order>,
	pub filled_orders: UnorderedMap<u128, Order>,
	pub spend_by_user: UnorderedMap<String, u128>,
	pub orders_by_price: TreeMap<u128, UnorderedMap<u128, bool>>,
	pub liquidity_by_price: TreeMap<u128, u128>,
	pub orders_by_user: UnorderedMap<String, Vector<u128>>,
	pub claimed_orders_by_user: UnorderedMap<String, Vec<u128>>,
	pub nonce: u128,
	pub outcome_id: u64
}

impl Orderbook {
	pub fn new(
		market_id: u64,
		outcome: u64
	) -> Self {
		Self {
			market_id,
			root: None,
			open_orders: UnorderedMap::new(format!("open_orders:{}:{}", market_id, outcome).as_bytes().to_vec()),
			filled_orders: UnorderedMap::new(format!("filled_orders:{}:{}", market_id, outcome).as_bytes().to_vec()),
			spend_by_user: UnorderedMap::new(format!("spend_by_user:{}:{}", market_id, outcome).as_bytes().to_vec()),
			orders_by_price: TreeMap::new(format!("orders_by_price:{}:{}", market_id, outcome).as_bytes().to_vec()),
			liquidity_by_price: TreeMap::new(format!("liquidity_by_price:{}:{}", market_id, outcome).as_bytes().to_vec()),
			orders_by_user: UnorderedMap::new(format!("orders_by_user:{}:{}", market_id, outcome).as_bytes().to_vec()),
			claimed_orders_by_user: UnorderedMap::new(format!("claimed_orders_by_user:{}:{}", market_id, outcome).as_bytes().to_vec()),
			best_price: None,
			nonce: 0,
			outcome_id: outcome,
		}
	}

    // Grabs latest nonce
	fn new_order_id(
		&mut self
	) -> u128 {
		let id = self.nonce;
		self.nonce = self.nonce + 1;
		return id;
	}

    // Places order in orderbook
	pub fn place_order(
		&mut self, 
		account_id: String, 
		outcome: u64, 
		spend: u128, 
		amt_of_shares: u128, 
		price: u128, 
		filled: u128, 
		shares_filled: u128,
		affiliate_account_id: Option<String>
	){
		let order_id = self.new_order_id();
		let new_order = Order::new(account_id.to_string(), outcome, order_id, spend, amt_of_shares, price, filled, shares_filled, affiliate_account_id);

		let spend_by_user = self.spend_by_user.get(&account_id).unwrap_or(0);
		self.spend_by_user.insert(&account_id, &(spend_by_user + spend));

		let mut user_orders = self.orders_by_user.get(&account_id.to_string()).unwrap_or(Vector::new(format!("user_orders:{}:{}:{}", self.market_id, outcome, account_id).as_bytes().to_vec()));
		user_orders.push(&order_id);
		self.orders_by_user.insert(&account_id, &user_orders);

        // If all of spend is filled, state order is fully filled
		let left_to_spend = spend - filled;

		if left_to_spend < 100 {
			self.filled_orders.insert(&order_id, &new_order);
			return;
		}

        // If there is a remaining order, set this new order as the new market rate
		self.set_best_price(price);

        // Insert order into order map
		self.open_orders.insert(&order_id, &new_order);

		let mut orders_at_price = self.orders_by_price.get(&price).unwrap_or(UnorderedMap::new(format!("orders_by_price:{}:{}:{}", self.market_id, outcome,price).as_bytes().to_vec()));
		orders_at_price.insert(&order_id, &true);

		self.orders_by_price.insert(&price, &orders_at_price);

		let liquidity_by_price = self.liquidity_by_price.get(&price).unwrap_or(0);
		
		self.liquidity_by_price.insert(&price, &(liquidity_by_price + left_to_spend));
	}

    // Updates current market order price
	fn set_best_price(
		&mut self, 
		price: u128
	) {
		let current_best_price = self.best_price;
		if current_best_price.is_none() {
			self.best_price = Some(price);
		} else {
			if let Some((current_market_price, _ )) = self.open_orders.iter().next() {
			    if price > current_market_price {
                    self.best_price = Some(price);
                }
			}
		}
	}

    // // Remove order account_id orderbook -- added price - if invalid order id passed behaviour undefined
	pub fn remove_order(
		&mut self, 
		order_id: u128
	) -> u128 {
		// Store copy of order to remove
		let order = self.open_orders.get(&order_id).expect("order doesn't exist").clone();
		
		// Remove original order account_id open_orders
		self.open_orders.remove(&order.id);
		
		let outstanding_spend = order.spend - order.filled;
		
		let spend_by_user = self.spend_by_user.get(&order.creator).unwrap() - outstanding_spend;
		self.spend_by_user.insert(&order.creator, &spend_by_user);
		
		let liq_at_price = self.liquidity_by_price.get(&order.price).unwrap_or(0) - outstanding_spend;
		self.liquidity_by_price.insert(&order.price, &liq_at_price);
		
        // Add back to filled if eligible, remove account_id user map if not
        if order.shares_filled > 0 {
			self.filled_orders.insert(&order.id, &order);
        } else {
			let mut order_by_user_vec = self.orders_by_user.get(&order.creator).unwrap();
			
			// Keep all orders that aren't order_id using the retain method
			let mut index = 0;
			let mut index_to_delete: Option<u64> = None;
			
			for id in order_by_user_vec.iter() {
				if id == order_id {
					index_to_delete = Some(index);
					break;
				}
				index += 1;
			}
			
			if index_to_delete.is_some() { order_by_user_vec.swap_remove(index_to_delete.unwrap()); }
			
			self.orders_by_user.insert(&order.creator, &order_by_user_vec);
		}
		
		// Remove account_id order tree
		let mut orders_at_price = self.orders_by_price.get(&order.price).expect("no orders at this price");

		orders_at_price.remove(&order_id);
		self.orders_by_price.insert(&order.price, &orders_at_price);
		
        if orders_at_price.is_empty() {
			self.orders_by_price.remove(&order.price);

			self.orders_by_price.insert(&order.price, &orders_at_price);
            if let Some((min_key, _ )) = self.orders_by_price.iter().next() {
				self.best_price = Some(min_key);
            } else {
				self.best_price = None;
			}
		}
		
        return outstanding_spend;
	}

	// TODO: Should catch these rounding errors earlier, right now some "dust" will be lost.
	pub fn fill_best_orders(
		&mut self, 
		mut amt_of_shares_to_fill: u128
	) -> u128 {
	    let mut to_remove : Vec<(u128, u128)> = vec![];
		let mut filled_for_matches = 0;
		if let Some(( _ , current_order_map)) = self.orders_by_price.iter().next() {
			// Iteratively fill market orders until done
            for (order_id, _) in current_order_map.iter() {
				let mut order = self.open_orders.get(&order_id).unwrap();

                if amt_of_shares_to_fill > 0 {					
                    let shares_remaining_in_order = order.amt_of_shares - order.shares_filled;
					let filling = cmp::min(shares_remaining_in_order, amt_of_shares_to_fill);
					
					filled_for_matches += filling * order.price;

					let liquidity_at_price = self.liquidity_by_price.get(&order.price).expect("expected there to be liquidty") - filling * order.price;
					self.liquidity_by_price.insert(&order.price, &liquidity_at_price);

                    order.shares_filled += filling;
					order.filled += filling * order.price;

                    if order.spend - order.filled < 100 { // some rounding errors here might cause some stack overflow bugs that's why this is build in.
                        to_remove.push((order_id, order.price));
                        self.filled_orders.insert(&order.id, &order);
                    } else {
						self.open_orders.insert(&order.id, &order);
					}
                    amt_of_shares_to_fill -= filling;
                } else {
                    break;
                }
            }
		}

		for entry in to_remove {
		    self.remove_order(entry.0);
		}

		return filled_for_matches;
	}

	// fn get_mut_order_by_id(&mut self, order_id: &u128) -> &mut Order {
	// 	let open_order = self.open_orders.get_mut(order_id);

	// 	if open_order.is_some() {
	// 		return open_order.unwrap();
	// 	} else {
	// 		return self.filled_orders.get_mut(order_id).expect("order with this id doesn't exist");
	// 	}
	// }

	// // Returns claimable_if_valid
	// pub fn subtract_shares(
	// 	&mut self, 
	// 	shares: u128,
	// 	sell_price: u128,
	// ) -> u128 {
	// 	let orders_by_user = self.orders_by_user.get(&env::predecessor_account_id()).unwrap();
	// 	let mut shares_to_sell = shares;
	// 	let mut to_remove = vec![];
	// 	let mut claimable_if_valid = 0;
	// 	let mut spend_to_decrement = 0;

	// 	for order_id in orders_by_user.to_vec() {
	// 		if shares_to_sell > 0 {
	// 			let order = self.get_mut_order_by_id(&order_id);
	// 			let mut shares_to_calculate_spend = order.shares_filled;

	// 			if order.shares_filled <= shares_to_sell {
	// 				if order.is_filled() {
	// 					shares_to_sell -= order.shares_filled;
	// 					to_remove.push(order.id);
	// 				} else {
	// 					order.spend -= order.shares_filled * order.price;
	// 					order.filled = 0;
	// 					order.amt_of_shares -= order.shares_filled;
	// 					order.shares_filled = 0;
	// 				}
	// 			} else {
	// 				order.spend -= shares_to_sell * order.price;
	// 				order.filled -= shares_to_sell * order.price;
	// 				order.amt_of_shares -= shares_to_sell;
	// 				order.shares_filled = shares_to_sell;

	// 				shares_to_calculate_spend = shares_to_sell;
	// 				shares_to_sell = 0;
	// 			}

	// 			if order.price < sell_price {
	// 				claimable_if_valid += (sell_price - order.price) * shares_to_calculate_spend;
	// 				spend_to_decrement += shares_to_calculate_spend * order.price;
	// 			} else if order.price > sell_price {
	// 				spend_to_decrement = sell_price * shares_to_calculate_spend;
	// 			} else {
	// 				spend_to_decrement += shares_to_calculate_spend * order.price;
	// 			}

	// 		} else {
	// 			continue;
	// 		}
	// 	}

	// 	for order_id in to_remove {
	// 		self.remove_filled_order(order_id);
	// 	}

	// 	let spend_by_user = self.spend_by_user.get_mut(&env::predecessor_account_id()).expect("user doens't have any spend left");
	// 	*spend_by_user -= spend_to_decrement;

	// 	return claimable_if_valid;
	// }

	// pub fn calc_claimable_amt(
	// 	&self, 
	// 	account_id: String
	// ) -> (u128, HashMap<String, u128>) {
	// 	let mut claimable = 0;
	// 	let empty_vec: Vec<u128> = vec![];
	// 	let orders_by_user_vec = self.orders_by_user.get(&account_id).unwrap_or(&empty_vec);
	// 	let mut affiliates: HashMap<String, u128> = HashMap::new();
	// 	for i in 0..orders_by_user_vec.len() {
	// 		let order = self.open_orders
	// 		.get(&orders_by_user_vec[i])
	// 		.unwrap_or_else(|| {
	// 			return self.filled_orders
	// 			.get(&orders_by_user_vec[i])
	// 			.expect("Order by user doesn't seem to exist");
	// 		});
			
	// 		// If there is in fact an affiliate connected to this order
	// 		if !order.affiliate_account_id.is_none() {
	// 			let affiliate_account = order.affiliate_account_id.as_ref().unwrap();
	// 			affiliates
	// 			.entry(affiliate_account.to_string())
	// 			.and_modify(|balance| {
	// 				*balance += order.shares_filled * 100;
	// 			})
	// 			.or_insert(order.shares_filled * 100);
	// 		}
	// 		claimable += order.shares_filled * 100;
	// 	}

	// 	return (claimable, affiliates);
	// }

	// pub fn delete_orders_for(
	// 	&mut self, 
	// 	account_id: String
	// ) {
	// 	let empty_vec = &mut vec![];
	// 	let orders_by_user_copy = self.orders_by_user.get(&account_id).unwrap_or(empty_vec).clone();
		
	// 	self.spend_by_user
	// 	.entry(account_id.to_string())
	// 	.and_modify(|spend| { *spend = 0; })
	// 	.or_insert(0);

	// 	self.claimed_orders_by_user
	// 	.insert(account_id.to_string(), orders_by_user_copy);

	// 	*self.orders_by_user
	// 	.get_mut(&account_id)
	// 	.unwrap_or(empty_vec) = vec![];
	// }

    // fn remove_filled_order(
	// 	&mut self, 
	// 	order_id : u128
	// ) {
    //     // Get filled orders at price
    //     let order = self.filled_orders.get(&order_id).expect("order with this id doens't exist");
    //     // Remove order account_id user map
    //     let order_by_user_map = self.orders_by_user.get_mut(&order.creator).unwrap();
    //     order_by_user_map.remove(order_id.try_into().unwrap());
    //     if order_by_user_map.is_empty() {
    //         self.orders_by_user.remove(&order.creator);
    //     }
    //     self.filled_orders.remove(&order_id);
    // }

	// pub fn get_best_price(
	// 	&self
	// ) -> u128 {
	// 	return self.best_price.unwrap();
	// }

	// pub fn get_open_order_value_for(
	// 	&self, 
	// 	account_id: String
	// ) -> u128 {
	// 	let mut claimable = 0;
	// 	let empty_vec: Vec<u128> = vec![];
	// 	let orders_by_user_vec = self.orders_by_user.get(&account_id).unwrap_or(&empty_vec);

    //     for i in 0..orders_by_user_vec.len() {
	// 		let order_id = orders_by_user_vec[i];
	// 		let open_order_prom = self.open_orders.get(&order_id);
	// 		let order_is_open = !open_order_prom.is_none();
	// 		if order_is_open {
	// 			let order = self.open_orders.get(&order_id).unwrap();
	// 			claimable += order.spend - order.filled;
	// 		}
    //     }
	// 	return claimable;
	// }

	// pub fn get_spend_by(
	// 	&self, 
	// 	account_id: String
	// ) -> u128 {
	// 	return *self.spend_by_user.get(&account_id).unwrap_or(&0);
	// }

	pub fn get_share_balance(
		&self,
		account_id: String
	) -> u128 {
	// 	let empty_vec: &Vec<u128> = &vec![];
	// 	let orders_by_user = self.orders_by_user.get(&account_id).unwrap_or(empty_vec);

	// 	let mut balance = 0;
	// 	for order_id in orders_by_user {
	// 		let order = self.open_orders
	// 		.get(&order_id)
	// 		.unwrap_or_else(| | {
	// 			return  self.filled_orders.get(&order_id).expect("order with this id doesn't seem to exist")
	// 		});
	// 		balance += order.shares_filled;
	// 	}
	// 	return balance;
		return 0;
	}

	pub fn get_liquidity_at_price(
		&self, 
		price: u128
	) -> u128 {
	// 	let spend_liquidity = *self.liquidity_by_price.get(&price).unwrap_or(&0);
	// 	if spend_liquidity == 0 {
	// 		return 0
	// 	} else {
	// 		return spend_liquidity / price;
	// 	}
		return 0;
	}
}

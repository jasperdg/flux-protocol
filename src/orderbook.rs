use near_sdk::{
	env,
	collections::{
		UnorderedMap,
		TreeMap,
		Vector
	},
	json_types::{U128, U64},
};
use std::{
	cmp,
	convert::TryInto,
	collections::HashMap
};
use serde_json::json;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::order;
pub type Order = order::Order;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PriceData {
	pub share_liquidity: u128,
	pub orders: TreeMap<u128, Order>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountData {
	pub balance: u128,
	pub spent: u128,
	pub open_orders: TreeMap<u128, u128> // Check if we need order id or can just keep track of balance of open orders - for now open order id mapped to price
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Orderbook {
	pub market_id: u64,
	pub outcome_id: u64,
	pub price_data: TreeMap<u128, PriceData>,
	pub user_data: UnorderedMap<String, AccountData>,
	pub nonce: u128,
}

impl Orderbook {

	fn new_account(&self, account_id: String) -> AccountData {
		AccountData {
			balance: 0,
			spent: 0,
			open_orders: TreeMap::new(format!("{}:open_orders:{}:{}", account_id, self.market_id, self.outcome_id).as_bytes().to_vec())
		}
	}

	fn new_price(&self, price: u128) -> PriceData {
		PriceData {
			share_liquidity: 0,
			orders: TreeMap::new(format!("price_data:{}:{}:{}", self.market_id, self.outcome_id, price).as_bytes().to_vec())
		}
	}

	pub fn new(
		market_id: u64,
		outcome: u64
	) -> Self {
		Self {
			market_id,
			// open_orders: UnorderedMap::new(format!("open_orders:{}:{}", market_id, outcome).as_bytes().to_vec()),
			// filled_orders: UnorderedMap::new(format!("filled_orders:{}:{}", market_id, outcome).as_bytes().to_vec()),
			// spend_by_user: UnorderedMap::new(format!("spend_by_user:{}:{}", market_id, outcome).as_bytes().to_vec()),
			// orders_by_price: TreeMap::new(format!("orders_by_price:{}:{}", market_id, outcome).as_bytes().to_vec()),
			// liquidity_by_price: TreeMap::new(format!("liquidity_by_price:{}:{}", market_id, outcome).as_bytes().to_vec()),
			// orders_by_user: UnorderedMap::new(format!("orders_by_user:{}:{}", market_id, outcome).as_bytes().to_vec()),
			price_data: TreeMap::new(format!("price_data:{}:{}", market_id, outcome).as_bytes().to_vec()),
			user_data: UnorderedMap::new(format!("user_data:{}:{}", market_id, outcome).as_bytes().to_vec()),
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
	pub fn new_order(
		&mut self, 
		account_id: String, 
		outcome: u64, 
		spend: u128, 
		shares: u128, 
		price: u128, 
		filled: u128, 
		shares_filled: u128,
		affiliate_account_id: Option<String>
	){
		let order_id = self.new_order_id();
		let new_order = Order::new(order_id, account_id.to_string(), spend, filled, shares, shares_filled, price, affiliate_account_id.clone());
		let mut user_data = self.user_data.get(&account_id).unwrap_or(self.new_account(account_id.to_string()));
		user_data.balance += shares_filled;
		user_data.spent += filled;
		
		let left_to_spend = spend - filled;

		let mut fill_price = 0;

		if shares_filled > 0 {
			fill_price = filled / shares_filled;
		}
		

		// TODO: add to affiliate_earnings
		// if left_to_spend < 100 the order counts as filled
		if left_to_spend < 100 {
			self.user_data.insert(&account_id, &user_data);

			env::log(
				json!({
					"type": "order_filled_at_placement".to_string(),
					"params": {
						"order_id": U128(order_id),
						"market_id": U64(self.market_id),
						"account_id": account_id, 
						"outcome": U64(outcome), 
						"spend":  U128(spend),
						"shares":  U128(shares),
						"fill_price": U128(fill_price),
						"price":  U128(price),
						"filled": U128(filled), 
						"shares_filling": U128(shares_filled),
						"shares_filled": U128(shares_filled),
						"affiliate_account_id": affiliate_account_id,
						"block_height": U64(env::block_index())
					}
				})
				.to_string()
				.as_bytes()
			);
			return;
		}

		// TODO: expect that we don't need a reference to the order
		user_data.open_orders.insert(&order_id, &price);
		self.user_data.insert(&account_id, &user_data);

		let mut price_data = self.price_data.get(&price).unwrap_or(self.new_price(price));
		price_data.orders.insert(&order_id, &new_order);
		price_data.share_liquidity += (spend - filled) / price;
		self.price_data.insert(&price, &price_data);

		env::log(
			json!({
				"type": "order_placed".to_string(),
				"params": {
					"order_id": U128(order_id),
					"market_id": U64(self.market_id),
					"account_id": account_id, 
					"outcome": U64(outcome), 
					"spend":  U128(spend),
					"fill_price": U128(fill_price),
					"shares_filling": U128(shares_filled),
					"shares":  U128(shares),
					"price":  U128(price),
					"filled": U128(filled), 
					"shares_filled": U128(shares_filled),
					"affiliate_account_id": affiliate_account_id,
					"block_height": U64(env::block_index())
				}
			})
			.to_string()
			.as_bytes()
		);
	}


	pub fn cancel_order(&mut self, order: Order) -> u128 {
		let mut price_data = self.price_data.get(&order.price).unwrap();
		let mut user_data = self.user_data.get(&order.creator).unwrap();

		let to_return = order.spend - order.filled; 

		price_data.share_liquidity -= to_return / order.price;
		price_data.orders.remove(&order.id);

		if price_data.orders.len() == 0 {
			env::log(format!("removing orders").to_string().as_bytes());
			self.price_data.remove(&order.price);
		} else {
			self.price_data.insert(&order.price, &price_data);
		}
		
		user_data.open_orders.remove(&order.id);
		
		self.user_data.insert(&order.creator, &user_data);

		return to_return;
	}

	fn log_order_filled(&self, order: &Order, shares_to_fill: u128) {
		env::log(
			json!({
			"type": "order_filled".to_string(),
				"params": {
					"market_id": U64(self.market_id),
					"outcome": U64(self.outcome_id),
					"order_id": U128(order.id),
					"account_id": order.creator,
					"shares_filling": U128(shares_to_fill),
					"filled": U128(order.filled + shares_to_fill * order.price),
					"price": U128(order.price),
					"fill_price": U128(order.price),
					"shares_filled": U128(order.shares_filled + shares_to_fill),
					"block_height": U64(env::block_index())
				}
			})
			.to_string()
			.as_bytes()
		);
	}

	// TODO: add to affiliate_earnings
	pub fn fill_order(
		&mut self, 
		mut order: Order, 
		shares_to_fill: u128,
		close_order: bool
	) {
		let mut user_data = self.user_data.get(&order.creator).expect("order is owned by non-existent user");
		let mut price_data = self.price_data.get(&order.price).expect("no price data for this order");

		user_data.balance += shares_to_fill;
		user_data.spent += shares_to_fill * order.price;
		price_data.share_liquidity -= shares_to_fill;


		if close_order {
			user_data.open_orders.remove(&order.id);
			price_data.orders.remove(&order.id);
		}  else {
			order.filled += shares_to_fill * order.price;
			order.shares_filled += shares_to_fill;
			price_data.orders.insert(&order.id, &order);
		}

		if price_data.orders.len() == 0 {
			env::log(format!("removing orders").to_string().as_bytes());
			self.price_data.remove(&order.price);
		} else {
			self.price_data.insert(&order.price, &price_data);
		}

		self.user_data.insert(&order.creator, &user_data);
		self.log_order_filled(&order, shares_to_fill);
	}

	pub fn fill_best_orders(
		&mut self, 
		mut shares_to_fill: u128
	) {
		let fill_price = match self.price_data.max() {
			Some(price) => price,
			None => return
		};

		let orders = self.price_data.get(&fill_price).expect("this price shouldn't exist if there are no orders to be filled").orders;

		for (order_id, order) in orders.iter() {
			if shares_to_fill < 1 { break;} 
			let shares_fillable_for_order = (order.spend - order.filled) / order.price;

			// TODO: test that panic is never called
			if shares_fillable_for_order == 0 {panic!("should never be 0")}

			let filling = cmp::min(shares_fillable_for_order, shares_to_fill); 

			env::log(format!("to fill: {}, fillable: {}", shares_to_fill, shares_fillable_for_order).to_string().as_bytes());
			
			if shares_to_fill < shares_fillable_for_order {
				self.fill_order(order, filling, false);
				break;
			} else if shares_to_fill > shares_fillable_for_order {
				self.fill_order(order, filling, true);
			} else {
				self.fill_order(order, filling, true);
				break;
			}
			
			env::log(format!("filling: {}", filling).to_string().as_bytes());
			shares_to_fill -= filling;
		}
	}

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

	// 	for (order_id, _) in orders_by_user.iter() {
	// 		if shares_to_sell > 0 {
	// 			let (mut order, state) = self.get_order_by_id(&order_id);
	// 			let mut shares_to_calculate_spend = order.shares_filled;

	// 			// if there's more or equal shares to be sold than ths
	// 			if order.shares_filled <= shares_to_sell {
	// 				// check if share that is being sold is filled
	// 				if order.is_filled() {
	// 					shares_to_sell -= order.shares_filled;
	// 					to_remove.push(order.id);
	// 				} else {
	// 					// if the shares sold are part of an open order - adjust said order
	// 					order.spend -= order.shares_filled * order.price;
	// 					order.filled = 0;
	// 					order.amt_of_shares -= order.shares_filled;
	// 					order.shares_filled = 0;
	// 					env::log(
	// 						json!({
	// 							"type": "sold_fill_from_order".to_string(),
	// 							"params": {
	// 								"order_id": U128(order_id),
	// 								"market_id": U64(self.market_id),
	// 								"outcome": U64(self.outcome_id),
	// 								"updated_spend": U128(order.spend),
	// 								"updated_filled": U128(order.filled),
	// 								"updated_amt_of_shares": U128(order.amt_of_shares),
	// 								"upated_shares_filled": U128(order.shares_filled)
	// 							}
	// 						})
	// 						.to_string()
	// 						.as_bytes()
	// 					);
	// 				}
	// 			} else {
	// 				order.spend -= shares_to_sell * order.price;
	// 				order.filled -= shares_to_sell * order.price;
	// 				order.amt_of_shares -= shares_to_sell;
	// 				order.shares_filled = shares_to_sell;
	// 				shares_to_calculate_spend = shares_to_sell;
	// 				shares_to_sell = 0;

	// 				env::log(
	// 					json!({
	// 						"type": "sold_fill_from_order".to_string(),
	// 						"params": {
	// 							"order_id": U128(order_id),
	// 							"market_id": U64(self.market_id),
	// 							"outcome": U64(self.outcome_id),
	// 							"updated_spend": U128(order.spend),
	// 							"updated_filled": U128(order.filled),
	// 							"updated_amt_of_shares": U128(order.amt_of_shares),
	// 							"upated_shares_filled": U128(order.shares_filled)
	// 						}
	// 					})
	// 					.to_string()
	// 					.as_bytes()
	// 				);
	// 			}

	// 			if order.price < sell_price {
	// 				claimable_if_valid += (sell_price - order.price) * shares_to_calculate_spend;
	// 				spend_to_decrement += shares_to_calculate_spend * order.price;
	// 			} else if order.price > sell_price {
	// 				spend_to_decrement = sell_price * shares_to_calculate_spend;
	// 			} else {
	// 				spend_to_decrement += shares_to_calculate_spend * order.price;
	// 			}
				
	// 			if state == "open".to_string() {
	// 				self.open_orders.insert(&order.id, &order);
	// 			} else {
	// 				self.filled_orders.insert(&order.id, &order);
	// 			}

	// 		} else {
	// 			break;
	// 		}
	// 	}

	// 	for order_id in to_remove {
	// 		self.remove_filled_order(order_id);
	// 	}

	// 	let mut spend_by_user = self.spend_by_user.get(&env::predecessor_account_id()).expect("user doens't have any spend left");
	// 	spend_by_user -= spend_to_decrement;
	// 	self.spend_by_user.insert(&env::predecessor_account_id(), &spend_by_user);
	// 	return claimable_if_valid;
	// }

	// pub fn calc_claimable_amt(
	// 	&self, 
	// 	account_id: String
	// ) -> (u128, HashMap<String, u128>) {
	// 	let mut claimable = 0;
	// 	let empty_vec: Vec<u128> = vec![];
	// 	let orders_by_user_map = self.orders_by_user.get(&account_id).unwrap_or(UnorderedMap::new(format!("user_orders:{}:{}:{}", self.market_id, self.outcome_id, account_id).as_bytes().to_vec()));
	// 	let mut affiliates: HashMap<String, u128> = HashMap::new();
	// 	for (order_id, _) in orders_by_user_map.iter() {
	// 		let order = self.open_orders
	// 		.get(&order_id)
	// 		.unwrap_or_else(|| {
	// 			return self.filled_orders
	// 			.get(&order_id)
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

    // fn remove_filled_order(
	// 	&mut self, 
	// 	order_id : u128
	// ) {
    //     // Get filled orders at price
    //     let order = self.filled_orders.get(&order_id).expect("order with this id doens't exist");
    //     // Remove order account_id user map
    //     let mut order_by_user_map = self.orders_by_user.get(&order.creator).unwrap();
	// 	order_by_user_map.remove(&order_id);
	// 	self.orders_by_user.insert(&order.creator, &order_by_user_map);
    //     if order_by_user_map.is_empty() {
    //         self.orders_by_user.remove(&order.creator);
    //     }
    //     self.filled_orders.remove(&order_id);
    // }

	// // pub fn get_best_price(
	// // 	&self
	// // ) -> u128 {
	// // 	return self.best_price.unwrap();
	// // }

	// // pub fn get_open_order_value_for(
	// // 	&self, 
	// // 	account_id: String
	// // ) -> u128 {
	// // 	let mut claimable = 0;
	// // 	let empty_vec: Vec<u128> = vec![];
	// // 	let orders_by_user_vec = self.orders_by_user.get(&account_id).unwrap_or(UnorderedMap::new(format!("user_orders:{}:{}:{}", self.market_id, self.outcome_id, account_id).as_bytes().to_vec()));

    // //     for (order_id, _) in orders_by_user_vec.iter() {
	// // 		let open_order_prom = self.open_orders.get(&order_id);
	// // 		let order_is_open = !open_order_prom.is_none();
	// // 		if order_is_open {
	// // 			let order = self.open_orders.get(&order_id).unwrap();
	// // 			claimable += order.spend - order.filled;
	// // 		}
    // //     }
	// // 	return claimable;
	// // }

	// pub fn get_spend_by(
	// 	&self, 
	// 	account_id: String
	// ) -> u128 {
	// 	return self.spend_by_user.get(&account_id).unwrap_or(0);
	// }

	// pub fn get_share_balance(
	// 	&self,
	// 	account_id: String
	// ) -> u128 {
	// 	let orders_by_user = self.orders_by_user.get(&account_id).unwrap_or(UnorderedMap::new(format!("user_orders:{}:{}:{}", self.market_id, self.outcome_id, account_id).as_bytes().to_vec()));

	// 	let mut balance = 0;
	// 	for (order_id, _) in orders_by_user.iter() {
	// 		let order = self.open_orders
	// 		.get(&order_id)
	// 		.unwrap_or_else(| | {
	// 			return  self.filled_orders.get(&order_id).expect("order with this id doesn't seem to exist")
	// 		});
	// 		balance += order.shares_filled;
	// 	}
	// 	return balance;
	// }

	// pub fn get_liquidity_at_price(
	// 	&self, 
	// 	price: u128
	// ) -> u128 {
	// 	let spend_liquidity = self.liquidity_by_price.get(&price).unwrap_or(0);
	// 	if spend_liquidity == 0 {
	// 		return 0
	// 	} else {
	// 		return spend_liquidity / price;
	// 	}
	// }
}

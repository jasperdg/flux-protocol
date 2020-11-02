use near_sdk::{
	AccountId,
	collections::{
		UnorderedMap,
		TreeMap,
	},
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};
use std::{
	cmp,
};

/* Import order impl */
use crate::order;
/* Import logger impl */
use crate::logger;

/* Declare order type */
pub type Order = order::Order;

/**
 * @notice PriceData is a struct that holds total liquidity denominated in shares(1e16) and an ordered Map of orders (order_id => Order) for a certain price
 */
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PriceData {
	pub share_liquidity: u128,
	pub orders: TreeMap<u128, Order>
}

/**
 * @notice AccountData is a struct that keeps some state for each participant that purchased shares of the orderbook's outcome
 */
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountData {
	pub balance: u128, // The user's balance denominated in shares (1e16)
	pub spent: u128, // How much the user has spent (denominated in 1e18)
	pub to_spend: u128, // How much is still to be spend (in open orders)
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Orderbook {
	pub market_id: u64,
	pub outcome_id: u8,
	pub price_data: TreeMap<u16, PriceData>, // Ordered map where price => PriceData
	pub user_data: UnorderedMap<AccountId, AccountData>, // Unordered map where account_id => AccountData
	pub nonce: u128, // Incrementing nonce to decide on order_ids
}

impl Orderbook {
	/**
	 * @notice Initialize new orderbook struct
	 */
	pub fn new(
		market_id: u64,
		outcome: u8
	) -> Self {
		Self {
			market_id,
			price_data: TreeMap::new(format!("price_data:{}:{}", market_id, outcome).as_bytes().to_vec()),
			user_data: UnorderedMap::new(format!("user_data:{}:{}", market_id, outcome).as_bytes().to_vec()),
			nonce: 0,
			outcome_id: outcome,
		}
	}

	/**
	 * @notice Initialize a new AccountData instance
	 * @return Returns AccountData struct
	 */
	fn new_account(&self) -> AccountData {
		AccountData {
			balance: 0,
			spent: 0,
			to_spend: 0,
		}
	}

	/**
	 * @notice Initialize a new PriceData instance
	 * @return Returns PriceData struct
	 */
	fn new_price(&self, price: u128) -> PriceData {
		PriceData {
			share_liquidity: 0,
			orders: TreeMap::new(format!("price_data:{}:{}:{}", self.market_id, self.outcome_id, price).as_bytes().to_vec())
		}
	}

    /**
	 * @notice Gets latest nonce and increments for next order_id 
	 * @return Returns u128 representing a new order_id
	 */ 
	fn new_order_id(
		&mut self
	) -> u128 {
		let id = self.nonce;
		self.nonce = self.nonce + 1;
		return id;
	}

    /**
	 * @notice Creates a new order and stores it
	 */
	pub fn new_order(
		&mut self,
		market_id: u64,
		account_id: AccountId, 
		outcome: u8, 
		spend: u128, 
		shares: u128, 
		price: u16, 
		filled: u128, 
		shares_filled: u128,
		affiliate_account_id: Option<AccountId>
	){
		let order_id = self.new_order_id();
		/* Create new order instance */
		let new_order = Order::new(order_id, account_id.to_string(), market_id, spend, filled, shares, shares_filled, price, affiliate_account_id.clone());

		/* Get user_data and if it doesn't exist create new instance */
		let mut user_data = self.user_data.get(&account_id).unwrap_or(self.new_account());

		/* Update user data */
		user_data.balance += shares_filled;
		user_data.spent += filled;
		user_data.to_spend += spend;
		
		logger::log_update_user_balance(account_id.to_string(), market_id, outcome, user_data.balance, user_data.to_spend, user_data.spent);
		
		/* Calculate how much of the order is still open */
		let left_to_spend = spend - filled;

		/* Calculate the average fill_price if anything was filled */
		let mut fill_price = 0;
		if shares_filled > 0 {
			fill_price = filled / shares_filled;
		}
		
		self.user_data.insert(&account_id, &user_data);
		
		/* if left_to_spend < 100 the order counts as filled to avoid rounding errors which produce overflow errors */
		if left_to_spend < 100 {
			/* Return if filled */
			logger::log_order_filled_at_placement(&new_order, outcome, fill_price);
			return;
		}
		
		/* Store the order by updating the price data, if there were no orders at this order's price create a new order instance */
		let mut price_data = self.price_data.get(&price).unwrap_or(self.new_price(price as u128));
		/* Insert order into open orders at price */
		price_data.orders.insert(&order_id, &new_order);
		/* Update liquidity by shares still open */
		price_data.share_liquidity += (spend - filled) / price as u128;
		/* Re-insert price_data to update state */
		self.price_data.insert(&price, &price_data);

		logger::log_order_placed(&new_order, outcome, fill_price);
	}

	/** 
	 * @notice Cancel an open order for a user
	 * @return Returns the amount of tokens to send to the user 
	*/
	pub fn cancel_order(&mut self, order: Order) -> u128 {
		let mut price_data = self.price_data.get(&order.price).expect("There are no orders at this price");
		let mut user_data = self.user_data.get(&order.creator).expect("There are no orders for this user");

		/* Calculate amount of tokens that are open on the specific order */
		let to_return = order.spend - order.filled; 

		/* Update price data */
		price_data.share_liquidity -= to_return / order.price as u128;
		price_data.orders.remove(&order.id);

		/* If there are no orders left at the price remove the price_data entry for this price, else re-insert the price_data to update state */
		if price_data.orders.len() == 0 {
			self.price_data.remove(&order.price);
		} else {
			self.price_data.insert(&order.price, &price_data);
		}
		
		/* Update user_data */
		user_data.to_spend -= order.spend - order.filled;
		/* Re-insert user_data to update state */
		self.user_data.insert(&order.creator, &user_data);

		logger::log_update_user_balance(order.creator.to_string(), order.market_id, self.outcome_id, user_data.balance, user_data.to_spend, user_data.spent);
		logger::log_order_closed(&order, self.market_id, self.outcome_id);

		return to_return;
	}

	/**
	 * @notice Fills best orders up to a certain amount of shares
	 * @return Returns the amount of shares filled
	 */
	pub fn fill_best_orders(
		&mut self, 
		mut shares_to_fill: u128
	) -> u128 {

		/* Get the highest key in price_data representing the best available order if there are no keys return 0 */
		let fill_price = match self.price_data.max() {
			Some(price) => price,
			None => return 0
		};

		/* Get the open orders at the best_price */
		let orders = self.price_data.get(&fill_price).expect("this price shouldn't exist if there are no orders to be filled").orders.to_vec();

		/* Keep track of how many shares we filled */
		let mut shares_filled = 0;
		
		/* Loop through all orders at the best price */
		for (_, order) in orders.iter() {
			/* If there ano more shares to fill stop loop */
			if shares_to_fill == 0 { break;} 

			/* Calc how many shares can still be filled for this order */
			let shares_fillable_for_order = (order.spend - order.filled) / order.price as u128;

			/* Get the min amount of shares fillable between shares_to_fill and shares_fillable_for_order */
			let filling = cmp::min(shares_fillable_for_order, shares_to_fill); 
			
			/* Increment shares_filled by filling */
			shares_filled += filling;

			/* If there are less shares to fill than the best_order we fill the order and stop the loop */
			/* If there are more shares to fill than the best_order we fill the order and go to the next iteration */
			if shares_to_fill <= shares_fillable_for_order {
				/* If the shares_to_fill are equal to the amount of shares this best_order has we need to close the best_order */
				let close_order = shares_to_fill == shares_fillable_for_order;
				self.fill_order(order.clone(), filling, close_order);
				break;
			} else if shares_to_fill > shares_fillable_for_order {
				self.fill_order(order.clone(), filling, true);
			}

			/* Decrement shares_to_fill by the amount of shares we just filled */
			shares_to_fill -= filling;
		}

		return shares_filled;
	}

	/**
	 * @notice Fills an order
	 */
	fn fill_order(
		&mut self, 
		mut order: Order, 
		shares_to_fill: u128,
		close_order: bool
	) {

		let mut user_data = self.user_data.get(&order.creator).expect("no user_data available for user");
		let mut price_data = self.price_data.get(&order.price).expect("no price_data available for price");

		/* Update price and user data accordingly */
		user_data.balance += shares_to_fill;
		user_data.spent += shares_to_fill * order.price as u128;
		/* Re-insert user_data to update state */

		self.user_data.insert(&order.creator, &user_data);

		price_data.share_liquidity -= shares_to_fill;

		/* If the order has be closed remove it from open orders */
		/* Else update order and re-insert it to update price_data */
		if close_order {
			price_data.orders.remove(&order.id);
			logger::log_order_closed(&order, self.market_id, self.outcome_id);
		}  else {
			order.filled += shares_to_fill * order.price as u128;
			order.shares_filled += shares_to_fill;
			price_data.orders.insert(&order.id, &order);
		}

		/* Remove price_data for price if there are no more open orders */
		/* Else re-insert into price_data to update state */
		if price_data.orders.len() == 0 {
			self.price_data.remove(&order.price);
		} else {
			self.price_data.insert(&order.price, &price_data);
		}

		logger::log_order_filled(&order, shares_to_fill, self.market_id, self.outcome_id);
		logger::log_update_user_balance(order.creator, order.market_id, self.outcome_id, user_data.balance, user_data.to_spend, user_data.spent);
	}

	/**
	 * @notice Calculate share depth down to a min_price
	 * @return Returns a tuple where the first index is depth and the second index is the average price to be paid per share
	 */
	pub fn get_depth_down_to_price(&self, max_shares: u128, min_price: u16) -> (u128, u128) {
		/* Get the best price for outcome */
		let mut best_price = self.price_data.max().unwrap_or(0);

		/* Keep track of total available liquidity */
		let mut depth = 0;
		/* Sum of products of shares and prices */
		let mut depth_price_prod_sum = 0;

		/* Loop through all the price from best to worst */
		while best_price >= min_price && max_shares > depth {
			/* Calculate how many shares are left to fill */
			let shares_left_to_fill = max_shares - depth;
			/* Get the price_data at the current best_price */
			let price_data = self.price_data.get(&best_price).expect("Expected there to be a value at this key");
			/* Calculate the minimal amount of shares to fill between open liquidity and max_shares */
			let liquidity = cmp::min(shares_left_to_fill, price_data.share_liquidity);

			/* Increment price sum by product of liquidity and price */
			depth_price_prod_sum += liquidity * best_price as u128;

			/* Increment depth by share_liquidity */
			depth += liquidity;

			/* Update best price to next best price */
			best_price = self.price_data.lower(&best_price).unwrap_or(0);
		}

		if depth == 0 {return (0, 0);}

		return (cmp::min(max_shares, depth), depth_price_prod_sum / depth);
	}
}

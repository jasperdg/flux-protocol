use std::string::String;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, env};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Order {
	pub id: u128,
	pub creator: String,
	pub market_id: u64,
	pub spend: u128,
	pub filled: u128,
	pub shares: u128,
	pub shares_filled: u128,
	pub price: u128,
	pub affiliate_account_id: Option<String>
}

impl Order {
	pub fn new(
		id: u128,
		creator: String,
		market_id: u64,
		spend: u128, 
		filled: u128, 
		shares: u128, 
		shares_filled: u128,
		price: u128, 
		affiliate_account_id: Option<String>
	) -> Self {
		Order {
			id,
			creator,
			market_id,
			spend,
			filled,
			shares,
			shares_filled,
			price,
			affiliate_account_id
		}
	}

	pub fn is_filled(&self) -> bool {
		return self.spend - self.filled < 100;
	}
}
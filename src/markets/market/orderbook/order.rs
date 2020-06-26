use std::string::String;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, env};

#[near_bindgen]
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Order {
	pub id: u128,
	pub creator: String,
	pub outcome: u64,
	pub spend: u128,
	pub amt_of_shares: u128,
	pub price: u128,
	pub filled: u128,
	pub shares_filled: u128,
	pub creation_time: u128,
	pub affiliate_account_id: Option<String>
}

impl Order {
	pub fn new(
		creator: String, 
		outcome: u64, 
		id: u128, 
		spend:u128, 
		amt_of_shares: u128, 
		price: u128, 
		filled: u128, 
		shares_filled: u128,
		affiliate_account_id: Option<String>
	) -> Self {
		let creation_time = env::block_timestamp() / 1000000;

		Order {
			id,
			creator,
			outcome,
			spend,
			amt_of_shares,
			price,
			filled,
			shares_filled,
			creation_time: creation_time as u128,
			affiliate_account_id
		}
	}

	pub fn is_filled(&self) -> bool {
		return self.amt_of_shares == self.shares_filled;
	}
}
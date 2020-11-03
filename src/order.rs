use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, Clone)]
pub struct Order {
	pub id: u128,
	pub creator: AccountId,
	pub market_id: u64,
	pub spend: u128,
	pub filled: u128,
	pub shares: u128,
	pub shares_filled: u128,
	pub price: u16,
	pub affiliate_account_id: Option<AccountId>
}

impl Order {
	pub fn new(
		id: u128,
		creator: AccountId,
		market_id: u64,
		spend: u128, 
		filled: u128, 
		shares: u128, 
		shares_filled: u128,
		price: u16, 
		affiliate_account_id: Option<AccountId>
	) -> Self {
		Self {
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
}
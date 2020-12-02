use near_sdk::{
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};

/**
 * @notice `AccountOutcomeData` is a struct that keeps some state for each participant that purchased shares of the orderbook's outcome
 */
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountOutcomeData {
	pub shares_balance: u128, // The user's balance in shares (1e16)
	pub spent: u128, // How much the user has spent in tokens (denominated in 1e18)
	pub to_spend: u128, // How much is still to be spend in tokens (in open orders)
}

impl AccountOutcomeData {
    pub fn new() -> Self {
		Self {
			shares_balance: 0,
			spent: 0,
			to_spend: 0,
		}
	}

	pub fn calc_avg_buy_price(&self) -> u128 {
		self.spent / self.shares_balance
    }
    
    pub fn update_balances(
        &mut self, 
        shares_filled: u128,
    ) {
        let avg_buy_price = self.calc_avg_buy_price();
        let value = shares_filled * avg_buy_price;
		/* Subtract user stats according the amount of shares sold */
        self.shares_balance -= shares_filled;
		self.to_spend -= value;
		self.spent -= value;
    }
}
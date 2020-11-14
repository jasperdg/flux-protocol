use near_sdk::{
    AccountId,
    collections::UnorderedMap,
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};
use std::cmp::Ordering;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidityEscrow {
	pub claimable_if_valid: UnorderedMap<AccountId, u128>,
	pub claimable_if_invalid: UnorderedMap<AccountId, u128>,
}

impl ValidityEscrow {
    pub fn get_owed(
        &self, 
        account_id: &AccountId,
        valid_market: bool
    ) -> u128 {
        if valid_market {
            self.claimable_if_valid.get(account_id).unwrap_or(0)
        } else {
            self.claimable_if_invalid.get(account_id).unwrap_or(0)
        }
    }


    pub fn update_escrow(
        &mut self,
        account_id: &AccountId,
        shares_filled: u128,
        avg_sell_price: u128,
        avg_buy_price: u128
    ) {
        match avg_sell_price.cmp(&avg_buy_price) {
            Ordering::Greater => {
                let to_add_to_escrow = shares_filled * (avg_sell_price - avg_buy_price);
                let curr_claimable_if_valid = self.claimable_if_valid.get(account_id).unwrap_or(0);
                let new_claimable_if_valid = to_add_to_escrow + curr_claimable_if_valid;
    
                self.claimable_if_valid.insert(account_id, &new_claimable_if_valid);
            },
            Ordering::Less => {
                let to_add_to_escrow = shares_filled * (avg_buy_price - avg_sell_price);
                let curr_claimable_if_invalid = self.claimable_if_invalid.get(account_id).unwrap_or(0);
                let new_claimable_if_invalid = to_add_to_escrow + curr_claimable_if_invalid;
    
                self.claimable_if_invalid.insert(account_id, &new_claimable_if_invalid);
            },
            _ => ()

        };
    }
}
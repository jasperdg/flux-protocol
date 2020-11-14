use near_sdk::{
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};

/*** Import market implementation ***/
use crate::market::Market;
/*** Import utils ***/
use crate::utils;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fees {
	pub creator_fee_percentage: u32, // Denominated in 1e4 so 2 percentage point percentage
	pub resolution_fee_percentage: u32, // Denominated in 1e4 so 2 percentage point percentage
	pub affiliate_fee_percentage: u32, // Denominated in 1e4 so 2 percentage point percentage - this is a percentage of the creator fee not of the total amount
}

impl Fees {
    pub fn calc_creator_fee(
        &self,
        feeable: u128,
        market: &Market,
    ) -> u128 {
        let creator_fee_percentage = utils::get_creator_fee_percentage(&market);

        match creator_fee_percentage {
            0 => 0,
            _ => utils::calc_fee(feeable, creator_fee_percentage)
        }
    }

    pub fn calc_total_fee(
        &self,
        feeable: u128,
        market: &Market
    ) -> u128 {
        utils::calc_fee(feeable, self.resolution_fee_percentage) + self.calc_creator_fee(feeable, market)
    }
}
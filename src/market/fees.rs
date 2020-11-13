use near_sdk::{
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};

/*** Import market implementation ***/
use crate::market::Market;
/*** Import constants ***/
use crate::constants;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Fees {
	pub creator_fee_percentage: u32, // Denominated in 1e4 so 2 percentage point percentage
	pub resolution_fee_percentage: u32, // Denominated in 1e4 so 2 percentage point percentage
	pub affiliate_fee_percentage: u32, // Denominated in 1e4 so 2 percentage point percentage - this is a percentage of the creator fee not of the total amount
}

impl Fees {
    /**
     * @notice Returns the market's `creator_fee`. If the market is resoluted as invalid the creator's fee is slashed so this method returns 0. 
     * @param market A reference to the market where the `fee_percentage` should be returned from
     * @return Returns a u128 integer representing the `creator_fee_percentage` denominated in 1e4, meaning 1 == 0.01%
     */
    pub fn get_creator_fee_percentage(
        &self, 
        market: &Market
    ) -> u32 {
        match market.winning_outcome {
            Some(_) => market.fees.creator_fee_percentage,
            None => 0
        }
    }

    pub fn calc_fee(
        &self, 
        feeable: u128, 
        fee_percentage: u32
    ) -> u128 {
        feeable * u128::from(fee_percentage) / u128::from(constants::PERCENTAGE_PRECISION)
    }

    pub fn calc_creator_fee(
        &self,
        feeable: u128,
        market: &Market,
    ) -> u128 {
        let creator_fee_percentage = self.get_creator_fee_percentage(&market);

        match creator_fee_percentage {
            0 => 0,
            _ => self.calc_fee(feeable, creator_fee_percentage)
        }
    }

    pub fn calc_total_fee(
        &self,
        feeable: u128,
        market: &Market
    ) -> u128 {
        self.calc_fee(feeable, self.resolution_fee_percentage) + self.calc_creator_fee(feeable, market)
    }
}
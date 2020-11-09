use near_sdk::{
	env,
	AccountId,
	collections::{
		UnorderedMap
	},
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};

/*** Import constants methods ***/
use crate::constants;

/** 
 * @notice Struct of a resolution window, meant to display both resolution and dispute progression and state
 * 
 * */
 #[derive(BorshDeserialize, BorshSerialize)]
 pub struct ResolutionWindow {
     pub round: u8, // round 0 = resolution round | round > 0 = dispute round
     pub participants_to_outcome_to_stake: UnorderedMap<AccountId, UnorderedMap<u8, u128>>, // Maps participant account_id => outcome => stake_in_outcome
     pub required_bond_size: u128, // Total bond_size required to move on to next round of escalation
     pub staked_per_outcome: UnorderedMap<u8, u128>, // Staked per outcome
     pub end_time: u64, // Unix timestamp in ms representing when Dispute round is over
     pub outcome: Option<u8>, // Bonded outcome of this window
 }

 impl ResolutionWindow {
     pub fn new(prev_round: Option<u8>, market_id: u64, resolution_bond_base: u128) -> Self {
         let round = prev_round.unwrap_or(0);
         Self {
            round,
            participants_to_outcome_to_stake: UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}", market_id, round).as_bytes().to_vec()),
			required_bond_size: resolution_bond_base * 2_u128.pow(round.into()),
			staked_per_outcome: UnorderedMap::new(format!("market:{}:staked_per_outcome:{}", market_id, round).as_bytes().to_vec()), // Staked per outcome
			end_time: env::block_timestamp() + constants::TWELVE_HOURS,
			outcome: None,
         }
     }
 }
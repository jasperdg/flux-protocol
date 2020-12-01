use std::cmp;
use near_sdk::{
	env,
	AccountId,
	collections::{
		UnorderedMap,
		Vector,
		LookupSet
	},
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};

/*** Import fees implementation ***/
pub mod fees;
pub use fees::Fees;

/*** Import resolution_window implementation ***/
pub mod resolution_window;
pub use resolution_window::ResolutionWindow;

/*** Import validity_escrow implementation ***/
pub mod validity_escrow;
pub use validity_escrow::ValidityEscrow;

/*** Import orderbook implementation ***/
use crate::orderbook::Orderbook;

/*** Import logger methods ***/
use crate::logger;
/*** Import utils methods ***/
use crate::utils;
/*** Import constants ***/
use crate::constants;

/** 
 * @notice Market state struct
 */
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
	pub id: u64,
	pub creator: AccountId,
	pub outcomes: u8,
	pub creation_time: u64,
	pub end_time: u64,
	pub orderbooks: UnorderedMap<u8, Orderbook>,
	pub winning_outcome: Option<u8>, // If market is finalized and winning_outcome == None, market is deemed invalid
	pub resoluted: bool,
	pub resolution_bond: u128,
	pub filled_volume: u128,
	pub disputed: bool,
	pub finalized: bool,
	pub fees: Fees,
	pub validity_escrow: ValidityEscrow,
	pub resolution_windows: Vector<ResolutionWindow>,
	pub validity_bond_claimed: bool,
	pub claimed_earnings: LookupSet<AccountId>
}

impl Market {

	/**
	 * @notice Creates new Market instance
	 * @return Returns new Market instance
	 */
	pub fn new(
		id: u64, 
		account_id: AccountId,
		outcomes: u8, 
		end_time: u64,
		fees: Fees,
	) -> Self {

		/* Create an empty UnorderedMap with an unique storage pointer to store an orderbook for each outcome */
		let mut empty_orderbooks = UnorderedMap::new(format!("market:{}:orderbooks", id).as_bytes().to_vec());

		/* For each of the outcomes insert a new orderbook into the empty_orderbooks map */
		for i in 0..outcomes {
			empty_orderbooks.insert(&i, &Orderbook::new(id, i));
		}

		/* Create empty Vector object that will store all resolution windows */
		let mut resolution_windows = Vector::new(format!("market:{}:resolution_windows", id).as_bytes().to_vec());
		let resolution_bond = 5 * utils::one_token();
		resolution_windows.push(&ResolutionWindow::new(None, id, resolution_bond));

		let validity_escrow = ValidityEscrow {
			claimable_if_valid: UnorderedMap::new(format!("market:{}:claimable_if_valid", id).as_bytes().to_vec()),
			claimable_if_invalid: UnorderedMap::new(format!("market:{}:feeable_if_invalid", id).as_bytes().to_vec()),
		};

		/* Return market instance */
		Self {
			id,
			creator: account_id,
			outcomes,
			creation_time: utils::ns_to_ms(env::block_timestamp()),
			end_time,
			orderbooks: empty_orderbooks,
			winning_outcome: None,
			resoluted: false,
			resolution_bond: resolution_bond,
			filled_volume: 0,
			disputed: false,
			finalized: false,
			fees,
			validity_escrow,
			resolution_windows,
			validity_bond_claimed: false,
			claimed_earnings: LookupSet::new(format!("market:{}:claimed_earnings_for", id).as_bytes().to_vec()),
		}
	}

	/*** Trading methods ***/
	pub fn place_order_internal(
		&mut self, 
		account_id: &AccountId, 
		outcome: u8, 
		shares: u128, 
		spend: u128, 
		price: u16,
		affiliate_account_id: Option<AccountId>
	) {
		/* Try to fill matching orders, returns how much was eventually spent and how many shares were bought */
		let (spent_on_shares, shares_filled) = self.fill_matches(outcome, spend, price);

		/* Add the amount volume that was filled by this order to the filled_volume */
		self.filled_volume += shares_filled * 100;

		/* Retrieve the orderbook for this orders' outcome */
		let mut orderbook = self.orderbooks.get(&outcome).unwrap();

		/* Create and place a new order for the orderbook */
		orderbook.new_order(
			self.id,
			&account_id,
			outcome,
			spend,
			shares,
			price,
			spent_on_shares,
			shares_filled,
			affiliate_account_id,
		);

		/* Re-insert the mutated orderbook */
		self.orderbooks.insert(&outcome, &orderbook);
	}

	/** 
	 * @notice Tries to fill matching orders 
	 * @return A tuple where the first value is the amount spent while filling the matches and the second value is the amount of shares purchased for the money spent
	 * */ 
	fn fill_matches(
		&mut self, 
		outcome: u8,
		to_spend: u128, 
		price: u16
	) -> (u128, u128) {
		/* Gets the current market price and depth at that current price */
		let (mut market_price, mut share_depth) = self.get_market_price_and_min_liquidity(outcome);

		if market_price > price { return (0, 0) }

		/* Stores the amount of shares filled */
		let mut shares_filled = 0;
		/* Stores how much was spent on these shares */
		let mut spent = 0;
		/* Stores how much is left to spend */
		let mut left_to_spend = to_spend;
		
		/* If spendable <= 100 we can get overflows due to rounding errors */
		while left_to_spend > 100 && market_price <= price {
			/* Calc the amount of shares to fill at the current price which is the min between the amount left_to_spend / price and depth */
			let shares_to_fill = cmp::min(left_to_spend / u128::from(market_price), share_depth.expect("expected there to be share depth"));

			/* Loop through all other orderbooks and fill the shares to fill */
			for orderbook_id in  0..self.outcomes {
				if orderbook_id == outcome {continue;}

				let mut orderbook = self.orderbooks.get(&orderbook_id).expect("orderbook doesn't exist where it should");
				/* Fill best orders up to the shares to fill */
				orderbook.fill_best_orders(shares_to_fill);
				/* Re-insert the mutated orderbook instance */
				self.orderbooks.insert(&orderbook_id, &orderbook); 
			}
			
			/* Update tracking variables */
			left_to_spend -= shares_to_fill * u128::from(market_price);
			shares_filled += shares_to_fill;
			spent += shares_to_fill * u128::from(market_price);

			let (updated_market_price, updated_share_depth) = self.get_market_price_and_min_liquidity(outcome);
			market_price = updated_market_price;
			share_depth = updated_share_depth;
		}

		(spent, shares_filled)
	}

	/**
	 * @notice Calculates the market price for a certain outcome
	 * @dev market_price = 100 - best_price_for_each_other_outcome
	 * @return A u128 number representing the market price of the provided outcome
	 */
	pub fn get_market_price(
		&self, 
		outcome: u8
	) -> u16 {
		let mut market_price = 100;
 		for (orderbook_id, orderbook) in self.orderbooks.iter() {
			if orderbook_id == outcome {continue};
			let best_price = orderbook.price_data.max().unwrap_or(0);
			market_price -= best_price;
		}
		market_price
	}

	/**
	 * @notice Calculates the market price and returns depth at this market price
	 * @dev market_price = 100 - best_price_for_each_other_outcome
	 *  depth = min liquidity available at the opposing outcomes' best price
	 * @return the market price and returns depth at this market price
	 */
	pub fn get_market_price_and_min_liquidity(
		&self, 
		outcome: u8
	) -> (u16, Option<u128>) {
		let mut market_price = 100;
		let mut min_liquidity = None;

 		for (orderbook_id, orderbook) in self.orderbooks.iter() {
			if orderbook_id == outcome {continue};

			let best_price = orderbook.price_data.max().unwrap_or(0);
			if best_price == 0 {continue;}
			let liq_at_price = orderbook.price_data
				.get(&best_price)
				.expect("there should be an entry at best price but there isn't")
				.share_liquidity;

			if min_liquidity.is_none() || min_liquidity.unwrap() > liq_at_price {
				min_liquidity = Some(liq_at_price);
			}

			market_price -= best_price;
		}
		(market_price, min_liquidity)
	}

	/**
	 * @notice Sell a certain amount of shares into the current orderbook with a min_price to prevent slippage.
	 *  For sales there are some mechanics that are unique to Flux Protocol, users can sell any shares they own but 
	 *  will only receive tokens up to the amount the user paid no average per share. The delta will be added to claimable_if_valid
	 *  and this will be rewarded to the user if it turns out the market was in fact valid. If the user sells the shares for less
	 *  than what they initially paid for the share the delta will be added to claimable_if_invalid and they will be able to claim
	 *  this delta if it turns out the market is invalid.
	 * @return Returns the amount that needs to be transferred to the user
	 */
	pub fn dynamic_market_sell_internal(
		&mut self,
		sender: AccountId,
		outcome: u8,
		shares_to_sell: u128,
		min_price: u16,
	) -> u128 {
		let mut orderbook = self.orderbooks.get(&outcome).unwrap_or_else(|| { panic!("outcome: {} doesn't exist for this market", outcome) });

		/* Get the account balance if there is none return 0 */
		let shares_balance = match orderbook.account_data.get(&sender) {
			Some(data) => data.balance,
			None => return 0
		};
		
		assert!(shares_balance >= shares_to_sell, "user doesn't own this many shares");
		
		/* Get the amount of shares that we can sell and the average sell price */
		let (sell_depth, avg_sell_price) = orderbook.get_depth_down_to_price(shares_to_sell, min_price);
		
		/* Fill the best orders up to the amount of shares that are sellable */
		let shares_filled = orderbook.fill_best_orders(sell_depth);

		self.filled_volume += avg_sell_price * shares_filled;
		
		let mut account_data = orderbook.account_data.get(&sender).expect("something went wrong while trying to retrieve the user's account data");
		let avg_buy_price = account_data.calc_avg_buy_price();
		self.validity_escrow.update_escrow(&sender, sell_depth, avg_sell_price, avg_buy_price);
		account_data.update_balances(shares_filled);
		
		logger::log_update_user_balance(&sender, self.id, outcome, account_data.balance, account_data.to_spend, account_data.spent);
		
		/* Re-insert the updated user data  */
		orderbook.account_data.insert(&sender, &account_data);
		
		/* Re-insert the orderbook */
		self.orderbooks.insert(&outcome, &orderbook);
		
		shares_filled * cmp::min(avg_buy_price, avg_sell_price)
	}

	/*** Resolution methods ***/

	/**
	 * @notice The resolute method is used to stake on certain outcomes once a market has ended
	 * @return Returns how many if any of the sender's stake needs to be returned
	 */
	pub fn resolute_internal(
		&mut self,
		sender: &AccountId,
		winning_outcome: Option<u8>, 
		stake: u128
	) -> u128 {
		/* Convert option<u64> to a number where None (invalid) = self.outcomes */
		let outcome_id = self.to_numerical_outcome(winning_outcome);

		/* Get the most recent resolution window */
		let mut resolution_window = self.resolution_windows.get(self.resolution_windows.len() - 1).expect("Something went wrong during market creation");
		let mut to_return = 0;

		/* Get how much is currently is staked on the target outcome */
		let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).unwrap_or(0);

		/* Check if the total stake on this outcome >= resolution bond if so the stake will be bonded */
		if stake + staked_on_outcome >= self.resolution_bond {
			/* Calculate if anything needs to be returned to the staker */
			to_return = stake + staked_on_outcome - self.resolution_bond;
			/* Set winning_outcome - this is not final there could be a dispute */
			self.winning_outcome = winning_outcome;
			self.resoluted = true;
		} 

		/* Update sender's stake state */
		let mut sender_stake_per_outcome = resolution_window.participants_to_outcome_to_stake
		.get(&sender)
		.unwrap_or_else(||{
			UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}:{}", self.id, resolution_window.round, sender).as_bytes().to_vec())
		});
		
		let stake_in_outcome = sender_stake_per_outcome
		.get(&outcome_id)
		.unwrap_or(0);
		let new_stake = stake_in_outcome + stake - to_return;
		sender_stake_per_outcome.insert(&outcome_id, &new_stake);
		resolution_window.participants_to_outcome_to_stake.insert(&sender, &sender_stake_per_outcome);

		/* Update resolution_window's stake state */
		resolution_window.staked_per_outcome.insert(&outcome_id, &(staked_on_outcome + stake - to_return));
		
		/* If the market is now resoluted open dispute window */
		if self.resoluted {
			resolution_window.outcome = winning_outcome;

			let new_resolution_window = ResolutionWindow::new(Some(resolution_window.round), self.id, self.resolution_bond);

			logger::log_market_resoluted(self.id, &sender, resolution_window.round, stake - to_return, outcome_id);
			logger::log_new_resolution_window(self.id, new_resolution_window.round, new_resolution_window.required_bond_size, new_resolution_window.end_time);
			self.resolution_windows.push(&new_resolution_window);
			
		}  else {
			logger::log_staked_on_resolution(self.id, &sender, resolution_window.round, stake - to_return, outcome_id);

		}
		
		/* Re-insert the resolution window after update */
		self.resolution_windows.replace(resolution_window.round.into(), &resolution_window);

		to_return
	}

	/**
	 * @notice The dispute method is to correct incorrect resolutions posted by the initial resolutor(s)
	 * @return Returns how many if any of the sender's stake needs to be returned
	 */
	pub fn dispute_internal(
		&mut self, 
		sender: &AccountId,
		winning_outcome: Option<u8>,
		stake: u128
	) -> u128 {
		/* Convert option<u64> to a number where None (invalid) = self.outcomes */
		let outcome_id = self.to_numerical_outcome(winning_outcome);
		
		/* Get the most recent resolution window */
		let mut resolution_window = self.resolution_windows.get(self.resolution_windows.len() - 1).expect("Something went wrong during market creation");
		let mut to_return = 0;
		let full_bond_size = resolution_window.required_bond_size;
		let mut bond_filled = false;
		let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).unwrap_or(0);

		/* Check if this stake adds up to an amount >= the bond_size if so dispute will be bonded */
		if staked_on_outcome + stake >= full_bond_size  {
			bond_filled = true;
			to_return = staked_on_outcome + stake - full_bond_size;
			self.disputed = true;
			/* Set winning_outcome to current outcome - will be finalized by Judge */
			self.winning_outcome = winning_outcome;
		}

		/* Add stake to user's stake state */
		let mut sender_stake_per_outcome = resolution_window.participants_to_outcome_to_stake
		.get(&sender)
		.unwrap_or_else(|| {
			UnorderedMap::new(format!("market:{}:participants_to_outcome_to_stake:{}:{}", self.id, resolution_window.round, sender).as_bytes().to_vec())
		});
		let stake_in_outcome = sender_stake_per_outcome
		.get(&outcome_id)
		.unwrap_or(0);
		let new_stake = stake_in_outcome + stake - to_return;
		sender_stake_per_outcome.insert(&outcome_id, &new_stake);
		resolution_window.participants_to_outcome_to_stake.insert(&sender, &sender_stake_per_outcome);

		/* Add stake to the window's stake state */
		resolution_window.staked_per_outcome.insert(&outcome_id, &(staked_on_outcome + stake - to_return));

		
		// Check if this order fills the bond - if so open a new resolution window
		if bond_filled {
			// Set last winning outcome
			resolution_window.outcome = winning_outcome;

			let updated_staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).expect("This can't be None");
			assert_eq!(updated_staked_on_outcome, full_bond_size, "the total staked on outcome needs to equal full bond size if we get here");

			let bond_base = if resolution_window.round == utils::max_rounds() { 0 } else { self.resolution_bond };
			let next_resolution_window = ResolutionWindow::new(Some(resolution_window.round), self.id, bond_base);
			logger::log_resolution_disputed(self.id, &sender, resolution_window.round, stake - to_return, outcome_id);
			logger::log_new_resolution_window(self.id, next_resolution_window.round, next_resolution_window.required_bond_size, next_resolution_window.end_time);

			self.resolution_windows.push(&next_resolution_window);
		} else {
			logger::log_staked_on_dispute(self.id, &sender, resolution_window.round, stake - to_return, outcome_id);
		}

		// Re-insert the resolution window
		self.resolution_windows.replace(resolution_window.round.into(), &resolution_window);

		to_return
	}

	/**
	 * @notice Finalize the market outcome, after which earnings can be claimed by all participants
	 */
	pub fn finalize_internal(
		&mut self, 
		winning_outcome: Option<u8>
	) {
		// If the market was disputed the sender of this tx will be the judge and the judge will provide the final verdict being the definite outcome
	    if self.disputed {
            self.winning_outcome = winning_outcome;
		}

		logger::log_finalized_market(self.id, self.to_numerical_outcome(self.winning_outcome));
		
	    self.finalized = true;
	}

	/*** After finalization ***/

	/**
	 * @notice Calculates the amount a participant can claim in the market
	 * @return returns a tuple containing: amount claimable through trading, amount still left in open orders, amount claimable through resolution participation
	 */
	pub fn get_claimable_internal(
		&self, 
		account_id: &AccountId
	) -> (u128, u128, u128) {
		let invalid = self.winning_outcome.is_none();
		let mut winnings = 0;
		let mut in_open_orders = 0;

		if invalid {
			/* Loop through all orderbooks */
			for (_, orderbook) in self.orderbooks.iter() {
				/* Check if the user has any participation in this outcome else continue to next outcome */
				let account_data = match orderbook.account_data.get(account_id) {
					Some(user) => user,
					None => continue
				};
								
				/* Calculate and add money in open orders */
				in_open_orders += account_data.to_spend - account_data.spent;
				/* Treat filled volume as winnings */
				winnings += account_data.spent;
			}
		} else {
			/* Loop through all orderbooks */
			for (_, orderbook) in self.orderbooks.iter() {
				/* Check if the user has any participation in this outcome else continue to next outcome */
				let account_data = match orderbook.account_data.get(account_id) {
					Some(user) => user,
					None => continue
				};
				/* Calculate and increment in_open_orders with open orders for each outcome */
				in_open_orders += account_data.to_spend - account_data.spent;
			}

			/* Get the orderbook of the winning outcome */
			let winning_orderbook = self.orderbooks.get(&self.to_numerical_outcome(self.winning_outcome)).unwrap();

			/* Check if the user traded in the winning_outcome */
			let winning_value = match winning_orderbook.account_data.get(account_id) {
				Some(user) => user.balance * 100, // Calculate user winnings: shares_owned * 100
				None => 0
			};

			/* Set winnings to the amount of participation */
			winnings = winning_value;
		}

		/* Calculate governance earnings */ 
		let governance_earnings = self.get_dispute_earnings(account_id);

		(winnings, in_open_orders, governance_earnings)
	}

	/**
	 * @notice Allows users to withdraw the stake they have in a resolution round as long as the amount is not bonded
	 * @dev Panics if the sender tries to withdraw stake in the bonded outcome
	 *  Panics if the user hasn't participated in the market
	 *  Panics if the user has already withdrawn they stake before
	 * @return Returns amount to transfer to user
	 */
	pub fn withdraw_resolution_stake_internal(
		&mut self,
		sender: AccountId,
		round: u8,
		outcome: Option<u8>
	) -> u128{
		/* Convert option<u64> to a number where None (invalid) = self.outcomes */
		let outcome_id = self.to_numerical_outcome(outcome);

		/* Get the target resolution window a user wants to withdraw their stake from */
		let mut resolution_window = self.resolution_windows.get(round.into()).expect("dispute round doesn't exist");
		assert_ne!(outcome, resolution_window.outcome, "you cant cancel dispute stake for bonded outcome");
		let mut sender_participation = resolution_window.participants_to_outcome_to_stake.get(&sender).expect("user didn't participate in this dispute round");
		let to_return = sender_participation.get(&outcome_id).expect("sender didn't participate in this outcome resolution");
		assert!(to_return > 0, "Can't withdraw 0");

		/* Set senders stake to 0 and re-insert to resolution window */
		sender_participation.insert(&outcome_id, &0);
		resolution_window.participants_to_outcome_to_stake.insert(&sender, &sender_participation);

		let staked_on_outcome = resolution_window.staked_per_outcome.get(&outcome_id).expect("Unexpected error during withdraw resolution");
		/* Decrement total stake by to_return */
		resolution_window.staked_per_outcome.insert(&outcome_id, &(staked_on_outcome - to_return));
		
		/* Re-insert updated resolution window */
		self.resolution_windows.replace(resolution_window.round.into(), &resolution_window);

		to_return
	}

	/** 
	 * @notice Calculate the resolution/dispute earnings for a account_id
	 * @return Returns total earnings from participating in resolution/dispute
	 */
	fn get_dispute_earnings(
		&self, 
		account_id: &AccountId
	) -> u128 {
		let mut user_correctly_staked = 0;
		let mut resolution_reward = 0;
		let mut total_correctly_staked = 0;
		let mut total_incorrectly_staked = 0;

		let winning_outcome_id = self.to_numerical_outcome(self.winning_outcome);
		
		/* Loop through all resolution_windows */
		for window in self.resolution_windows.iter() {
			/* check if round = 0 - which is the resolution round */
			if window.round == 0 {

				/* Calculate how much the total fee payout will be */
				let total_resolution_fee = utils::calc_fee(self.filled_volume, self.fees.resolution_fee_percentage);
		
				/* Check if the outcome that a resolution bond was staked on corresponds with the finalized outcome */
				if self.winning_outcome == window.outcome {
					/* check if the user participated in this outcome */
					let resolution_participation = window.participants_to_outcome_to_stake.get(&account_id);
					
					if resolution_participation.is_some() {
						/* Check how much of the bond the user participated */
						let correct_outcome_participation = match resolution_participation {
							Some(participation) => participation.get(&self.to_numerical_outcome(self.winning_outcome)).unwrap_or(0),
							None => 0
						};

						if correct_outcome_participation > 0 {
							/* If a user participated < 1 / precision of the total stake in an outcome their resolution_fee distribution will be rounded down to 0 */
							let relative_participation = correct_outcome_participation * constants::EARNINGS_PRECISION / window.required_bond_size;
							let user_fee_reward = relative_participation * total_resolution_fee / constants::EARNINGS_PRECISION;
							/* calculate his relative share of the total_resolution_fee relative to his participation */
							resolution_reward += user_fee_reward + correct_outcome_participation;
						}
						
					} 
				} else {
					/* If the initial resolution bond wasn't staked on the correct outcome, divide the resolution fee amongst disputors */
					total_incorrectly_staked += total_resolution_fee + window.required_bond_size;
				}
			} else {
				/* If it isn't the first round calculate according to escalation game */
				let window_outcome_id = self.to_numerical_outcome(window.outcome);

				if window_outcome_id == winning_outcome_id {
					let round_participation = window.participants_to_outcome_to_stake
					.get(&account_id)
					.unwrap_or_else(|| {
						UnorderedMap::new(format!("market:{}:staked_per_outcome:{}:{}", self.id, window.round, account_id).as_bytes().to_vec())
					})
					.get(&winning_outcome_id)
					.unwrap_or(0);

					user_correctly_staked += round_participation;
					total_correctly_staked += window.required_bond_size;
				} else if window.outcome.is_some() {
					total_incorrectly_staked += window.required_bond_size;
				 
				}
			}
		}

		if total_correctly_staked == 0 || total_incorrectly_staked == 0 || user_correctly_staked == 0 {return resolution_reward}


		/* Declare decimals to ensure that people with up until 1/`constants::EARNINGS_PRECISION`th of total stake are rewarded */
		/* Calculate profit from participating in disputes */
		let profit = ((total_incorrectly_staked * constants::EARNINGS_PRECISION) / (total_correctly_staked / user_correctly_staked)) / constants::EARNINGS_PRECISION; 

		profit + user_correctly_staked + resolution_reward
	}

	/**
	 * @notice Convert winning_outcome (Option<u64>) -> u64 where None = self.outcomes
	 */
	pub fn to_numerical_outcome(
		&self, 
		outcome: Option<u8>, 
	) -> u8 {
		outcome.unwrap_or(self.outcomes)
	}
}

/**
 * @notice Makes sure market is initialized by the new method
 * @dev Panics if market isn't initialized with the new method
 */
impl Default for Market {
	fn default() -> Self {
		panic!("No default state available init with ::new"); 
	}
}
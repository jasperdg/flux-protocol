use near_sdk::{
	near_bindgen, 
	env, 
	ext_contract, 
 	Promise, 
	PromiseOrValue, 
	json_types::{U128, U64},
	PromiseResult,
	collections::{
		UnorderedMap,
	},
	borsh::{
		self, 
		BorshDeserialize, 
		BorshSerialize
	}
};

/** 
 * @title Flux Protocol
 */

/*** Import market implementation ***/
use crate::market;
/*** Import logger methods ***/
use crate::logger;

/*** Create market type ***/
type Market = market::Market;

/**
 * @notice The state struct for the Flux Protocol implementation 
 */
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct FluxProtocol {
	owner: String,
	markets: UnorderedMap<u64, Market>,
	nonce: u64,
	max_fee_percentage: u128,
	creation_bond: u128,
	affiliate_earnings: UnorderedMap<String, u128>,
	fun_token_account_id: String,
}

/**
 * @notice A hardcoded amount of gas that's used for external transactions
 * @dev Currently set to MAX_GAS / 3
 * TODO: Add affiliate payouts 
 */
const SINGLE_CALL_GAS: u64 = 100000000000000;

/*** External Contract Interfaces ***/
/** @dev To interact with an external contract there needs to be an interface defined in the form of a trait */

/**
 * @notice Contract interface for the Fungible Token contract we're using:
 * @dev based on older version of: https://github.com/near/near-sdk-rs/tree/master/examples/fungible-token
 * TODO: Update FunToken contract
 */
#[ext_contract]
pub trait FunToken {
    fn transfer_from(&mut self, owner_id: String, new_owner_id: String, amount: U128);
    fn transfer(&mut self, new_owner_id: String, amount: U128);
    fn get_total_supply(&self) -> u128;
    fn get_balance(&self, owner_id: AccountId) -> u128;
}

/**
 * @notice Contract interface for the Flux Protocol contract itself: 
 * @dev: We need to define this interface to be able to call Flux Protocol methods in a promise chain, which is required in NEAR promise API
 *  for more info checkout the Promise api: https://github.com/near/near-sdk-rs/blob/master/near-sdk/src/promise.rs
 */
#[ext_contract]
pub trait FluxProtocol {
    fn market_creation(&mut self, sender: String, market_id: u64, outcome: u64, amount_of_shares: u128, spend: u128, price: u128, affiliate_account_id: Option<String>);
    fn proceed_order_placement(&mut self, sender: String, market_id: u64, outcome: u64, shares: u128, spend: u128, price: u128, affiliate_account_id: Option<String>);
    fn proceed_market_resolution(&mut self, sender: String, market_id: u64, winning_outcome: Option<u64>, stake: u128);
	fn proceed_market_dispute(&mut self, sender: String, market_id: u64, winning_outcome: Option<u64>, stake: u128);
	fn proceed_market_creation(&mut self, sender: String, description: String, extra_info: String, outcomes: u64, outcome_tags: Vec<String>, categories: Vec<String>, end_time: u64, creator_fee_percentage: u128, resolution_fee_percentage: u128, affiliate_fee_percentage: u128, api_source: String);
}


/**
 * @dev Flux Protocol contract is unusable until it is initialized and should be initialized in the same transaction as it's deployment
 8  checkout the near-cli deploy method: https://github.com/near/near-cli
 */
impl Default for FluxProtocol {
    fn default() -> Self {
        panic!("Flux protocol should be initialized before usage")
    }
}

/**
 * @notice Flux Protocol implementation
 */
#[near_bindgen]
impl FluxProtocol {

	/**
	 * @notice Initialize the Flux Protocol contract
	 * @dev should be treated as constructor and fired during deployment, the contract is unusable before the init method succeeded
	 *  panics if the contract is already initialized
	 * @param owner Account id of the contract owner, the owner is for now set to solve disputes
	 * @param fun_token_account_id The account id of the token used for trading
	 */
	#[init]
	pub fn init(
		owner: String, 
		fun_token_account_id: String
	) -> Self {
		assert!(!env::state_exists(), "Already initialized");
		Self {
			owner,
			markets: UnorderedMap::new(b"markets".to_vec()),
			nonce: 0,
			max_fee_percentage: 500,
			creation_bond: 25e18 as u128 / 100,
			affiliate_earnings: UnorderedMap::new(b"affiliate_earnings".to_vec()),
			fun_token_account_id
		}
	}

	/*** Getters ***/
	/**
	 * @notice Returns the owner's account id
	 * @return owner's account id
	 */
	pub fn owner(
		&self
	) -> String {
		return self.owner.to_string();
	}

	/**
	 * @notice Returns the fungible token's account id
	 * @return Fungible token's account id
	 */
	fn fun_token_account_id(
		&self
	) -> String {
		return self.fun_token_account_id.to_string();
	}

	/**
	 * @dev Checks if the method called is the contract itself
	 *  panics if predecessor_account (sender) isn't the FluxProtcol account id
	 */
	fn assert_self(
		&self
	) {
		assert_eq!(env::current_account_id(), env::predecessor_account_id(), "this method can only be called by the contract itself"); 
	}

	/**
	 * @dev Checks if the previous promise in the promise chain passed successfully
	 *  panics if the previous promise in the promise chain was unsuccessful
	 */
	fn assert_prev_promise_successful(
		&self
	) {
		assert_eq!(self.is_promise_success(), true, "previous promise failed");
	}

	/**
	 * @notice returns market volume
	 * @dev only needed for unit tests
	 */
	 pub fn get_market_volume(
		&self,
		market_id: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		return self.markets
		.get(&market_id)
		.expect("market doesn't exist")
		.filled_volume.into();
	}

	/**
	 * @notice returns market price
	 * @dev only needed for unit tests
	 */
	pub fn get_market_price(
		&self,
		market_id: U64,
		outcome: U64
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		return U128(self.markets
			.get(&market_id)
			.expect("market doesn't exist")
			.get_market_price(outcome));
	}

	/**
	 * @notice returns an account their balance in a certain market for a certain outcome
	 * @dev only needed for unit tests
	 */
	pub fn get_outcome_share_balance(
		&self,
		account_id: String,
		market_id: U64,
		outcome: U64,
	) -> U128 {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();

		let market = self.markets.get(&market_id).expect("non existent market");
		let orderbook = market.orderbooks.get(&outcome).expect("non existent outcome");
		let user_data = orderbook.user_data.get(&account_id);

		if user_data.is_none() {return U128(0)}

		return U128(user_data.unwrap().balance);
	}

	/**
	 * @notice Withdraw your stake on a specific outcome in a resolution or dispute
	 * @dev Panics if sender don't have any stake in the market / round / outcome
	 *  Panics if the market doesn't exist
	 *	Only works as long as the total stake < the stake required for that round, afterwards the stake will be bonded and not withdrawable until market finalization
	 * @param market A reference to the market where from to return the creator fee
	 * @return Returns the fee percentage to be paid out to the market creator, if the market turns out to be invalid this will be 0
	 */
	 fn get_creator_fee_percentage(&self, market: &Market) -> u128 {
		return match market.winning_outcome {
			Some(_) => market.creator_fee_percentage,
			None => 0
		}
	}

	/**
	 * @notice Calculates and returns the amount a user can claim in a market if the current resolution data is correct
	 * @param market A reference to the market where from to return the creator fee
	 * @return Returns the amount of base tokens claimable denominated in 1e18
	 * TODO: Make sure get_claimable is only callable for finalized markets
	 */
	pub fn get_claimable(
		&self, 
		market_id: U64, 
		account_id: String
	) -> U128 {
		
		let market_id: u64 = market_id.into();
		let market = self.markets.get(&market_id).expect("market doesn't exist");
		let claimed_earnings = market.claimed_earnings.get(&account_id);
		if claimed_earnings.is_some() {
			return U128(0);
		}

		let mut validity_bond = 0;
		if account_id == market.creator && market.validity_bond_claimed == false && market.winning_outcome != None {
			validity_bond = self.creation_bond;
		}

		let (winnings, left_in_open_orders, governance_earnings) = market.get_claimable_internal(account_id.to_string());
		
		let claimable_if_invalid = match market.winning_outcome {
			None =>  market.claimable_if_invalid.get(&account_id).unwrap_or(0),
			_ => 0
		};

		let claimable_if_valid = match market.winning_outcome {
			Some(_) =>  market.claimable_if_valid.get(&account_id).unwrap_or(0),
			_ => 0
		};

		let total_feeable_amount = winnings + claimable_if_invalid;
		let total_fee_percentage =  market.resolution_fee_percentage + self.get_creator_fee_percentage(&market);
		let total_fee = (total_feeable_amount * total_fee_percentage + 10000 - 1) / 10000;
		

		let to_claim = total_feeable_amount + governance_earnings + left_in_open_orders + validity_bond + claimable_if_valid - total_fee;

		return (to_claim).into();
	}

	/**
	 * @dev Panics if the previous promise in the promise chain was unsuccessful
	 * @return Returns a bool representing the success of the previous promise in a promise chain
	 */
	fn is_promise_success(&self) -> bool {
		assert_eq!(
			env::promise_results_count(),
			1,
			"Contract expected a result on the callback"
		);
		match env::promise_result(0) {
			PromiseResult::Successful(_) => true,
			_ => false,
		}
	}

	/*** Setters ***/

	/**
	 * @notice Allows the protocol owner to change the fungible token used in the protocol
	 * @dev Panics if predecssor_account_id isn't the protocol owner account id
	 * @param fun_token_account_id the account id of the fungible token that should be used from there on
	 * TODO: Make sure that each market has an attribute referencing what token was used as the base token for that market. After set_fun_token was called each market created should use the new fun_token_account_id as the base currency
	 */
	pub fn set_fun_token (
		&mut self, 
		fun_token_account_id: String
	) {
		assert_eq!(env::predecessor_account_id(), self.owner);
		self.fun_token_account_id = fun_token_account_id;
	}
	
	/**
	 * @notice Kicks off market creation returns a promise that exists of a promise chain
	 * @dev Panics if market parameters are invalid
	 *  if outcomes == 2 we assume that it's a binary market and expect outcome_tags to be empty because assume it's ["NO", "YES"]
	 * @param description A description of the market
	 * @param extra_info Extra info about the market, these could be specific details like what source should be used to resolve the market etc
	 * @param outcomes The number out outcomes a market has, min is 2 max is 8
	 * @param outcome_tags A list of strings where the outcome id corresponds to the index of the outcome_tags array e.g. outcome 0 = outcome_tags[0]
	 * @param categories A list of categories that describe the market (helps with filtering)
	 * @param end_time Unix timestamp in miliseconds of when the market stops being tradeable and can be resoluted
	 * @param creator_fee_percentage Percentage with two decimals so denominated in 1e4 between 0 - 500 where 1 = 0.01% and 100 = 1%
	 * @param affiliate_fee_percentage Percentage of the creator fee that should go to affiliate accounts range betwen 1 - 100
	 * @param api_source For when we have validators running, these validators then use this attribute to automatically resolute / dispute the market
	 * @return returns a promise chain - this chain tries to escrow the base currency as a validity bond from the market creation and if successful proceed the market creation
	 * TODO: Should consider not storing categories but just logging them, that way the indexer will pick them up but wills save gas cost
	 */
	pub fn create_market(
		&mut self, 
		description: String, 
		extra_info: String, 
		outcomes: U64,
		outcome_tags: Vec<String>,
		categories: Vec<String>,
		end_time: U64,
		creator_fee_percentage: U128,
		affiliate_fee_percentage: U128,
		api_source: String
	) -> Promise {
		let outcomes: u64 = outcomes.into();
		let end_time: u64 = end_time.into();
		let creator_fee_percentage: u128 = creator_fee_percentage.into();
		let affiliate_fee_percentage: u128 = affiliate_fee_percentage.into();

		for outcome_tag in &outcome_tags {
			assert!(outcome_tag.chars().count() < 20, "outcome tag can't be more than 20 chars");
		}

		for category in &categories {
			assert!(category.chars().count() < 20, "category tag can't be more than 20 chars");
		}

		assert!(description.chars().count() < 201, "description can't than 200 characters");
		assert!(extra_info.chars().count() < 401, "extra_info can't than 400 characters");
		assert!(outcomes > 1, "need to have more than 2 outcomes");
		assert!(outcomes == 2 || outcomes == outcome_tags.len() as u64, "invalid outcomes");
		assert!(outcomes < 8, "can't have more than 8 outcomes"); // up for change
		assert!(end_time > env::block_timestamp() / 1000000, "end_time has to be greater than NOW");
		assert!(categories.len() < 8, "can't have more than 8 categories");
		assert!(creator_fee_percentage <= self.max_fee_percentage, "creator_fee_percentage too high");
		assert!(affiliate_fee_percentage <= 100, "affiliate_fee_percentage can't be higher than 100");

		if outcomes == 2 {assert!(outcome_tags.len() == 0)}

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), self.creation_bond.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
			flux_protocol::proceed_market_creation(
				env::predecessor_account_id(), 
				description,
				extra_info,
				outcomes,
				outcome_tags,
				categories,
				end_time,
				creator_fee_percentage, 
				100,
				affiliate_fee_percentage,
				api_source,
				&env::current_account_id(),
				0,
				SINGLE_CALL_GAS
			)
		);
	}

	/**
	 * @notice Continues market creation
	 * @dev Panics if the previous promise (token transfer) failed
	 *  panics if predecessor account_id isn't the Flux Protocol contract itself
	 * @param sender The account_id that signed the create_market transaction
	 * @param description A description of the market
	 * @param extra_info Extra info about the market, these could be specific details like what source should be used to resolve the market etc
	 * @param outcomes The number out outcomes a market has, min is 2 max is 8
	 * @param outcome_tags A list of strings where the outcome id corresponds to the index of the outcome_tags array e.g. outcome 0 = outcome_tags[0]
	 * @param categories A list of categories that describe the market (helps with filtering)
	 * @param end_time Unix timestamp in miliseconds of when the market stops being tradeable and can be resoluted
	 * @param creator_fee_percentage Percentage with two decimals so denominated in 1e4 between 0 - 500 where 1 = 0.01% and 100 = 1%
	 * @param affiliate_fee_percentage Percentage of the creator fee that should go to affiliate accounts range betwen 1 - 100
	 * @param api_source For when we have validators running, these validators then use this attribute to automatically resolute / dispute the market
	 * @return Returns the newly_created market_id
	 * TODO: Should consider not storing categories but just logging them, that way the indexer will pick them up but wills save gas cost
	 */
	pub fn proceed_market_creation(
		&mut self, 
		sender: String, 
		description: String, 
		extra_info: String, 
		outcomes: u64, 
		outcome_tags: Vec<String>, 
		categories: Vec<String>, 
		end_time: u64, 
		creator_fee_percentage: u128, 
		resolution_fee_percentage: u128, 
		affiliate_fee_percentage: u128, 
		api_source: String
	) -> PromiseOrValue<u64> {
		self.assert_self();
		self.assert_prev_promise_successful();

		let new_market = Market::new(
			self.nonce, 
			sender, 
			description, 
			extra_info, 
			outcomes, 
			outcome_tags, 
			categories, 
			end_time, 
			creator_fee_percentage, 
			resolution_fee_percentage, 
			affiliate_fee_percentage,
			api_source
		)
		;
		logger::log_market_creation(&new_market);
		let resolution_window = new_market.resolution_windows.get(0).expect("something went wrong during market creation");
		logger::log_new_resolution_window(new_market.id, resolution_window.round, resolution_window.required_bond_size, resolution_window.end_time);

		let market_id = new_market.id;
		self.markets.insert(&self.nonce, &new_market);
		self.nonce = self.nonce + 1;

		
		return PromiseOrValue::Value(market_id);
	}

	/** 
	 * @notice Kicks off order placement
	 * @dev Panics if the order parameters are invalid
	 * @param market_id The id of the market
	 * @param outcome The specific outcome this order wants to buy
	 * @param shares The amount of shares a user wants to buy denominated in 1e16
	 * @param price The price the user is willing to pay for this outcome, ranged 1 - 99
	 * @param affiliate_account_id The account id of the affiliate that sent the user to the platform
	 * @return Returns a promise chain that will first transfer the funds into escrow on this contract and then will proceed to place the order
	 */
	pub fn place_order(
		&mut self, 
		market_id: U64, 
		outcome: U64,
		shares: U128,
		price: U128,
		affiliate_account_id: Option<String>
	) -> Promise {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let price: u128 = price.into();
		let shares: u128 = shares.into();
		let rounded_spend = shares * price;
		let market = self.markets.get(&market_id).expect("market doesn't exist");

		assert!(rounded_spend >= 10000, "order must be valued at > 10000");
		assert!(price > 0 && price < 100, "price can only be between 0 - 100");
		assert!(outcome < market.outcomes, "invalid outcome");
		assert_eq!(market.resoluted, false, "market has already been resoluted");
		assert!(env::block_timestamp() / 1000000 < market.end_time, "market has already ended");

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), rounded_spend.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 10)
		.then(
			flux_protocol::proceed_order_placement( 
				env::predecessor_account_id(),
				market_id,
				outcome,
				shares,
				rounded_spend,
				price,
				affiliate_account_id,
				&env::current_account_id(), 
				0, 
				SINGLE_CALL_GAS * 2 - SINGLE_CALL_GAS / 10
			)
		);
	}

	/** 
	 * @notice Kicks off order placement
	 * @dev Panics if the signer isn't the contract itself
	 *  panics if the previous promise wasn't successful
	 * @param sender The signer of the original place_order transaction
	 * @param market_id The id of the market
	 * @param outcome The specific outcome this order wants to buy
	 * @param shares The amount of shares a user wants to buy denominated in 1e16
	 * @param spend The rounded (down) amount of base tokens to spend on this transaction 
	 * @param price The price the user is willing to pay for this outcome, ranged 1 - 99
	 * @param affiliate_account_id The account id of the affiliate that sent the user to the platform
	 * @return Returns a bool indicating that the tx was successful 
	 */
	pub fn proceed_order_placement(
		&mut self,
		sender: String,
		market_id: u64, 
		outcome: u64,
		shares: u128,
		spend: u128,
		price: u128,
		affiliate_account_id: Option<String>,
	) -> PromiseOrValue<bool> {
		self.assert_self();
		self.assert_prev_promise_successful();
		
		let mut market = self.markets.get(&market_id).unwrap();
		market.place_order_internal(sender, outcome, shares, spend, price, affiliate_account_id);
		self.markets.insert(&market.id, &market);
		return PromiseOrValue::Value(true);
	}

	/** 
	 * @notice Sells owned shares at market prices
	 * @dev Panics if the min_price provided is 0
	 *  panics if the min_price > 99
	 *  panics if shares < 1
	 *  panics if the market is already finalized
	 *  panics if there are no shares to sell owned by the sender for the min_price
	 * @param market_id The id of the market to sell shares
	 * @param outcome The specific outcome this order wants to sell shares
	 * @param shares The amount of shares a sender wants to sell
	 * @param min_price The min_price the sender is willing to sell his shares for
	 */
	pub fn dynamic_market_sell(
		&mut self,
		market_id: U64,
		outcome: U64,
		shares: U128,
		min_price: U128
	) {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let shares: u128 = shares.into();
		let min_price: u128 = min_price.into();
		
		assert!(min_price > 0, "min_price need to be higher than 0");
		assert!(min_price < 100, "min_price need to be smaller than 100");
		assert!(shares > 0, "can't sell 0 shares");
		
		let mut market = self.markets.get(&market_id).expect("non existent market");
		assert_eq!(market.finalized, false, "can't sell shares after market is finalized");
		let earnings = market.dynamic_market_sell_internal(outcome, shares, min_price);
		assert!(earnings > 0, "no matching orders");
		self.markets.insert(&market_id, &market);
		
		fun_token::transfer(env::predecessor_account_id(), U128(earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
	}

	/**
	 * @notice Cancels an order and returns outstanding open value to order creator
	 * @dev Panics if the predecessor_account isn't the owner of the order he's trying to cancel
	 *  Panics if market is already resoluted, open orders are included in the claimable amount 
	 * @param market_id The id of the market this order was placed on before
	 * @param outcome The outcome this order was for
	 * @param price The price this order was placed at, this is necessary because of the way orders are stored
	 * @param order_id The id of the order that's to be canceled
	 */
	pub fn cancel_order(
		&mut self, 
		market_id: U64, 
		outcome: U64,
		price: U128,
		order_id: U128
	) {
		let market_id: u64 = market_id.into();
		let outcome: u64 = outcome.into();
		let order_id: u128 = order_id.into();
		let price: u128 = price.into();
		
		let mut market = self.markets.get(&market_id).unwrap();
		assert_eq!(market.resoluted, false);
		let mut orderbook = market.orderbooks.get(&outcome).unwrap();
		let price_data = orderbook.price_data.get(&price).expect("order at this price doesn't exist");
		let order = price_data.orders.get(&order_id).expect("order with this id doesn't exist");
		assert!(env::predecessor_account_id() == order.creator, "not this user's order");

		let to_return = orderbook.cancel_order(order);
		market.orderbooks.insert(&outcome, &orderbook);
		self.markets.insert(&market_id, &market);
		fun_token::transfer(env::predecessor_account_id(), to_return.into(), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
    }

	/**
	 * @notice Kicks off market resolution, supply the outcome data to the 
	 * @dev Panics if the market hasn't ended yet
	 *  Panics if the market doens't exist
	 *  Panics if the market is already resoluted
	 *  Panics if the market is already finalized
	 *  Panics if the winning_outcome is invalid
	 *  Panics if the user doesn't have enough balance / allowance to transfer `stake`
	 * @param market_id The id of the market to resolute
	 * @param winning_outcome The winning_outcome according to the staker
	 * @param stake The amount of stake the user wants to contribute to the resolution round
	 */
	pub fn resolute_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<U64>,
		stake: U128
	) -> Promise {
		let market_id: u64 = market_id.into();
		let winning_outcome: Option<u64> = match winning_outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};
		let stake_u128: u128 = stake.into();
		let market = self.markets.get(&market_id).expect("market doesn't exist");
		assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, false, "market is already resoluted");
		assert_eq!(market.finalized, false, "market is already finalized");
		assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");

		return fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2)
		.then(
			flux_protocol::proceed_market_resolution(
				env::predecessor_account_id(),
				market_id,
				winning_outcome,
				stake_u128,
				&env::current_account_id(),
				0,
				SINGLE_CALL_GAS
			)
		);
	}

	/**
	 * @notice Proceeds the market resolution if the transfer was successful
	 * @dev Panics if the previous method (transfer) failed
	 *  Panics if the predecessor_id isn't equal to the contract id itself
	 * @param market_id The id of the market to resolute
	 * @param winning_outcome The winning_outcome according to the staker
	 * @param stake The amount of stake the user wants to contribute to the resolution round
	 * @param sender The account id of the original transaction's signer
	 */
	pub fn proceed_market_resolution(
		&mut self,
		market_id: u64,
		winning_outcome: Option<u64>,
		stake: u128,
		sender: String
	) -> PromiseOrValue<bool> {
		self.assert_self();
		self.assert_prev_promise_successful();
		
		let mut market = self.markets.get(&market_id).unwrap();
		let change: u128 = market.resolute_internal(sender.to_string(), winning_outcome, stake).into();
		self.markets.insert(&market_id, &market);
		if change > 0 {
			let prom = fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2);
			return PromiseOrValue::Promise(prom);
		} else {
			return PromiseOrValue::Value(true);
		}
	}

	/**
	 * @notice Kicks of a dispute of a certain outcome
	 * @dev Panics if the market hasn't been resoluted yet
	 *  Panics if the market doens't exist
	 *  Panics if the market is already finalized
	 *  Panics if the winning_outcome is invalid
	 *  Panics if the disputed outcomeis the same outcome as the previous winning outcome
	 *  Panics if the sender doesn't have enough balance / allowance to transfer `stake`
	 *  Panics if the dispute round is > 1. After one initial dispute the market has to be finalized by the owner ("judge")
	 * @param market_id The id of the market to dispute
	 * @param winning_outcome The winning_outcome according to the staker
	 * @param stake The amount of stake the sender wants to contribute to the dispute round
	 */
	pub fn dispute_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<U64>,
		stake: U128
	) -> Promise {
		let market_id: u64 = market_id.into();
		let winning_outcome: Option<u64> = match winning_outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};
		let stake_u128: u128 = stake.into();
        let market = self.markets.get(&market_id).expect("market doesn't exist");

		assert_eq!(market.resoluted, true, "market isn't resoluted yet");
		assert_eq!(market.finalized, false, "market is already finalized");
        assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");
        assert!(winning_outcome != market.winning_outcome, "same oucome as last resolution");
		let resolution_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("Invalid dispute window unwrap");
		assert_eq!(resolution_window.round, 1, "for this version, there's only 1 round of dispute");
		assert!(env::block_timestamp() / 1000000 < resolution_window.end_time, "dispute window is closed, market can be finalized");

		fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2).then(
			flux_protocol::proceed_market_dispute(
				env::predecessor_account_id(),
				market_id,
				winning_outcome,
				stake_u128,
				&env::current_account_id(), 
				0, 
				SINGLE_CALL_GAS
			)
		)
	}


	/**
	 * @notice Continues the dispute proces if transfer of funds was successful
	 * @dev Panics if the previous method (transfer) failed
	 *  Panics if the predecessor_id isn't equal to the contract id itself
	 * @param market_id The id of the market to dispute
	 * @param winning_outcome The winning_outcome according to the staker
	 * @param stake The amount of stake the sender wants to contribute to the dispute round
	 * @param sender The account id of the original transaction's signer
	 */
	pub fn proceed_market_dispute(		
		&mut self,
		market_id: u64,
		winning_outcome: Option<u64>,
		stake: u128,
		sender: String
	) -> PromiseOrValue<bool> {
		self.assert_self();
		self.assert_prev_promise_successful();

        let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		let change = market.dispute_internal(sender.to_string(), winning_outcome, stake);

		self.markets.insert(&market.id, &market);
		if change > 0 {
			return PromiseOrValue::Promise(fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS / 2));
		} else {
			return PromiseOrValue::Value(true);
		}
	}

	/**
	 * @notice Finalizes a market once disputed or the dispute window has been closed
	 * @dev Panics if the market hasn't been resoluted yet
	 *  Panics if the market is disputed and finalize is not called by the judge
	 *	Panics if the dispute window is still open
	 * @param market_id The id of the market to finalize
	 * @param winning_outcome Optional in case the market has been disptud, the judges ruling
	 */
	pub fn finalize_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<U64>
	) {
		let market_id: u64 = market_id.into();
		let winning_outcome: Option<u64> = match winning_outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};

		let mut market = self.markets.get(&market_id).unwrap();
		assert_eq!(market.resoluted, true, "market has to be resoluted before it can be finalized");
		if market.disputed {
			assert_eq!(env::predecessor_account_id(), self.owner, "only the judge can resolute disputed markets");
		} else {
			let dispute_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("no dispute window found, something went wrong");
			assert!(env::block_timestamp() / 1000000 >= dispute_window.end_time || dispute_window.round == 2, "dispute window still open")
		}

		market.finalize_internal(winning_outcome);
		self.markets.insert(&market_id, &market);
	}

	/**
	 * @notice Withdraw your stake on a specific outcome in a resolution or dispute
	 * @dev Panics if sender don't have any stake in the market / round / outcome
	 *  Panics if the market doesn't exist
	 *	Only works as long as the total stake < the stake required for that round, afterwards the stake will be bonded and not withdrawable until market finalization
	 * @param market_id The id of the market to withdraw the users stake from
	 * @param dispute_round The round of resolution of dispute the user wants to withdraw from
	 * @param outcome The outcome the user staked on
	 */
	pub fn withdraw_dispute_stake(
		&mut self, 
		market_id: U64,
		dispute_round: U64,
		outcome: Option<U64>
	) -> Promise {
		let market_id: u64 = market_id.into();
		let dispute_round: u64 = dispute_round.into();
		let outcome: Option<u64> = match outcome {
			Some(outcome) => Some(outcome.into()),
			None => None
		};

		let mut market = self.markets.get(&market_id).expect("invalid market");
		let to_return = market.withdraw_resolution_stake_internal(dispute_round, outcome);
		self.markets.insert(&market_id, &market);
		if to_return > 0 {
			logger::log_dispute_withdraw(market_id, env::predecessor_account_id(), dispute_round, outcome);

			return fun_token::transfer(env::predecessor_account_id(), U128(to_return), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		} else {
			panic!("user has no participation in this dispute");
		}
	}

	/**
	 * @notice Claims a users earnings in a finalized market
	 * @dev Panics if user already claimed earnigns
	 *  Panics if the market is finalized
	 * @param market_id The id of the market that earnings are going to be claimed for
	 * @param account_id The account_id of the user to claim earnings for
	 */
	pub fn claim_earnings(
		&mut self, 
		market_id: U64, 
		account_id: String
	) {
		let market_id: u64 = market_id.into();
		let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		let market_creator = market.creator.to_string();
		let claimed_earnings = market.claimed_earnings.get(&account_id);
		assert_eq!(claimed_earnings.is_none(), true, "user already claimed earnings");
		assert!(env::block_timestamp() / 1000000 >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, true, "market isn't resoluted yet");
		assert_eq!(market.finalized, true, "market isn't finalized yet");

		market.claimed_earnings.insert(&account_id, &true);
		let (winnings, left_in_open_orders, governance_earnings) = market.get_claimable_internal(account_id.to_string());

		let mut validity_bond = 0;
		if account_id == market.creator && market.validity_bond_claimed == false && market.winning_outcome != None {
			validity_bond = self.creation_bond;
			market.validity_bond_claimed = true;			
		}

		let claimable_if_invalid = match market.winning_outcome {
			None =>  market.claimable_if_invalid.get(&account_id).unwrap_or(0),
			_ => 0
		};
		let claimable_if_valid = match market.winning_outcome {
			Some(_) =>  market.claimable_if_valid.get(&account_id).unwrap_or(0),
			_ => 0
		};

		let total_feeable_amount = winnings + claimable_if_invalid;
		let resolution_fee = (total_feeable_amount * market.resolution_fee_percentage + 10000 - 1) / 10000;
		let market_creator_fee = (total_feeable_amount * self.get_creator_fee_percentage(&market) + 10000 - 1) / 10000;
		let total_fee = resolution_fee + market_creator_fee;

		let to_claim = total_feeable_amount + governance_earnings + left_in_open_orders + validity_bond + claimable_if_valid - total_fee;
		
		if to_claim == 0 {panic!("can't claim 0 tokens")}

		logger::log_earnings_claimed(market_id, env::predecessor_account_id(), to_claim);
		
		self.markets.insert(&market_id, &market);
		if market_creator_fee > 0 {
			fun_token::transfer(account_id.to_string(), U128(to_claim), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS).then(
				fun_token::transfer(market_creator, U128(market_creator_fee), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS)
			);
		} else {
			fun_token::transfer(account_id.to_string(), U128(to_claim), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		}
		
	}

	/**
	 * @notice Claim the affiliate fees accumulated by a certain account
	 * @dev Panics if the account doens't have any fees to claim
	 * @return External contract call to transfer earnings to the user
	 * TODO: reimplement affiliate fees
	 */
	pub fn claim_affiliate_earnings(
		&mut self,
		account_id: String
	) -> Promise {
		let affiliate_earnings = self.affiliate_earnings.get(&account_id).expect("account doesn't have any affiliate fees to collect");
		if affiliate_earnings > 0 {
			logger::log_affiliate_earnings_claimed(account_id.to_string(), affiliate_earnings);
			self.affiliate_earnings.insert(&account_id, &0);
			return fun_token::transfer(account_id.to_string(), U128(affiliate_earnings), &self.fun_token_account_id(), 0, SINGLE_CALL_GAS);
		} else {
			panic!("account doesn't have any affiliate fees to collect");
		}	
	}


	
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use super::*;
	mod utils;
	use utils::{ntoy, ExternalUser, init_markets_contract};
    use near_sdk::MockedBlockchain;
    use near_sdk::{VMContext, testing_env};
	use near_runtime_standalone::{RuntimeStandalone};
	use near_primitives::transaction::{ExecutionStatus, ExecutionOutcome};

	fn to_dai(amt: u128) -> u128 {
		return amt * 1e18 as u128;
	}

	fn flux_protocol() -> String {
		return "flux_protocol".to_string();
	}

	fn judge() -> String {
		return "flux-dev".to_string();
	}

	fn affiliate() -> String {
		return "affiliate".to_string();
	}

	fn alice() -> String {
		return "alice.near".to_string();
	}

	fn carol() -> String {
		return "carol.near".to_string();
	}

	fn bob() -> String {
		return "bob.near".to_string();
	}

	fn empty_string() -> String {
		return "".to_string();
	}

	fn categories () -> Vec<String> {
		return vec![];
	}

	fn outcome_tags(
		number_of_outcomes: u64
	) -> Vec<String> {
		let mut outcomes: Vec<String> = vec![];
		for _ in 0..number_of_outcomes {
			outcomes.push(empty_string());
		}
		return outcomes;
	}

	fn current_block_timestamp() -> u64 {
		return 123789;
	}
	
	fn market_creation_timestamp() -> u64 {
		return 12378;
	}
	fn market_end_timestamp_ns() -> u64 {
		return 12379000000;
	}
	fn market_end_timestamp_ms() -> u64 {
		return 12379;
	}

	fn get_context(
		predecessor_account_id: String, 
		block_timestamp: u64
	) -> VMContext {

		VMContext {
			current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
			block_index: 0,
			epoch_height: 0,
            account_balance: 0,
			is_view: false,
            storage_usage: 0,
			block_timestamp: block_timestamp,
			account_locked_balance: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(12),
            random_seed: vec![0, 1, 2],
            output_data_receivers: vec![],
		}
	}

	fn init_runtime_env() -> (RuntimeStandalone, ExternalUser, Vec<ExternalUser>) {
		let (mut runtime, root) = init_markets_contract();


		let mut accounts: Vec<ExternalUser> = vec![];
		for acc_no in 0..2 {
			let acc = if let Ok(acc) =
				root.create_external(&mut runtime, format!("account_{}", acc_no), ntoy(100))
			{
				acc
			} else {
				break;
			};
			accounts.push(acc);
		}

		root.deploy_fun_token(&mut runtime, accounts[0].get_account_id(), U128(to_dai(100000000))).unwrap();

		return (runtime, root, accounts);
	}

	mod binary_order_matching_tests;
	mod categorical_market_tests;
	mod init_tests; 
	mod market_order_tests;
	mod order_sale_tests; 
	mod market_resolution_tests; 
	mod claim_earnings_tests;
	mod validity_bond_tests;
	mod fee_payout_tests;
	mod market_dispute_tests;
}

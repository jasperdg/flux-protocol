use near_sdk::{
	near_bindgen, 
	env,
	AccountId,
	ext_contract, 
 	Promise, 
	PromiseOrValue, 
	json_types::{U128, U64},
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
/*** Import utils ***/
use crate::utils;
/*** Import constants ***/
use crate::constants;

/*** Create market type ***/
type Market = market::Market;


/**
 * @notice The state struct for the Flux Protocol implementation 
 */
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct FluxProtocol {
	owner: AccountId,
	markets: UnorderedMap<u64, Market>,
	nonce: u64,
	max_fee_percentage: u32,
	creation_bond: u128, 
	affiliate_earnings: UnorderedMap<AccountId, u128>,
	fun_token_account_id: AccountId,
	min_stake: u128,
}

/*** External Contract Interfaces ***/
/** @dev To interact with an external contract there needs to be an interface defined in the form of a trait */

/**
 * @notice Contract interface for the Fungible Token contract we're using:
 * @dev based on older version of: https://github.com/near/near-sdk-rs/tree/master/examples/fungible-token
 */
#[ext_contract]
pub trait FunToken {
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128);
    fn transfer(&mut self, new_owner_id: AccountId, amount: U128);
    fn get_total_supply(&self) -> u128;
    fn get_balance(&self, owner_id: AccountId) -> u128;
}

/**
 * @notice Contract interface for the Flux Protocol contract itself: 
 * @dev: We need to define this interface to be able to call Flux Protocol methods in a promise chain, which is required in NEAR promise API
 *  for more info checkout the Promise api: <https://github.com/near/near-sdk-rs/blob/master/near-sdk/src/promise.rs>
 */
#[ext_contract]
pub trait FluxProtocol {
    fn proceed_order_placement(&mut self, sender: AccountId, market_id: u64, outcome: u8, shares: u128, spend: u128, price: u16, affiliate_account_id: Option<AccountId>);
    fn proceed_market_resolution(&mut self, sender: AccountId, market_id: u64, winning_outcome: Option<u8>, stake: u128, gas: u64);
	fn proceed_market_dispute(&mut self, sender: AccountId, market_id: u64, winning_outcome: Option<u8>, stake: u128, gas: u64);
	fn proceed_market_creation(&mut self, sender: AccountId, description: String, extra_info: String, outcomes: u8, outcome_tags: Vec<String>, categories: Vec<String>, end_time: u64, creator_fee_percentage: u32, resolution_fee_percentage: u32, affiliate_fee_percentage: u32, api_source: String);
}

/**
 * @dev Flux Protocol contract is unusable until it is initialized and should be initialized in the same transaction as it's deployment
 8  checkout the near-cli deploy method: <https://github.com/near/near-cli>
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
		owner: AccountId, 
		fun_token_account_id: AccountId
	) -> Self {
		assert!(!env::state_exists(), "Already initialized");
		assert!(env::is_valid_account_id(owner.as_bytes()), "Invalid account_id for owner");
		assert!(env::is_valid_account_id(fun_token_account_id.as_bytes()), "Invalid account_id for FunToken");
		Self {
			owner,
			markets: UnorderedMap::new(b"markets".to_vec()),
			nonce: 0,
			max_fee_percentage: 500,
			creation_bond: constants::TOKEN_DENOMINATION / 4, // 0.25 dai
			affiliate_earnings: UnorderedMap::new(b"affiliate_earnings".to_vec()), // This Map is not used for for now, we're adding affiliate fees back in on the next V of the protocol
			fun_token_account_id,
			min_stake: constants::TOKEN_DENOMINATION / 10
		}
	}

	/*** Getters ***/
	/**
	 * @notice Returns the owner's account id
	 * @return owner's account id
	 */
	pub fn owner(
		&self
	) -> AccountId {
		self.owner.to_string()
	}

	/**
	 * @notice Returns the fungible token's account id
	 * @return Fungible token's account id
	 */
	fn fun_token_account_id(
		&self
	) -> AccountId {
		self.fun_token_account_id.to_string()
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
		self.markets
			.get(&market_id)
			.expect("market doesn't exist")
			.filled_volume
			.into()
	}

	/**
	 * @notice returns market price
	 * @dev only needed for unit tests
	 */
	pub fn get_market_price(
		&self,
		market_id: U64,
		outcome: u8
	) -> u16 {
		let market_id: u64 = market_id.into();
		self.markets
			.get(&market_id)
			.expect("market doesn't exist")
			.get_market_price(outcome)
	}

	/**
	 * @notice returns an account's balance in a certain market for a certain outcome
	 * @dev only needed for unit tests
	 */
	pub fn get_outcome_share_balance(
		&self,
		account_id: &AccountId,
		market_id: U64,
		outcome: u8,
	) -> U128 {
		let market_id: u64 = market_id.into();

		/* Get user_data for an outcome in a market */
		let market = self.markets.get(&market_id).expect("non existent market");
		let orderbook = market.orderbooks.get(&outcome).expect("non existent outcome");
		let user_data = orderbook.user_data.get(account_id);

		/* If there is no data for this account_id return 0 */
		if user_data.is_none() {return U128(0)}

		U128(user_data.unwrap().balance)
	}



	/**
	 * @notice Calculates and returns the amount a user can claim in a market if the current resolution data is correct
	 * @param market A reference to the market where from to return the creator fee
	 * @return Returns the amount of base tokens claimable denominated in 1e18
	 */
	pub fn get_claimable(
		&self, 
		market_id: U64, 
		account_id: AccountId
	) -> U128 {
		let market_id: u64 = market_id.into();
		let market = self.markets.get(&market_id).expect("market doesn't exist");

		/* Check if account_id has claimed earnings in this market, if so return 0 */
		if market.claimed_earnings.contains(&account_id) {
			return U128(0);
		}

		/* If account_id is the market creator, and if the market was resolved as being valid. If this is the case account_id is eligable to receive the validity bond back */ 
		let validity_bond = if account_id == market.creator && !market.validity_bond_claimed && market.winning_outcome != None { self.creation_bond } else { 0 };
		 
		/* Get how much would be claimable for account_id, governance earnings relates to what we call "market governance" or the dispute resolution process */
		let (winnings, left_in_open_orders, governance_earnings) = market.get_claimable_internal(&account_id);
		
		let claimable_if_invalid = match market.winning_outcome {
			None =>  market.claimable_if_invalid.get(&account_id).unwrap_or(0),
			_ => 0
		};

		let claimable_if_valid = match market.winning_outcome {
			Some(_) =>  market.claimable_if_valid.get(&account_id).unwrap_or(0),
			_ => 0
		};

		/* Calculate the sum of winnings + claimable_if_invalid to determined what amount of funds can be feed */
		let total_feeable_amount = winnings + claimable_if_invalid;

		/* Calculate total fee percentage */
		let total_fee_percentage =  market.resolution_fee_percentage + utils::get_creator_fee_percentage(&market);

		/* Calculate total fee */
		let total_fee = (total_feeable_amount * u128::from(total_fee_percentage)) / u128::from(constants::PERCENTAGE_PRECISION);
		
		/* Calculate the total amount claimable */
		let to_claim = total_feeable_amount + governance_earnings + left_in_open_orders + validity_bond + claimable_if_valid - total_fee;

		U128(to_claim)
	}

	/*** Setters ***/

	/**
	 * @notice Change ownership - will be used to change "judge" to either a multisig or a last resort measure
	 * @dev Panics if the sender isn't the current owner
	 */
	pub fn set_owner(
		&mut self, 
		new_owner: AccountId
	) {
		assert_eq!(env::predecessor_account_id(), self.owner, "Owner can only be changed by previous owner");
		self.owner = new_owner;
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
	 * @param end_time Unix timestamp in milliseconds of when the market stops being tradeable and can be resolved
	 * @param creator_fee_percentage Percentage with two decimals so denominated in 1e4 between 0 - 500 where 1 = 0.01% and 100 = 1%
	 * @param affiliate_fee_percentage Percentage of the creator fee that should go to affiliate accounts range between 1 - 100
	 * @param api_source For when we have validators running, these validators then use this attribute to automatically resolute / dispute the market
	 * @return returns a promise chain - this chain tries to escrow the base currency as a validity bond from the market creation and if successful proceed the market creation
	 * */
	pub fn create_market(
		&mut self, 
		description: String, 
		extra_info: String, 
		outcomes: u8,
		outcome_tags: Vec<String>,
		categories: Vec<String>,
		end_time: U64,
		creator_fee_percentage: u32,
		affiliate_fee_percentage: u32,
		api_source: String,
		gas_arr: Option<Vec<U64>>
	) -> Promise {
		let end_time: u64 = end_time.into();

		for outcome_tag in &outcome_tags {
			assert!(outcome_tag.chars().count() < 20, "outcome tag can't be more than 20 chars");
		}
		
		for category in &categories {
			assert!(category.chars().count() < 20, "category tag can't be more than 20 chars");
		}
		
		utils::assert_gas_arr_validity(&gas_arr, 2);
		assert!(description.chars().count() < 201, "description can't be longer than 200 characters");
		assert!(extra_info.chars().count() < 401, "extra_info can't be longer than 400 characters");
		assert!(outcomes > 1, "need to have more than 2 outcomes");
		assert!(outcomes == 2 || outcomes == outcome_tags.len() as u8, "invalid outcomes");
		assert!(outcomes < 8, "can't have more than 8 outcomes"); // up for change
		assert!(end_time > utils::ns_to_ms(env::block_timestamp()), "end_time has to be greater than NOW");
		assert!(categories.len() < 8, "can't have more than 8 categories");
		assert!(creator_fee_percentage <= self.max_fee_percentage, "creator_fee_percentage too high");
		assert!(affiliate_fee_percentage <= constants::PERCENTAGE_PRECISION, "affiliate_fee_percentage can't be higher than 100.00%");

		if outcomes == 2 { assert!(outcome_tags.is_empty(), "If a binary markets the outcomes are always assumed to be ['NO', 'YES'] so there is no need for provide outcome_tags") }

		/* Promise chain, call external token contract to transfer funds from user to flux protocol contract. Then self call proceed_market_creation. */
		fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), self.creation_bond.into(), &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS)).then(
			flux_protocol::proceed_market_creation(
				env::predecessor_account_id(), 
				description,
				extra_info,
				outcomes,
				outcome_tags,
				categories,
				end_time,
				creator_fee_percentage, 
				100, // = 1%
				affiliate_fee_percentage,
				api_source,
				&env::current_account_id(),
				0,
				utils::get_gas_for_tx(&gas_arr, 1, constants::SINGLE_CALL_GAS)
			)
		)
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
	 * @param end_time Unix timestamp in milliseconds of when the market stops being tradeable and can be resolved
	 * @param creator_fee_percentage Percentage with two decimals so denominated in 1e4 between 0 - 500 where 1 = 0.01% and 100 = 1%
	 * @param affiliate_fee_percentage Percentage of the creator fee that should go to affiliate accounts range between 1 - 100
	 * @param api_source For when we have validators running, these validators then use this attribute to automatically resolute / dispute the market
	 * @return Returns the newly created market_id
	 * TODO: Just logs the vec types out instead of actually storing them, there is no filtering on chain
	 */
	pub fn proceed_market_creation(
		&mut self, 
		sender: AccountId, 
		description: String, 
		extra_info: String, 
		outcomes: u8, 
		outcome_tags: Vec<String>, 
		categories: Vec<String>, 
		end_time: u64, 
		creator_fee_percentage: u32, 
		resolution_fee_percentage: u32, 
		affiliate_fee_percentage: u32, 
		api_source: String
	) -> PromiseOrValue<u64> {
		/* Make sure that the caller of this method is the contract itself */
		utils::assert_self();
		/* Make sure the previous promise in the promise chain was successful */
		utils::assert_prev_promise_successful();

		/* Create new market instance */
		let new_market = Market::new(
			self.nonce,
			sender,
			description,
			extra_info,
			outcomes,
			end_time,
			creator_fee_percentage,
			resolution_fee_percentage,
			affiliate_fee_percentage,
			api_source
		);
		
		/* Get the newly created market's resolution_window */
		let resolution_window = new_market.resolution_windows.get(0).expect("something went wrong during market creation");

		logger::log_market_creation(&new_market, outcome_tags, categories);
		logger::log_new_resolution_window(new_market.id, resolution_window.round, resolution_window.required_bond_size, resolution_window.end_time);

		let market_id = new_market.id;
		
		/* Re-insert the markets into the markets map with the market_id as key */
		self.markets.insert(&self.nonce, &new_market);

		/* Increment nonce, for next market's id */
		self.nonce += 1;

		PromiseOrValue::Value(market_id)
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
		outcome: u8,
		shares: U128,
		price: u16,
		affiliate_account_id: Option<AccountId>,
		gas_arr: Option<Vec<U64>>
	) -> Promise {
		let market_id: u64 = market_id.into();
		let shares: u128 = shares.into();
		let rounded_spend = shares * u128::from(price);
		let market = self.markets.get(&market_id).expect("market doesn't exist");

		utils::assert_gas_arr_validity(&gas_arr, 2);
		assert!(rounded_spend >= constants::TOKEN_DENOMINATION / 10, "order must be valued at > 0.1 tokens");
		assert!(price > 0 && price < 100, "price can only be between 1 - 99");
		assert!(outcome < market.outcomes, "invalid outcome");
		assert_eq!(market.resoluted, false, "market has already been resoluted");
		assert!(utils::ns_to_ms(env::block_timestamp()) < market.end_time, "market has already ended");

		let transfer_gas = utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS / 10);
		let order_placement_gas = utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS * 10 - transfer_gas);

		/* Attempt to transfer deposit the tokens from the user to this contract, then continue order placement */
		fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), rounded_spend.into(), &self.fun_token_account_id(), 0, transfer_gas)
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
				utils::get_gas_for_tx(&gas_arr, 0, order_placement_gas)
			)
		)
	}

	/** 
	 * @notice Kicks off order placement
	 * @dev Panics if the signer isn't the contract itself
	 *  panics if the previous promise wasn't successful due to lack of balance or allowance
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
		sender: AccountId,
		market_id: u64, 
		outcome: u8,
		shares: u128,
		spend: u128,
		price: u16,
		affiliate_account_id: Option<AccountId>,
	) -> PromiseOrValue<bool> {
		/* Make sure that the caller of this method is the contract itself */
		utils::assert_self();
		/* Make sure the previous promise in the promise chain was successful */
		utils::assert_prev_promise_successful();
		
		let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		market.place_order_internal(&sender, outcome, shares, spend, price, affiliate_account_id);
		self.markets.insert(&market.id, &market);
		PromiseOrValue::Value(true)
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
		outcome: u8,
		shares: U128,
		min_price: u16,
		gas_arr: Option<Vec<U64>>
	) {
		let market_id: u64 = market_id.into();
		let shares: u128 = shares.into();
		
		utils::assert_gas_arr_validity(&gas_arr, 1);
		assert!(min_price > 0, "min_price need to be higher than 0");
		assert!(min_price < 100, "min_price need to be smaller than 100");
		assert!(shares > 0, "can't sell 0 shares");
		
		let mut market = self.markets.get(&market_id).expect("non existent market");
		assert_eq!(market.finalized, false, "can't sell shares after market is finalized");
		let earnings = market.dynamic_market_sell_internal(env::predecessor_account_id(), outcome, shares, min_price);
		assert!(earnings > 0, "no matching orders");
		self.markets.insert(&market_id, &market);
		
		fun_token::transfer(env::predecessor_account_id(), U128(earnings), &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS));
	}

	/**
	 * @notice Cancels an order and returns outstanding open value to order creator
	 * @dev Panics if the predecessor_account isn't the owner of the order he's trying to cancel
	 *  Panics if market is already resolved, open orders are included in the claimable amount 
	 * @param market_id The id of the market this order was placed on before
	 * @param outcome The outcome this order was for
	 * @param price The price this order was placed at, this is necessary because of the way orders are stored
	 * @param order_id The id of the order that's to be canceled
	 */
	pub fn cancel_order(
		&mut self, 
		market_id: U64, 
		outcome: u8,
		price: u16,
		order_id: U128,
		gas_arr: Option<Vec<U64>>
	) {
		let market_id: u64 = market_id.into();
		let order_id: u128 = order_id.into();
		
		let mut market = self.markets.get(&market_id).expect(format!("market with id: {} does not exist", market_id).as_str());

		utils::assert_gas_arr_validity(&gas_arr, 1);
		assert_eq!(market.resoluted, false);
		/* Get corresponding outcome orderbook */
		let mut orderbook = market.orderbooks.get(&outcome).expect(format!("outcome: {} does not exist for market with id: {}", outcome, market_id).as_str());
		let price_data = orderbook.price_data.get(&price).expect("order at this price doesn't exist");
		let order = price_data.orders.get(&order_id).expect("order with this id doesn't exist or is already canceled");
		assert!(env::predecessor_account_id() == order.creator, "not this user's order");

		/* Cancel the order, this returns how much value was left in the open order */
		let to_return = orderbook.cancel_order(&order);
		
		/* Reinsert the orderbook and market to update state */
		market.orderbooks.insert(&outcome, &orderbook);
		self.markets.insert(&market_id, &market);

		/* Transfer value left in open order to order owner */
		fun_token::transfer(env::predecessor_account_id(), to_return.into(), &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS));
    }

	/**
	 * @notice Kicks off market resolution, supply the outcome data to the 
	 * @dev Panics if the market hasn't ended yet
	 *  Panics if the market doesn't exist
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
		winning_outcome: Option<u8>,
		stake: U128,
		gas_arr: Option<Vec<U64>>,
	) -> Promise {
		let market_id: u64 = market_id.into();
		let stake_u128: u128 = stake.into();
		let market = self.markets.get(&market_id).expect("market doesn't exist");

		utils::assert_gas_arr_validity(&gas_arr, 3);
		assert!(stake_u128 >= self.min_stake, format!("stake needs to greater than min_stake of {}", self.min_stake));
		assert!(utils::ns_to_ms(env::block_timestamp()) >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, false, "market is already resoluted");
		assert_eq!(market.finalized, false, "market is already finalized");
		assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");

		let external_gas: u64 = (*gas_arr.as_ref().unwrap_or(&vec![]).get(2).unwrap_or(&U64(constants::SINGLE_CALL_GAS))).into();

		fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS) / 2)
		.then(
			flux_protocol::proceed_market_resolution(
				env::predecessor_account_id(),
				market_id,
				winning_outcome,
				stake_u128,
				external_gas,
				&env::current_account_id(),
				0,
				utils::get_gas_for_tx(&gas_arr, 1, constants::SINGLE_CALL_GAS)
			)
		)
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
		winning_outcome: Option<u8>,
		stake: u128,
		sender: AccountId,
		gas: u64,
	) -> PromiseOrValue<bool> {
		/* Make sure that the caller of this method is the contract itself */
		utils::assert_self();
		/* Make sure the previous promise in the promise chain was successful */
		utils::assert_prev_promise_successful();

		let mut market = self.markets.get(&market_id).unwrap();
		
		/* Resolute the market, which returns how much of the stake the sender overpaid */
		let change = market.resolute_internal(&sender, winning_outcome, stake);
		self.markets.insert(&market_id, &market);

		/* If the sender stakes more than necessary to progress to the next dispute round  */
		if change > 0 {
			let prom = fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, gas / 2);
			PromiseOrValue::Promise(prom)
		} else {
			PromiseOrValue::Value(true)
		}
	}

	/**
	 * @notice Kicks of a dispute of a certain outcome
	 * @dev Panics if the market hasn't been resoluted yet
	 *  Panics if the market doesn't exist
	 *  Panics if the market is already finalized
	 *  Panics if the winning_outcome is invalid
	 *  Panics if the disputed outcome is the same outcome as the previous winning outcome
	 *  Panics if the sender doesn't have enough balance / allowance to transfer `stake`
	 *  Panics if the dispute round is > 1. After one initial dispute the market has to be finalized by the owner ("judge")
	 * @param market_id The id of the market to dispute
	 * @param winning_outcome The winning_outcome according to the staker
	 * @param stake The amount of stake the sender wants to contribute to the dispute round
	 */
	pub fn dispute_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<u8>,
		stake: U128,
		gas_arr: Option<Vec<U64>>,
	) -> Promise {
		let market_id: u64 = market_id.into();
		let stake_u128: u128 = stake.into();
        let market = self.markets.get(&market_id).expect("market doesn't exist");
		
		utils::assert_gas_arr_validity(&gas_arr, 2);
		assert!(stake_u128 >= self.min_stake, format!("stake needs to greater than min_stake of {}", self.min_stake));
		assert_eq!(market.resoluted, true, "market isn't resoluted yet");
		assert_eq!(market.finalized, false, "market is already finalized");
        assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid winning outcome");
        assert!(winning_outcome != market.winning_outcome, "same outcome as last resolution");
		let resolution_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("Invalid dispute window unwrap");
		assert_eq!(resolution_window.round, 1, "for this version, there's only 1 round of dispute");
		assert!(utils::ns_to_ms(env::block_timestamp()) < resolution_window.end_time, "dispute window is closed, market can be finalized");

		let external_gas: u64 = (*gas_arr.as_ref().unwrap_or(&vec![]).get(2).unwrap_or(&U64(constants::SINGLE_CALL_GAS))).into();

		/* Transfer from sender to contract then proceed dispute */
		fun_token::transfer_from(env::predecessor_account_id(), env::current_account_id(), stake, &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS / 2)).then(
			flux_protocol::proceed_market_dispute(
				env::predecessor_account_id(),
				market_id,
				winning_outcome,
				stake_u128,
				external_gas,
				&env::current_account_id(), 
				0, 
				utils::get_gas_for_tx(&gas_arr, 1, constants::SINGLE_CALL_GAS)
			)
		)
	}

	/**
	 * @notice Continues the dispute process if transfer of funds was successful
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
		winning_outcome: Option<u8>,
		stake: u128,
		sender: AccountId,
		gas: u64,
	) -> PromiseOrValue<bool> {
		/* Make sure that the caller of this method is the contract itself */
		utils::assert_self();
		/* Make sure the previous promise in the promise chain was successful */
		utils::assert_prev_promise_successful();
        let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		
		/* Resolute the market, which returns how much of the stake the sender overpaid */
		let change = market.dispute_internal(&sender, winning_outcome, stake);

		self.markets.insert(&market.id, &market);
		
		/* If the sender stakes more than necessary to progress to the next dispute round  */
		if change > 0 {
			PromiseOrValue::Promise(fun_token::transfer(sender, U128(change), &self.fun_token_account_id(), 0, gas / 2))
		} else {
			PromiseOrValue::Value(true)
		}
	}

	/**
	 * @notice Finalizes a market once disputed or the dispute window has been closed
	 * @dev Panics if the market hasn't been resoluted yet
	 *  Panics if the market is disputed and finalize is not called by the judge
	 *	Panics if the dispute window is still open
	 *	Panics if the winning_outcome is an invalid outcome
	 * @param market_id The id of the market to finalize
	 * @param winning_outcome Optional in case the market has been disputed, the judges ruling
	 */
	pub fn finalize_market(
		&mut self, 
		market_id: U64, 
		winning_outcome: Option<u8>
	) {
		let market_id: u64 = market_id.into();
		let mut market = self.markets.get(&market_id).expect(format!("market with id: {} does not exist", market_id).as_str());
		assert!(winning_outcome == None || winning_outcome.unwrap() < market.outcomes, "invalid outcome");
		assert_eq!(market.resoluted, true, "market has to be resoluted before it can be finalized");

		if market.disputed {
			/* If the market is disputed this means that the market is to be finalized by the owner */
			assert_eq!(env::predecessor_account_id(), self.owner, "only the judge can resolute disputed markets");
		} else {
			/* If the market is not disputed it can be resoluted as soon as the dispute window is closed */
			let dispute_window = market.resolution_windows.get(market.resolution_windows.len() - 1).expect("no dispute window found, something went wrong");
			assert!(utils::ns_to_ms(env::block_timestamp()) >= dispute_window.end_time || dispute_window.round == 2, "dispute window still open")
		}

		/* Finalize the market and re-insert it to update state */
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
	pub fn withdraw_resolution_stake(
		&mut self, 
		market_id: U64,
		dispute_round: U64,
		outcome: Option<u8>,
		gas_arr: Option<Vec<U64>>,
	) -> Promise {
		let market_id: u64 = market_id.into();
		let dispute_round: u64 = dispute_round.into();		
		utils::assert_gas_arr_validity(&gas_arr, 1);
		let mut market = self.markets.get(&market_id).expect("invalid market");
		let to_return = market.withdraw_resolution_stake_internal(env::predecessor_account_id(), dispute_round, outcome);

		/* If the user has stake to withdraw transfer the stake back to the user */
		if to_return > 0 {
			/* Re-insert the market into the markets struct to update state */
			self.markets.insert(&market_id, &market);
			logger::log_dispute_withdraw(market_id, &env::predecessor_account_id(), dispute_round, outcome);
			fun_token::transfer(env::predecessor_account_id(), U128(to_return), &self.fun_token_account_id(), 0, constants::SINGLE_CALL_GAS / 2)
		} else {
			panic!("user has no participation in this dispute");
		}
	}

	/**
	 * @notice Claims a users earnings in a finalized market
	 * @dev Panics if user already claimed earnings
	 *  Panics if the market is not finalized
	 *  Panics if the user has 0 tokens to claim
	 * @param market_id The id of the market that earnings are going to be claimed for
	 * @param account_id The account_id of the user to claim earnings for
	 */
	pub fn claim_earnings(
		&mut self, 
		market_id: U64, 
		account_id: AccountId,
		gas_arr: Option<Vec<U64>>,
	) {
		let market_id: u64 = market_id.into();
		let mut market = self.markets.get(&market_id).expect("market doesn't exist");
		let market_creator = market.creator.to_string();

		/* Check if account_id has claimed earnings in this market, if so return 0 */
		assert!(market.claimed_earnings.contains(&account_id), "user already claimed earnings");
		assert!(utils::ns_to_ms(env::block_timestamp()) >= market.end_time, "market hasn't ended yet");
		assert_eq!(market.resoluted, true, "market isn't resoluted yet");
		assert_eq!(market.finalized, true, "market isn't finalized yet");

		/* Make sure it is noted that user claimed earnings to avoid double claims */
		market.claimed_earnings.insert(&account_id);
		
		/* Get how much would be claimable for account_id, governance earnings relates to what we call "market governance" or the dispute resolution process */
		let (winnings, left_in_open_orders, governance_earnings) = market.get_claimable_internal(&account_id);

		/* If account_id is the market creator, and if the market was resoluted as being valid. If this is the case account_id is eligable to receive the validity bond back */ 
		let mut validity_bond = 0;
		if account_id == market.creator && !market.validity_bond_claimed && market.winning_outcome != None {
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
		
		utils::assert_gas_arr_validity(&gas_arr, 1);
		/* Calculate the sum of winnings + claimable_if_invalid to determined what amount of funds can be feed */
		let total_feeable_amount = winnings + claimable_if_invalid;

		/* Calculate total fee percentage */
		let resolution_fee = total_feeable_amount * u128::from(market.resolution_fee_percentage) / u128::from(constants::PERCENTAGE_PRECISION);
		let market_creator_fee = total_feeable_amount * u128::from(utils::get_creator_fee_percentage(&market)) / u128::from(constants::PERCENTAGE_PRECISION);
		let total_fee = resolution_fee + market_creator_fee;

		/* Calculate the total amount claimable */
		let to_claim = total_feeable_amount + governance_earnings + left_in_open_orders + validity_bond + claimable_if_valid - total_fee;
		env::log(format!("claiming for: {} {} {}", account_id, to_claim, total_feeable_amount).as_bytes());
		if to_claim == 0 {panic!("can't claim 0 tokens")}

		logger::log_earnings_claimed(market_id, &account_id, to_claim);
		
		/* Reinsert market instance to update claim state */
		self.markets.insert(&market_id, &market);

		if market_creator_fee > 0 {
			/* If the market_creator_fee > 0; first transfer funds to the user, and after that transfer the fee to the market creator */
			fun_token::transfer(account_id, U128(to_claim), &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS)).then(
				fun_token::transfer(market_creator, U128(market_creator_fee), &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS))
			);
		} else {
			/* If the market_creator_fee == 0; Just transfer the user his earnings */
			fun_token::transfer(account_id, U128(to_claim), &self.fun_token_account_id(), 0, utils::get_gas_for_tx(&gas_arr, 0, constants::SINGLE_CALL_GAS));
		}
	}	
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
	use super::*;
	mod utils;
	use utils::{ntoy, ExternalUser, init_markets_contract};
    use near_sdk::{VMContext};
	use near_runtime_standalone::{RuntimeStandalone};
	use near_primitives::transaction::{ExecutionStatus};

	fn to_dai(amt: u128) -> u128 {
		amt * constants::TOKEN_DENOMINATION
	}

	fn to_shares(amt: u128) -> u128 {
		amt * constants::SHARE_DENOMINATION
	}
	
	fn flux_protocol() -> AccountId {
		"flux_protocol".to_string()
	}

	fn judge() -> AccountId {
		"flux-dev".to_string()
	}

	fn affiliate() -> AccountId {
		"affiliate".to_string()
	}

	fn alice() -> AccountId {
		"alice.near".to_string()
	}

	fn carol() -> AccountId {
		"carol.near".to_string()
	}

	fn bob() -> AccountId {
		"bob.near".to_string()
	}

	fn empty_string() -> AccountId {
		"".to_string()
	}

	fn categories () -> Vec<String> {
		vec![]
	}

	fn outcome_tags(
		number_of_outcomes: u8
	) -> Vec<String> {
		let mut outcomes: Vec<String> = vec![];
		for _ in 0..number_of_outcomes {
			outcomes.push(empty_string());
		}
		outcomes
	}

	fn current_block_timestamp() -> u64 {
		123_789
	}
	
	fn market_creation_timestamp() -> u64 {
		12378
	}
	fn market_end_timestamp_ns() -> u64 {
		12_379_000_000
	}
	fn market_end_timestamp_ms() -> u64 {
		12379
	}

	fn get_context(
		predecessor_account_id: AccountId, 
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
			block_timestamp,
			account_locked_balance: 0,
            attached_deposit: 0,
            prepaid_gas: 10_u64.pow(12),
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

		root.deploy_fun_token(&mut runtime, accounts[0].get_account_id(), U128(to_dai(100_000_000))).unwrap();

		(runtime, root, accounts)
	}

	mod init_tests;
	mod binary_order_matching_tests;
	mod categorical_market_tests;
	mod market_order_tests;
	mod order_sale_tests; 
	mod market_resolution_tests; 
	mod claim_earnings_tests;
	mod validity_bond_tests;
	mod fee_payout_tests;
	mod market_dispute_tests;
	mod custom_gas_tests;
}

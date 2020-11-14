use near_sdk::{
    env,
    json_types::{U64},
    PromiseResult
};

/*** Import market implementation ***/
use crate::market::Market;
/*** Import constants ***/
use crate::constants;

/**
 * @dev Checks if the method called is the contract itself
 *  panics if `predecessor_account` (sender) isn't the protocol's `account_id`
 */
pub fn assert_self() {
    assert_eq!(env::current_account_id(), env::predecessor_account_id(), "this method can only be called by the contract itself"); 
}

/**
 * @dev Checks if the previous promise in the promise chain passed successfully
 *  panics if the previous promise in the promise chain was unsuccessful
 */
pub fn assert_prev_promise_successful() {
    assert!(is_promise_success(), "previous promise failed");
}

/**
 * @dev Checks if the previous promise in the promise chain passed successfully
 *  panics if the previous promise in the promise chain was unsuccessful
 */
pub fn assert_gas_arr_validity(gas_arr: &Option<Vec<U64>>, num_of_promises: usize) {
    assert!(gas_arr.is_none() || gas_arr.as_ref().unwrap().len() == num_of_promises, "if custom gas values are provided there needs to be a specified value for each of the external transactions");
}

/**
 * @notice Check if previous promise in promise chain was executed successfully 
 * @dev Panics if the previous promise in the promise chain was unsuccessful
 * @dev Taken from: <https://github.com/near/core-contracts/blob/a009f52ccf5e36db75cf31104604eaec69dd67f8/lockup/src/utils.rs#L7>
 * @dev Latest commit which is still up to date with this version: e3688c90dc01f69735fb02178b0f98297dee08c0
 * @return Returns a bool representing the success of the previous promise in a promise chain
 */
pub fn is_promise_success() -> bool {
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

/**
 * @notice Parse a gas array to return the gas amount for a certain tx
 * @return Returns the amount of gas to be attached to the transaction
 */
pub fn get_gas_for_tx(gas_arr: &Option<Vec<U64>>, index: usize, default_gas: u64) -> u64 {
    (*gas_arr.as_ref().unwrap_or(&vec![]).get(index).unwrap_or(&U64(default_gas))).into()
}

/** 
 * @notice Converts nano seconds to milliseconds by dividing the ns amount by `1_000_000`
 * @return Returns current `block_timestamp` denominated in ms
*/
pub fn ns_to_ms(timestamp_ns: u64) -> u64 {
    timestamp_ns / 1_000_000
} 

pub fn one_token() -> u128 {
    10_u128.pow(18)
}

/**
 * @notice Returns the market's `creator_fee`. If the market is resoluted as invalid the creator's fee is slashed so this method returns 0. 
 * @param market A reference to the market where the `fee_percentage` should be returned from
 * @return Returns a u128 integer representing the `creator_fee_percentage` denominated in 1e4, meaning 1 == 0.01%
 */
pub fn get_creator_fee_percentage(
    market: &Market
) -> u32 {
    match market.winning_outcome {
        Some(_) => market.fees.creator_fee_percentage,
        None => 0
    }
}

pub fn calc_fee(
    feeable: u128, 
    fee_percentage: u32
) -> u128 {
    feeable * u128::from(fee_percentage) / u128::from(constants::PERCENTAGE_PRECISION)
}

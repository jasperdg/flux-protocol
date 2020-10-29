use near_sdk::{
    env,
    json_types::{U64},
    PromiseResult
};

/**
 * @dev Checks if the method called is the contract itself
 *  panics if predecessor_account (sender) isn't the FluxProtcol account id
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
    assert!(gas_arr.is_none() || gas_arr.as_ref().unwrap().len() == num_of_promises, "if custom gas vals are provided there needs to be a specified value for each of the external transactions");
}

/**
 * @dev Panics if the previous promise in the promise chain was unsuccessful
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
    return (*gas_arr.as_ref().unwrap_or(&vec![]).get(index).unwrap_or(&U64(default_gas))).into();
}
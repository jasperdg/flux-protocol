use near_sdk::{
    env,
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
    assert_eq!(is_promise_success(), true, "previous promise failed");
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
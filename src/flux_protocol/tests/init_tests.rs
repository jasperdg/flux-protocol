use super::*;

#[test]
fn test_contract_creation() {
    let (ref mut runtime, ref root) = init_markets_contract();

}

#[test]
fn test_runtime_init() {
	let (runtime, root, accounts) = init_runtime_env();
}

#[test]
fn test_external_token() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].get_balance(&mut runtime, accounts[0].get_account_id());
	assert_eq!(tx_res, U128(to_dai(100000000)));
}

#[test]
fn test_market_creation() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	assert_eq!(accounts[0].get_owner(&mut runtime), root.get_account_id());
}
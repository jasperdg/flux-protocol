use super::*;

#[test]
fn simplest_binary_order_matching_test() {
	let (mut runtime, _root, accounts) = init_runtime_env();

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30_000))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(110_000))).expect("allowance couldn't be set");
	accounts[0].place_order(&mut runtime, U64(0), 0, U128(to_dai(1000)), 50, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_dai(1000)), 50, None, None).expect("order placement tx failed unexpectedly");

	let no_share_balance: u128 = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 0).into();
	let yes_share_balance: u128 = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 1).into();
	assert_eq!(no_share_balance, to_dai(1000));
	assert_eq!(yes_share_balance, to_dai(1000));
}

#[test]
fn partial_binary_order_matching_test() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30_000))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(10_000_000_000))).expect("allowance couldn't be set");

	
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_dai(1000)), 50, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_dai(1000)), 50, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_dai(550)), 50, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_dai(200)), 50, None, None).expect("order placement tx failed unexpectedly");

	accounts[0].place_order(&mut runtime, U64(0), 0, U128(to_dai(1000)), 50, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 0, U128(to_dai(1555)), 50, None, None).expect("order placement tx failed unexpectedly");

	let no_share_balance = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 0);
	let yes_share_balance = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 1);
	assert_eq!(no_share_balance, U128(to_dai(2555)));
	assert_eq!(yes_share_balance, U128(to_dai(2555)));
}
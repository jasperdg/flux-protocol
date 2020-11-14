use super::*;

#[test]
fn test_categorical_market_automated_matcher() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].inc_allowance(&mut runtime, flux_protocol(), U128(to_dai(3_000_000))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	accounts[0].transfer(&mut runtime, accounts[1].get_account_id(), to_dai(10).into()).expect("transfer failed couldn't be set");
	accounts[0].inc_allowance(&mut runtime, flux_protocol(), U128(to_dai(110_000_000))).expect("allowance couldn't be set");

	accounts[0].place_order(&mut runtime, U64(0), 0, U128(to_dai(1000)), 25, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_dai(900)), 25, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 2, U128(to_dai(1000)), 25, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 3, U128(to_dai(1000)), 24, None, None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), 2, U128(to_dai(1000)), 50, None, None).expect("order placement tx failed unexpectedly");

	let balance: u128 = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 0).into();
	assert_eq!(balance, to_dai(900));
	let balance: u128 = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 1).into();
	assert_eq!(balance, to_dai(900));
	let balance: u128 = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 2).into();
	assert_eq!(balance, to_dai(900));
	let balance: u128 = accounts[0].get_outcome_share_balance(&runtime, accounts[0].get_account_id(), U64(0), 3).into();
	assert_eq!(balance, to_dai(900));

}

use super::*;

#[test]
fn test_categorical_market_automated_matcher() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	accounts[0].transfer(&mut runtime, accounts[1].get_account_id(), to_dai(10).into()).expect("transfer failed couldn't be set");
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(110000000)).expect("allowance couldn't be set");

	let res = accounts[0].place_order(&mut runtime, U64(0), 0, U128(1000), U128(25), None).expect("order placement tx failed unexpectedly");
	let res = accounts[0].place_order(&mut runtime, U64(0), 1, U128(900), U128(25), None).expect("order placement tx failed unexpectedly");
	let res = accounts[0].place_order(&mut runtime, U64(0), 2, U128(1000), U128(25), None).expect("order placement tx failed unexpectedly");
	let res = accounts[0].place_order(&mut runtime, U64(0), 3, U128(1000), U128(24), None).expect("order placement tx failed unexpectedly");
	let res = accounts[0].place_order(&mut runtime, U64(0), 2, U128(1000), U128(50), None).expect("order placement tx failed unexpectedly");

	println!("");
	println!("test {:?}", res);

	let balance: u128 = accounts[0].get_outcome_share_balance(&mut runtime, accounts[0].get_account_id(), U64(0), 0).into();
	println!("balance outcome 0: {}", balance);
	// assert_eq!(balance, U128(1000));
	let balance: u128 = accounts[0].get_outcome_share_balance(&mut runtime, accounts[0].get_account_id(), U64(0), 1).into();
	println!("balance outcome 1: {}", balance);
	// assert_eq!(balance, U128(1000));
	let balance: u128 = accounts[0].get_outcome_share_balance(&mut runtime, accounts[0].get_account_id(), U64(0), 2).into();
	println!("balance outcome 2: {}", balance);
	// assert_eq!(balance, U128(1000));
	let balance: u128 = accounts[0].get_outcome_share_balance(&mut runtime, accounts[0].get_account_id(), U64(0), 3).into();
	println!("balance outcome 3: {}", balance);
	// assert_eq!(balance, U128(1000));

}

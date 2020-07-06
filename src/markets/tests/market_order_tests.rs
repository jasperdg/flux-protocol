use super::*;

#[test]
fn test_place_order_insufficient_funds() {
	let (mut runtime, root, accounts) = init_runtime_env();

	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	
	accounts[1].set_allowance(&mut runtime, flux_protocol(), U128(5000)).expect("allowance couldn't be set");

	let account_1_res = accounts[1].place_order(&mut runtime, U64(0), U64(0), U128(50000), U128(50), None);
	assert_eq!(account_1_res.is_err(), true);
}

#[test]
fn test_order_placement_cancelation_and_market_prices() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(2000000)).expect("allowance couldn't be set");

	let tx_res = accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(50000), U128(50), None).expect("tx unexpectedly failed");
	println!("res1: {:?}", tx_res);
	let tx_res = accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(50000), U128(50), None).expect("tx unexpectedly failed");
	println!("res2: {:?}", tx_res);
	
	let no_market_price = accounts[0].get_market_price(&mut runtime, U64(0), U64(0));
	assert_eq!(no_market_price, U128(50));
	
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(50000), U128(60), None).expect("tx unexpectedly failed");

	let no_market_price = accounts[0].get_market_price(&mut runtime, U64(0), U64(0));
	assert_eq!(no_market_price, U128(40));

	accounts[0].cancel_order(&mut runtime, U64(0), U64(1), U128(2)).expect("order cancelation failed");

	// balance checks: 
	let expected_contract_balance = U128(100000);
	let expected_account_balance = U128(99999999999999999999900000);
	let account_balance = accounts[0].get_balance(&mut runtime, accounts[0].get_account_id());
	let contract_balance = accounts[0].get_balance(&mut runtime, flux_protocol());
	
	assert_eq!(expected_contract_balance, contract_balance);
	assert_eq!(expected_account_balance, account_balance);

	let no_market_price = accounts[0].get_market_price(&mut runtime, U64(0), U64(0));
	assert_eq!(no_market_price, U128(50));

	let tx_res = accounts[0].cancel_order(&mut runtime, U64(0), U64(1), U128(1)).expect("order cancelation failed");
	let tx_res = accounts[0].cancel_order(&mut runtime, U64(0), U64(1), U128(0)).expect("order cancelation failed");

	let expected_account_balance = U128(100000000000000000000000000);
	let expected_contract_balance = U128(0);
	let account_balance = accounts[0].get_balance(&mut runtime, accounts[0].get_account_id());
	let contract_balance = accounts[0].get_balance(&mut runtime, flux_protocol());

	assert_eq!(account_balance, expected_account_balance);
	assert_eq!(contract_balance, expected_contract_balance);

}
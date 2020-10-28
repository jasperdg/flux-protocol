use super::*;

#[test]
fn test_place_order_insufficient_funds() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	
	accounts[1].set_allowance(&mut runtime, flux_protocol(), U128(5000)).expect("allowance couldn't be set");

	let account_1_res = accounts[1].place_order(&mut runtime, U64(0), 0, U128(50000), U128(50), None, None);
	assert_eq!(account_1_res.is_err(), true);
}

#[test]
fn test_order_placement_cancelation_and_market_prices() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	accounts[0].transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(2000000)).expect("allowance couldn't be set");

	let tx_res = accounts[0].place_order(&mut runtime, U64(0), 1, U128(1000), U128(50), None, None).expect("tx unexpectedly failed");
	let tx_res = accounts[0].place_order(&mut runtime, U64(0), 1, U128(1000), U128(50), None, None).expect("tx unexpectedly failed");
	
	let no_market_price = accounts[0].get_market_price(&mut runtime, U64(0), 0);
	assert_eq!(no_market_price, U128(50));
	
	accounts[0].place_order(&mut runtime, U64(0), 1, U128(1000), U128(60), None, None).expect("tx unexpectedly failed");

	let no_market_price = accounts[0].get_market_price(&mut runtime, U64(0), 0);
	assert_eq!(no_market_price, U128(40));

	accounts[0].cancel_order(&mut runtime, U64(0), 1, U128(60), U128(2), None).expect("order cancelation failed");

	// balance checks: 
	let expected_contract_balance = 100000;
	let expected_account_balance = 99999969999999999999900000;
	let account_balance: u128 = accounts[0].get_balance(&mut runtime, accounts[0].get_account_id()).into();
	let contract_balance: u128 = accounts[0].get_balance(&mut runtime, flux_protocol()).into();
	
	let validity_bond = to_dai(25) / 100;
	assert_eq!(contract_balance, expected_contract_balance + validity_bond);
	assert_eq!(account_balance, expected_account_balance - validity_bond);

	let no_market_price = accounts[0].get_market_price(&mut runtime, U64(0), 0);
	assert_eq!(no_market_price, U128(50));

	let tx_res = accounts[0].cancel_order(&mut runtime, U64(0), 1, U128(50), U128(1), None).expect("order cancelation failed");
	let tx_res = accounts[0].cancel_order(&mut runtime, U64(0), 1, U128(50), U128(0), None).expect("order cancelation failed");

	let expected_account_balance = to_dai(99999970) - validity_bond;
	let expected_contract_balance = 0;
	let account_balance: u128 = accounts[0].get_balance(&mut runtime, accounts[0].get_account_id()).into();
	let contract_balance: u128 = accounts[0].get_balance(&mut runtime, flux_protocol()).into();

	assert_eq!(account_balance, expected_account_balance);
	assert_eq!(contract_balance, validity_bond);

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	let claimable = accounts[0].get_claimable(&mut runtime, U64(0), accounts[0].get_account_id());

	assert_eq!(claimable, U128(0));

}
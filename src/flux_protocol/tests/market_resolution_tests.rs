use super::*;

#[test]
fn test_invalid_market_payout_calc() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.place_order(&mut runtime, U64(0), 0, U128(1000), 70, None, None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(1000), 10, None, None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 2, U128(1000), 10, None, None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 3, U128(1000), 10, None, None).expect("order placement failed unexpectedly");
	
	carol.place_order(&mut runtime, U64(0), 0, U128(1000), 60, None, None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), 1, U128(1000), 20, None, None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), 2, U128(1000), 20, None, None).expect("order placement failed unexpectedly");
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let initially_claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();
	assert_eq!(initially_claimable_alice, 100000 - 1000);
	assert_eq!(initially_claimable_carol, 100000 - 1000);
}

#[test]
fn test_valid_market_payout_calc() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.place_order(&mut runtime, U64(0), 0, U128(1000), 70, None, None).expect("order placement failed unexpectedly");

	carol.place_order(&mut runtime, U64(0), 1, U128(1000), 10, None, None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), 2, U128(1000), 20, None, None).expect("order placement failed unexpectedly");
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();

	let validity_bond = to_dai(25) / 100;
	assert_eq!(claimable_alice, validity_bond);
	assert_eq!(claimable_carol, 100000 - 1000);
}

use super::*;


// #[test]
// #[should_panic(expected = "transfer failed")]
// fn test_unable_to_create_market_without_allowance() {
// 	let (mut runtime, root, accounts) = init_runtime_env();
// 	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
// }

#[test]
fn test_validity_market_payout_calc_valid_market() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let alice = &accounts[0];

	alice.transfer(&mut runtime, root.get_account_id(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	let tx_res = alice.create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let validity_bond = to_dai(25) / 100;
	let balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	assert_eq!(balance, to_dai(99999970) - validity_bond);
	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, validity_bond);
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(U64(0)), U128(to_dai(5))).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(U64(0))).expect("market finalization failed unexpectedly"); 

	// let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	// assert_eq!(claimable_alice, validity_bond);

	let tx_res = alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id()).expect("claim earnings failed");
	println!("{:?}", tx_res);

	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, to_dai(5));

	let balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	assert_eq!(balance, to_dai(99999970));
}


#[test]
fn test_validity_market_payout_calc_invalid_market() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let alice = &accounts[0];

	alice.transfer(&mut runtime, root.get_account_id(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let validity_bond = to_dai(25) / 100;
	let balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	assert_eq!(balance, to_dai(99999970) - validity_bond);
	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, validity_bond);
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market finalization failed unexpectedly"); 

	let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	assert_eq!(claimable_alice, 0);

	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, to_dai(5) + validity_bond);

	let balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	assert_eq!(balance, to_dai(99999970) - validity_bond);
}

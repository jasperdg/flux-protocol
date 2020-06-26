use super::*;

#[test]
fn test_payout() {
	let (mut runtime, root, accounts) = init_runtime_env();
	runtime.current_block().block_timestamp = current_block_timestamp();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), ntoy(10).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(10))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(10))).expect("allowance couldn't be set");
	
	carol.place_order(&mut runtime, U64(0), U64(0), U128(10000), U128(70), None).expect("tx failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), U64(3), U128(1000), U128(10), None).expect("tx failed unexpectedly");
	
	alice.place_order(&mut runtime, U64(0), U64(1), U128(1000), U128(10), None).expect("tx failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(2), U128(1000), U128(10), None).expect("tx failed unexpectedly");

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into();
	
	println!("alice  {:?}", initial_balance_alice);
	println!("carol  {:?}", initial_balance_carol);

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	println!("tx res resolution: {:?}", tx_res);
	
	let tx_res = carol.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("tx failed unexpectedly");
	let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let initially_claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();
	
	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into();
	
	// skip to after dispute window closed
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;

	alice.finalize_market(&mut runtime, U64(0), None).expect("market finalization failed unexpectedly");
	
	let tx_res = carol.claim_earnings(&mut runtime, U64(0), carol.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	println!("carol: {:?}", tx_res);
	let tx_res = alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	println!("alice: {:?}", tx_res);

	let updated_claimable_alice = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id());
	let updated_claimable_carol = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id());

	let updated_balance_alice = alice.get_balance(&mut runtime, alice.get_account_id());
	let updated_balance_carol = alice.get_balance(&mut runtime, carol.get_account_id());

	println!("alice {:?}  {:?}  {:?}", updated_balance_alice, initially_claimable_alice, initial_balance_alice);
	println!("carol {:?}  {:?}  {:?}", updated_balance_carol, initially_claimable_carol, initial_balance_carol);

	assert_eq!(updated_balance_alice, U128(initially_claimable_alice + initial_balance_alice));
	assert_eq!(updated_balance_carol, U128(initially_claimable_carol + initial_balance_carol));

	assert_eq!(updated_claimable_alice, U128(0));
	assert_eq!(updated_claimable_carol, U128(0));
}

use super::*;

#[test]
fn test_payout() {
	let (mut runtime, root, accounts) = init_runtime_env();
	runtime.current_block().block_timestamp = current_block_timestamp();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(10).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(10))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(10))).expect("allowance couldn't be set");
	
	carol.place_order(&mut runtime, U64(0), U64(0), U128(1428), U128(70), None).expect("tx failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), U64(3), U128(10000), U128(10), None).expect("tx failed unexpectedly");
	
	alice.place_order(&mut runtime, U64(0), U64(1), U128(10000), U128(10), None).expect("tx failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(2), U128(10000), U128(10), None).expect("tx failed unexpectedly");

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into();

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	let tx_res = carol.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("tx failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	carol.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let initially_claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();
	
	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into();

	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	let tx_res = alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let tx_res = carol.claim_earnings(&mut runtime, U64(0), carol.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	
	let updated_balance_alice = alice.get_balance(&mut runtime, alice.get_account_id());
	let updated_balance_carol = alice.get_balance(&mut runtime, carol.get_account_id());

	assert_eq!(updated_balance_alice, U128(initially_claimable_alice + initial_balance_alice));
	assert_eq!(updated_balance_carol, U128(initially_claimable_carol + initial_balance_carol));

}

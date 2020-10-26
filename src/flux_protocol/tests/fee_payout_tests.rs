use super::*;

#[test]
#[should_panic(expected = "affiliate claim failed unexpectedly")]
fn fee_distribution_test() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let alice = &accounts[0];
	let carol = &accounts[1];
	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = carol.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 400, 50, "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	alice.place_order(&mut runtime, U64(0), 0, U128(to_dai(5) / 50), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");

	let filled_volume: u128 = alice.get_market_volume(&mut runtime, U64(0)).into();
	assert_eq!(filled_volume, to_dai(10));
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	let resolution_fee_percentage = 1;

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into(); // trader
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	let initial_balance_root: u128 = alice.get_balance(&mut runtime, root.get_account_id()).into(); // resolutor
	
	let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let expected_claimable_alice_excl_fees = to_dai(30);
	let claimable_root: u128 = alice.get_claimable(&mut runtime, U64(0), root.get_account_id()).into();
	let market_creator_fee = 4 * to_dai(10) / 100;
	let resolution_fee = 1 * to_dai(10) / 100;

	
	assert_eq!(claimable_alice, expected_claimable_alice_excl_fees - market_creator_fee - resolution_fee);
	assert_eq!(claimable_root, 1 * to_dai(10) / 100 + to_dai(5));
	
	let tx_res = alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let tx_res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let after_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into(); // trader
	let after_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	let after_balance_root: u128 = alice.get_balance(&mut runtime, root.get_account_id()).into(); // resolutor
	
	assert_eq!(after_balance_alice, initial_balance_alice + expected_claimable_alice_excl_fees - market_creator_fee - resolution_fee);
	assert_eq!(after_balance_carol, initial_balance_carol + market_creator_fee);
	assert_eq!(after_balance_root, initial_balance_root + resolution_fee + to_dai(5));
	
	let tx_res = alice.claim_affiliate_earnings(&mut runtime, carol.get_account_id()).expect("affiliate claim failed unexpectedly");
	let after_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	assert_eq!(after_balance_carol, initial_balance_carol + 4 * to_dai(10) / 100);
	
	let tx_res = alice.claim_affiliate_earnings(&mut runtime, carol.get_account_id()).expect("affiliate claim failed unexpectedly"); // should fail

	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}
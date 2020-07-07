use super::*;

#[test]
#[should_panic(expected = "affiliate claim failed unexpectedly")]
fn fee_distribution_test() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[1].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(400), U128(50), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];


	alice.transfer(&mut runtime, root.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.place_order(&mut runtime, U64(0), U64(0), U128(to_dai(5)), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(1), U128(to_dai(5)), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	
	alice.place_order(&mut runtime, U64(0), U64(1), U128(to_dai(5)), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(1), U128(to_dai(5)), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(1), U128(to_dai(5)), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(1), U128(to_dai(5)), U128(50), Some(carol.get_account_id())).expect("order placement failed unexpectedly");

	let filled_volume: u128 = alice.get_market_volume(&mut runtime, U64(0)).into();
	assert_eq!(filled_volume, to_dai(10));
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(U64(1)), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), Some(U64(1))).expect("market resolution failed unexpectedly");

	let resolution_fee_percentage = 1;

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into(); // trader
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	let initial_balance_root: u128 = alice.get_balance(&mut runtime, root.get_account_id()).into(); // resolutor

	let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let expected_claimable_alice_excl_fees = to_dai(30);
	let claimable_root: u128 = alice.get_claimable(&mut runtime, U64(0), root.get_account_id()).into();
	let fees = 5 * to_dai(10) / 100;

	assert_eq!(claimable_alice, expected_claimable_alice_excl_fees - fees);
	assert_eq!(claimable_root, 1 * to_dai(10) / 100 + to_dai(5));
 
	let tx_res = alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let tx_res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let after_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into(); // trader
	let after_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	let after_balance_root: u128 = alice.get_balance(&mut runtime, root.get_account_id()).into(); // resolutor
	
	assert_eq!(after_balance_alice, initial_balance_alice + expected_claimable_alice_excl_fees - fees);
	assert_eq!(after_balance_carol, initial_balance_carol + 2 * to_dai(10) / 100);
	assert_eq!(after_balance_root, initial_balance_root + 1 * to_dai(10) / 100 + to_dai(5));
	
	let tx_res = alice.claim_affiliate_earnings(&mut runtime, carol.get_account_id()).expect("affiliate claim failed unexpectedly");
	println!("re: {:?}", tx_res);
	let after_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	assert_eq!(after_balance_carol, initial_balance_carol + 4 * to_dai(10) / 100);
	
	let tx_res = alice.claim_affiliate_earnings(&mut runtime, carol.get_account_id()).expect("affiliate claim failed unexpectedly"); // should fail
}

// TODO: test coverage for crowdsourced pariticipation on winning outcome that isn;t bonded - this shouldnt be added to the users pariticpation
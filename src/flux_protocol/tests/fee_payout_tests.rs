use super::*;

#[test]
fn fee_distribution_test() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let alice = &accounts[0];
	let carol = &accounts[1];
	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = carol.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 400, 50, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	alice.place_order(&mut runtime, U64(0), 0, U128(to_dai(5) / 50), 50, Some(carol.get_account_id()), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, Some(carol.get_account_id()), None).expect("order placement failed unexpectedly");
	
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, Some(carol.get_account_id()), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, Some(carol.get_account_id()), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, Some(carol.get_account_id()), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, Some(carol.get_account_id()), None).expect("order placement failed unexpectedly");

	let filled_volume: u128 = alice.get_market_volume(&runtime, U64(0)).into();
	assert_eq!(filled_volume, to_dai(10));
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");


	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into(); // trader
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	let initial_balance_root: u128 = alice.get_balance(&mut runtime, root.get_account_id()).into(); // resolutor
	
	let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let expected_claimable_alice_excl_fees = to_dai(30);
	let claimable_root: u128 = alice.get_claimable(&mut runtime, U64(0), root.get_account_id()).into();
	let market_creator_fee = 4 * to_dai(10) / 100;
	let resolution_fee = to_dai(10) / 100;

	assert_eq!(claimable_alice, expected_claimable_alice_excl_fees - market_creator_fee - resolution_fee);
	assert_eq!(claimable_root, to_dai(10) / 100 + to_dai(5));
	
	alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	
	let after_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into(); // trader
	let after_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	let after_balance_root: u128 = alice.get_balance(&mut runtime, root.get_account_id()).into(); // resolutor
	
	assert_eq!(after_balance_alice, initial_balance_alice + expected_claimable_alice_excl_fees - market_creator_fee - resolution_fee);
	assert_eq!(after_balance_carol, initial_balance_carol + market_creator_fee);
	assert_eq!(after_balance_root, initial_balance_root + resolution_fee + to_dai(5));
	
	let after_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into(); // creator / affiliate
	assert_eq!(after_balance_carol, initial_balance_carol + 4 * to_dai(10) / 100);
	carol.claim_earnings(&mut runtime, U64(0), carol.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn valid_market_fee_distribution_with_sales_test() {
	let (mut runtime, resolver, accounts) = init_runtime_env();
	let trader = &accounts[0];
	let creator = &accounts[1];
	trader.transfer(&mut runtime, creator.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	trader.transfer(&mut runtime, resolver.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	resolver.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30000))).expect("allowance couldn't be set");
	creator.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30000))).expect("allowance couldn't be set");
	trader.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30000))).expect("allowance couldn't be set");
	
	let tx_res = creator.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 100, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let tx_1 = trader.place_order(&mut runtime, U64(0), 0, U128(to_dai(5)), 50, None, None).expect("order placement failed unexpectedly");
	let tx_2 = trader.place_order(&mut runtime, U64(0), 1, U128(to_dai(5)), 50, None, None).expect("order placement failed unexpectedly");
	let tx_3 = trader.place_order(&mut runtime, U64(0), 1, U128(to_dai(5)), 50, None, None).expect("order placement failed unexpectedly");
	let pre_claim_balance_creator: u128 = creator.get_balance(&mut runtime, creator.get_account_id()).into();

	trader.dynamic_market_sell(&mut runtime, U64(0), 1, U128(to_dai(5)), 50, None).expect("order placement failed unexpectedly");
	
	let filled_volume: u128 = trader.get_market_volume(&runtime, U64(0)).into();
	assert_eq!(filled_volume, to_dai(15) * 50);
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	resolver.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	resolver.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	let pre_claim_balance_resolver: u128 = creator.get_balance(&mut runtime, resolver.get_account_id()).into();

	creator.claim_earnings(&mut runtime, U64(0), creator.get_account_id(), None).expect("tx failed");
	resolver.claim_earnings(&mut runtime, U64(0), resolver.get_account_id(), None).expect("tx failed");

	let tx = trader.claim_earnings(&mut runtime, U64(0), trader.get_account_id(), None).expect("tx failed");

	let post_claim_balance_creator: u128 = creator.get_balance(&mut runtime, creator.get_account_id()).into();
	let post_claim_balance_resolver: u128 = creator.get_balance(&mut runtime, resolver.get_account_id()).into();

	let expected_creator_fee_earnings = to_dai(10) * 50 / 100;
	let expected_resolution_fee_earnings = filled_volume / 100;
	let expected_balance_addition_creator = to_dai(25) / 100 + expected_creator_fee_earnings;
	let expected_balance_addition_resolver = to_dai(5) + expected_resolution_fee_earnings;

	assert_eq!(post_claim_balance_creator, pre_claim_balance_creator + expected_balance_addition_creator);
	assert_eq!(post_claim_balance_resolver, pre_claim_balance_resolver + expected_balance_addition_resolver);

	let contract_balance: u128 = trader.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn invalid_market_fee_distribution_with_sales_test() {
	let (mut runtime, resolver, accounts) = init_runtime_env();
	let trader = &accounts[0];
	let creator = &accounts[1];
	trader.transfer(&mut runtime, creator.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	trader.transfer(&mut runtime, resolver.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	resolver.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30000))).expect("allowance couldn't be set");
	creator.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30000))).expect("allowance couldn't be set");
	trader.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30000))).expect("allowance couldn't be set");
	
	let tx_res = creator.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 100, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let tx_1 = trader.place_order(&mut runtime, U64(0), 0, U128(to_dai(5)), 50, None, None).expect("order placement failed unexpectedly");
	let tx_2 = trader.place_order(&mut runtime, U64(0), 1, U128(to_dai(5)), 50, None, None).expect("order placement failed unexpectedly");
	let tx_3 = trader.place_order(&mut runtime, U64(0), 1, U128(to_dai(5)), 50, None, None).expect("order placement failed unexpectedly");
	let pre_claim_balance_creator: u128 = creator.get_balance(&mut runtime, creator.get_account_id()).into();

	trader.dynamic_market_sell(&mut runtime, U64(0), 1, U128(to_dai(5)), 50, None).expect("order placement failed unexpectedly");
	
	let filled_volume: u128 = trader.get_market_volume(&runtime, U64(0)).into();
	assert_eq!(filled_volume, to_dai(15) * 50);
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	resolver.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	resolver.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let pre_claim_balance_resolver: u128 = creator.get_balance(&mut runtime, resolver.get_account_id()).into();

	resolver.claim_earnings(&mut runtime, U64(0), resolver.get_account_id(), None).expect("tx failed");

	let tx = trader.claim_earnings(&mut runtime, U64(0), trader.get_account_id(), None).expect("tx failed");

	let post_claim_balance_resolver: u128 = creator.get_balance(&mut runtime, resolver.get_account_id()).into();
	let post_claim_balance_creator: u128 = creator.get_balance(&mut runtime, creator.get_account_id()).into();

	let expected_resolution_fee_earnings = filled_volume / 100;
	let expected_balance_addition_resolver = to_dai(5) + expected_resolution_fee_earnings;

	// assert_eq!(post_claim_balance_creator, pre_claim_balance_creator + expected_balance_addition_creator);
	assert_eq!(post_claim_balance_resolver, pre_claim_balance_resolver + expected_balance_addition_resolver);

	let contract_balance: u128 = trader.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, to_dai(1) / 4);
	assert_eq!(pre_claim_balance_creator, post_claim_balance_creator);
}

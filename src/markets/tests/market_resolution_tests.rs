use super::*;

#[test]
fn test_invalid_market_payout_calc() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.place_order(&mut runtime, U64(0), U64(0), U128(7000), U128(70), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(1), U128(1000), U128(10), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(2), U128(1000), U128(10), None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), U64(3), U128(1000), U128(10), None).expect("order placement failed unexpectedly");
	
	carol.place_order(&mut runtime, U64(0), U64(0), U128(6000), U128(60), None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), U64(1), U128(2000), U128(20), None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), U64(2), U128(2000), U128(20), None).expect("order placement failed unexpectedly");
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	// let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	// let initially_claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();
	// assert_eq!(initially_claimable_alice, 10000 - 100);
	// assert_eq!(initially_claimable_carol, 10000 - 100);

	let open_orders_0: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(0)).into();
	let open_orders_1: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(1)).into();
	let open_orders_2: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(2)).into();
	let open_orders_3: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(3)).into();

	assert_eq!(open_orders_0, 0);
	assert_eq!(open_orders_1, 0);
	assert_eq!(open_orders_2, 0);
	assert_eq!(open_orders_3, 0);

	let filled_orders_0: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(0)).into();
	let filled_orders_1: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(1)).into();
	let filled_orders_2: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(2)).into();
	let filled_orders_3: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(3)).into();

	assert_eq!(filled_orders_0, 2);
	assert_eq!(filled_orders_1, 2);
	assert_eq!(filled_orders_2, 2);
	assert_eq!(filled_orders_3, 1);
}

#[test]
fn test_valid_market_payout_calc() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.place_order(&mut runtime, U64(0), U64(0), U128(7000), U128(70), None).expect("order placement failed unexpectedly");
	
	carol.place_order(&mut runtime, U64(0), U64(1), U128(1000), U128(10), None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), U64(2), U128(2000), U128(20), None).expect("order placement failed unexpectedly");
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(U64(1)), U128(to_dai(5))).expect("market resolution failed unexpectedly");

	let open_orders_0: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(0)).into();
	let open_orders_1: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(1)).into();
	let open_orders_2: u128 = alice.get_open_orders_len(&mut runtime, U64(0), U64(2)).into();

	assert_eq!(open_orders_0, 0);
	assert_eq!(open_orders_1, 0);
	assert_eq!(open_orders_2, 0);

	let filled_orders_0: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(0)).into();
	let filled_orders_1: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(1)).into();
	let filled_orders_2: u128 = alice.get_filled_orders_len(&mut runtime, U64(0), U64(2)).into();

	assert_eq!(filled_orders_0, 1);
	assert_eq!(filled_orders_1, 1);
	assert_eq!(filled_orders_2, 1);

	// let claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	// let claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();

	// assert_eq!(claimable_alice, 0);
	// assert_eq!(claimable_carol, 10000 - 100);
}

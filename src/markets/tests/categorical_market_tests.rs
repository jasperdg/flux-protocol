use super::*;
#[test]
fn test_categorical_market_automated_matcher() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(4), outcome_tags(4), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].transfer(&mut runtime, accounts[1].get_account_id(), ntoy(10).into()).expect("transfer failed couldn't be set");

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(110000)).expect("allowance couldn't be set");

	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(3000), U128(30), None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(6000), U128(60), None).expect("order placement tx failed unexpectedly");
	
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(2500), U128(25), None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(5000), U128(50), None).expect("order placement tx failed unexpectedly");

	// alice fills all orders
	accounts[1].set_allowance(&mut runtime, flux_protocol(), U128(110000)).expect("allowance couldn't be set");
	accounts[1].place_order(&mut runtime, U64(0), U64(2), U128(3500), U128(25), None).expect("order placement tx failed unexpectedly");

	let open_0_orders = accounts[0].get_open_orders_len(&mut runtime, U64(0), U64(0));
    let open_2_orders = accounts[0].get_open_orders_len(&mut runtime, U64(0), U64(1));
    let open_1_orders = accounts[0].get_open_orders_len(&mut runtime, U64(0), U64(2));
    let filled_0_orders = accounts[0].get_filled_orders_len(&mut runtime, U64(0), U64(0));
    let filled_1_orders = accounts[0].get_filled_orders_len(&mut runtime, U64(0), U64(1));
    let filled_2_orders = accounts[0].get_filled_orders_len(&mut runtime, U64(0), U64(2));

	// assertions for the orderbook lengths
	assert_eq!(open_0_orders, U128(0));
	assert_eq!(open_1_orders, U128(0));
	assert_eq!(open_2_orders, U128(0));
	assert_eq!(filled_0_orders, U128(2));
	assert_eq!(filled_1_orders, U128(2));
	assert_eq!(filled_2_orders, U128(1));
}

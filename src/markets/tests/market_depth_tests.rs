use super::*;

#[test]
fn test_liquidity_for_price() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(11000000)).expect("allowance couldn't be set");
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(60000), U128(50), None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(60000), U128(50), None).expect("order placement tx failed unexpectedly");
	
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(60000), U128(20), None).expect("order placement tx failed unexpectedly");
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(80000), U128(20), None).expect("order placement tx failed unexpectedly");

	let liquidity_60 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(60));
	let liquidity_50 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(50));
	let liquidity_20 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(20));

	assert_eq!(liquidity_60, U128(0));
	assert_eq!(liquidity_50, U128(120000 / 50));
	assert_eq!(liquidity_20, U128(140000 / 20));

	accounts[0].cancel_order(&mut runtime, U64(0), U64(0), U128(0)).expect("order cancelation failed");
	accounts[0].cancel_order(&mut runtime, U64(0), U64(0), U128(1)).expect("order cancelation failed");
	accounts[0].cancel_order(&mut runtime, U64(0), U64(0), U128(3)).expect("order cancelation failed");

	let liquidity_50 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(50));
	let liquidity_20 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(20));

	assert_eq!(liquidity_50, U128(0));
	assert_eq!(liquidity_20, U128(60000 / 20));

	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(80000), U128(80), None).expect("order placement tx failed unexpectedly");

	let liquidity_20 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(20));
	let liquidity_80 = accounts[0].get_liquidity(&mut runtime, U64(0), U64(0), U128(80));

	assert_eq!(liquidity_20, U128(40000 / 20));
	assert_eq!(liquidity_80, U128(0));
}

#[test]
fn test_valid_binary_market_depth() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(3), outcome_tags(3), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(1100000)).expect("allowance couldn't be set");
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(50000), U128(50), None).expect("order placement tx failed unexpectedly"); // 1000 shares
	accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(60000), U128(60), None).expect("order placement tx failed unexpectedly"); // 1000 shares
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(20000), U128(20), None).expect("order placement tx failed unexpectedly"); // 1000 shares
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(30000), U128(30), None).expect("order placement tx failed unexpectedly"); // 1000 shares


	let depth_0 = accounts[0].get_depth(&mut runtime, U64(0), U64(2), U128(1000000), U128(100));
	let depth_1 = accounts[0].get_depth(&mut runtime, U64(0), U64(1), U128(100000), U128(11));

    assert_eq!(depth_0, U128(40000));
	assert_eq!(depth_1, U128(0));
}


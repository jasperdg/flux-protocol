use super::*;

pub const SINGLE_CALL_GAS: u64 = 100000000000000;

#[test]
fn test_custom_gas_txs() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let alice = &accounts[0];

	alice.transfer(&mut runtime, root.get_account_id(), U128(to_dai(500))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(500))).expect("allowance couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(500))).expect("allowance couldn't be set");

	alice.create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();
	alice.place_order(&mut runtime, U64(0), 0, U128(to_shares(1)), 50, None, Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();    
    alice.cancel_order(&mut runtime, U64(0), 0, 50, U128(to_shares(0)), Some(vec![U64(SINGLE_CALL_GAS)])).unwrap();    
	alice.place_order(&mut runtime, U64(0), 0, U128(to_shares(1)), 50, None, Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();    
	alice.place_order(&mut runtime, U64(0), 0, U128(to_shares(1)), 50, None, Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();    
	alice.place_order(&mut runtime, U64(0), 1, U128(to_shares(1)), 50, None, Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();    
	alice.dynamic_market_sell(&mut runtime, U64(0), 0, U128(100), 40, Some(vec![U64(SINGLE_CALL_GAS)])).unwrap();    

    runtime.current_block().block_timestamp = market_end_timestamp_ns();

    alice.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();
    alice.dispute_market(&mut runtime, U64(0), Some(0), U128(to_dai(10)), Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id(), Some(vec![U64(SINGLE_CALL_GAS)])).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

}
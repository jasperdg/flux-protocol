use super::*;

pub const SINGLE_CALL_GAS: u64 = 100000000000000;

#[test]
fn test_custom_gas_txs() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let alice = &accounts[0];

	alice.transfer(&mut runtime, root.get_account_id(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();
	alice.place_order(&mut runtime, U64(0), 0, U128(100000000000), 99, None, Some(vec![U64(SINGLE_CALL_GAS), U64(SINGLE_CALL_GAS)])).unwrap();    

}
use super::*;

#[test]
fn simplest_binary_order_matching_test() {
	
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let account_0_res = accounts[0].place_order(&mut runtime, U64(0), U64(0), U128(100000), U128(50), None).unwrap();

	// should fail - no balance
	let account_1_res = accounts[1].place_order(&mut runtime, U64(0), U64(0), U128(100000), U128(50), None);

	match account_1_res {
		Ok(_) => panic!("tx shouldn't process because of lack of balance"),
		Err(_) => println!("tx failed as it should")
	}

	// TODO: Do we still need mint fucntionality or should we jsut transfer from owner
	// contract.claim_fdai();

	// contract.place_order(0, 0, 5000, 50, None);
	// contract.place_order(0, 1, 5000, 50, None);

	// let open_no_orders = contract.get_open_orders(0, 0);
	// let open_yes_orders = contract.get_open_orders(0, 1);
	// assert_eq!(open_no_orders.len(), 0);
	// assert_eq!(open_yes_orders.len(), 0);
	// let filled_no_orders = contract.get_filled_orders(0, 0);
	// let filled_yes_orders = contract.get_filled_orders(0, 1);
	// assert_eq!(filled_no_orders.len(), 1);
	// assert_eq!(filled_yes_orders.len(), 1);
}

// fn partial_binary_order_matching_test() {
// 	testing_env!(get_context(carol(), current_block_timestamp()));
// 	let mut contract = Markets::default();
// 	contract.claim_fdai();
// 	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

// 	contract.place_order(0, 0, 5000, 50, None);
// 	contract.place_order(0, 1, 5000, 50, None);

// 	contract.place_order(0, 1, 5000, 50, None);
// 	contract.place_order(0, 1, 2750, 50, None);
// 	contract.place_order(0, 0, 7777, 50, None);

// 	let open_no_orders = contract.get_open_orders(0, 0);
// 	let open_yes_orders = contract.get_open_orders(0, 1);
// 	assert_eq!(open_no_orders.len(), 0);
// 	assert_eq!(open_yes_orders.len(), 0);
// 	let filled_no_orders = contract.get_filled_orders(0, 0);
// 	let filled_yes_orders = contract.get_filled_orders(0, 1);
// 	assert_eq!(filled_no_orders.len(), 1);
// 	assert_eq!(filled_yes_orders.len(), 2);
// }
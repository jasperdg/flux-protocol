use super::*;

// #[test]
// fn test_place_order_insufficient_funds() {
// 	let (mut runtime, root, accounts) = init_runtime_env();

// 	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
// 	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	
// 	accounts[1].set_allowance(&mut runtime, flux_protocol(), U128(5000)).expect("allowance couldn't be set");

// 	let account_1_res = accounts[1].place_order(&mut runtime, U64(0), U64(0), U128(5000), U128(50), None);
// 	match account_1_res {
// 		Ok(_) => panic!("tx shouldn't process because of lack of balance"),
// 		Err(_) => println!("tx failed as it should")
// 	}	
// }

#[test]
fn test_market_orders() {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(15000)).expect("allowance couldn't be set");

	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(5000), U128(50), None).expect("tx unexpectedly failed");
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(5000), U128(50), None).expect("tx unexpectedly failed");

	let no_market_price = accounts[0].get_market_price(&mut runtime, 0, 0);
	assert_eq!(no_market_price, 50);
	
	accounts[0].place_order(&mut runtime, U64(0), U64(1), U128(5000), U128(60), None).expect("tx unexpectedly failed");
	
	let no_market_price = accounts[0].get_market_price(&mut runtime, 0, 0);
	assert_eq!(no_market_price, 40);

	accounts[0].cancel_order(&mut runtime, U64(0), U64(1), U128(2)).expect("order cancelation failed");
// 	// yes_market_price = contract.get_market_price(0, 0);
// 	// assert_eq!(yes_market_price, 50);

// 	// contract.cancel_order(0, 1, 1);
// 	// yes_market_price = contract.get_market_price(0, 0);
// 	// assert_eq!(yes_market_price, 50);

// 	// contract.cancel_order(0, 1, 0);
// 	// yes_market_price = contract.get_market_price(0, 0);
// 	// assert_eq!(yes_market_price, 100);

}

// // // list of test cases to include
// // // Try calling proceed order fill
// // // try calling place order with insufficient balance

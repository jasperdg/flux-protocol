use super::*;
#[test]
fn test_categorical_market_automated_matcher() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 3, outcome_tags(3), categories(),  market_end_timestamp_ms(), 0, 0, "test".to_string());

	// best prices - market price = 10
	contract.place_order("bid", 0, 0, 3000, 30, None);
	contract.place_order("bid", 0, 1, 6000, 60, None);

	// worse prices - market price = 25
	contract.place_order("bid", 0, 0, 2500, 25, None);
	contract.place_order("bid", 0, 1, 5000, 50, None);

	testing_env!(get_context(alice(), current_block_timestamp()));

	contract.claim_fdai();

	// alice fills all orders
	contract.place_order("bid", 0, 2, 3500, 25, None);

	let open_0_orders = contract.get_open_orders(0, 0);
    let open_1_orders = contract.get_open_orders(0, 1);
    let open_2_orders = contract.get_open_orders(0, 2);
    let filled_0_orders = contract.get_filled_orders(0, 0);
    let filled_1_orders = contract.get_filled_orders(0, 1);
	let filled_2_orders = contract.get_filled_orders(0, 2);

	//// uncomment for orderbook state check
	// println!("open orders outcome 0: {:?}", open_0_orders);
	// println!("____________________________________________________");
	// println!("open orders outcome 1: {:?}", open_1_orders);
	// println!("____________________________________________________");
	// println!("open orders outcome 2: {:?}", open_2_orders);
	// println!("____________________________________________________");
	// println!("filled orders outcome 0: {:?}", filled_0_orders);
	// println!("____________________________________________________");
	// println!("filled orders outcome 1: {:?}", filled_1_orders);
	// println!("____________________________________________________");
	// println!("filled orders outcome 2: {:?}", filled_2_orders);
	// println!("____________________________________________________");

	// assertions for the orderbook lengths
	assert_eq!(open_0_orders.len(), 0);
	assert_eq!(open_1_orders.len(), 0);
	assert_eq!(open_2_orders.len(), 0);
	assert_eq!(filled_0_orders.len(), 2);
	assert_eq!(filled_1_orders.len(), 2);
	assert_eq!(filled_2_orders.len(), 1);
}

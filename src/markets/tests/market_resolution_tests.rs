use super::*;

#[test]
fn test_invalid_market_payout_calc() {
	testing_env!(get_context(carol()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 4, outcome_tags(4), categories(), 100010101001010);

	let rounded_10k_at_70 = (10000 / 70) * 70;
	let amount_filled_after_cancel = 10000 - 2840;
	let expected_carol_balance = (2 * rounded_10k_at_70 + 3 * 10000) - amount_filled_after_cancel;
	contract.place_order(0, 0, 10000, 70);
	contract.place_order(0, 0, 10000, 70);
	contract.place_order(0, 1, 10000, 10); 
	contract.place_order(0, 2, 10000, 10); 
	contract.place_order(0, 3, 10000, 10);
	
	testing_env!(get_context(alice()));
	contract.claim_fdai();

	let rounded_10k_at_60 = (10000 / 60) * 60;
	contract.place_order(0, 0, 10000, 60);
	contract.place_order(0, 1, 10000, 20); 
	contract.place_order(0, 1, 10000, 20); 
	
	let expected_alice_balance = (1 * rounded_10k_at_60 + 2 * 10000) - (10000 - 3320);
	contract.cancel_order(0,1, 1);

	testing_env!(get_context(carol()));
	contract.cancel_order(0,1, 0);
	contract.resolute(0, None);
	let claimable_carol = contract.get_claimable(0, carol());
	let claimable_alice = contract.get_claimable(0, alice());
	assert_eq!(claimable_carol, expected_carol_balance);
	assert_eq!(claimable_alice, expected_alice_balance);

	// Orderbook length assertions
	let open_0_orders = contract.get_open_orders(0, 0, carol());
	let open_1_orders = contract.get_open_orders(0, 1, carol());
	let open_2_orders = contract.get_open_orders(0, 2, carol());
	let filled_0_orders = contract.get_filled_orders(0, 0, carol());
	let filled_1_orders = contract.get_filled_orders(0, 1, carol());
	let filled_2_orders = contract.get_filled_orders(0, 2, carol());

	// // debugging logs
	// println!("open 0 {:?}", open_0_orders);
	// println!("open 1 {:?}", open_1_orders);
	// println!("open 2 {:?}", open_2_orders);

	// println!("filled 0 {:?}", filled_0_orders);
	// println!("filled 1 {:?}", filled_1_orders);
	// println!("filled 2 {:?}", filled_2_orders);

	assert_eq!(open_0_orders.len(), 0);
	assert_eq!(filled_0_orders.len(), 3);

	assert_eq!(open_1_orders.len(), 1);
	assert_eq!(filled_1_orders.len(), 2);

	assert_eq!(open_2_orders.len(), 1);
	assert_eq!(filled_2_orders.len(), 0);
	
}

#[test]
fn test_valid_market_payout_calc() {
	testing_env!(get_context(carol()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 4, outcome_tags(4), categories(), 100010101001010);

	contract.place_order(0, 0, 30000, 70);
	
	testing_env!(get_context(alice()));
	contract.claim_fdai();
	contract.place_order(0, 1, 1000, 10); 
	contract.place_order(0, 2, 2000, 20); 


	testing_env!(get_context(carol()));
	contract.resolute(0, Some(1));

	let open_0_orders = contract.get_open_orders(0, 0, carol());
	let open_1_orders = contract.get_open_orders(0, 1, carol());
	let open_2_orders = contract.get_open_orders(0, 2, carol());
	let filled_0_orders = contract.get_filled_orders(0, 0, carol());
	let filled_1_orders = contract.get_filled_orders(0, 1, carol());
	let filled_2_orders = contract.get_filled_orders(0, 2, carol());

	// // uncomment for orderbook state check
	//println!("open {:?}", 	open_0_orders);
	//println!("open {:?}", 	open_1_orders);
	//println!("open {:?}", 	open_2_orders);

	//println!("filled {:?}", filled_0_orders);
	//println!("filled {:?}", filled_1_orders);
	//println!("filled {:?}", filled_2_orders);

	let claimable_carol = contract.get_claimable(0, carol());
	let claimable_alice = contract.get_claimable(0, alice());
}
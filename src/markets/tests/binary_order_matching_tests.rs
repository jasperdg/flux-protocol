use super::*;
// #[test]
// fn simplest_binary_order_matching_test() {
// 	testing_env!(get_context(carol(), current_block_timestamp()));
// 	let mut contract = Markets::default();
// 	contract.claim_fdai();
// 	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

// 	contract.place_order(0, 0, 5000, 50);
// 	contract.place_order(0, 1, 5000, 50);

// 	let open_no_orders = contract.get_open_orders(0, 0);
// 	let open_yes_orders = contract.get_open_orders(0, 1);
// 	assert_eq!(open_no_orders.len(), 0);
// 	assert_eq!(open_yes_orders.len(), 0);
// 	let filled_no_orders = contract.get_filled_orders(0, 0);
// 	let filled_yes_orders = contract.get_filled_orders(0, 1);
// 	assert_eq!(filled_no_orders.len(), 1);
// 	assert_eq!(filled_yes_orders.len(), 1);
// }

// fn partial_binary_order_matching_test() {
// 	testing_env!(get_context(carol(), current_block_timestamp()));
// 	let mut contract = Markets::default();
// 	contract.claim_fdai();
// 	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

// 	contract.place_order(0, 0, 5000, 50);
// 	contract.place_order(0, 1, 5000, 50);

// 	contract.place_order(0, 1, 5000, 50);
// 	contract.place_order(0, 1, 2750, 50);
// 	contract.place_order(0, 0, 7777, 50);

// 	let open_no_orders = contract.get_open_orders(0, 0);
// 	let open_yes_orders = contract.get_open_orders(0, 1);
// 	assert_eq!(open_no_orders.len(), 0);
// 	assert_eq!(open_yes_orders.len(), 0);
// 	let filled_no_orders = contract.get_filled_orders(0, 0);
// 	let filled_yes_orders = contract.get_filled_orders(0, 1);
// 	assert_eq!(filled_no_orders.len(), 1);
// 	assert_eq!(filled_yes_orders.len(), 2);
// }

#[test]
fn simple_binary_order_sale() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

	contract.place_order(0, 0, 10000, 50, None);
	contract.place_order(0, 1, 5000, 50, None);
	contract.place_order(0, 1, 5000, 50, None);
	contract.place_order(0, 1, 4900, 49, None);
	contract.place_order(0, 1, 2500, 50, None);

	let share_balance = contract.get_outcome_share_balance(0, 1, carol());
	assert_eq!(200, share_balance);
	let (sell_depth, shares_fillable) = contract.get_market_sell_depth(0, 1, 10000);
	assert_eq!(150, shares_fillable);
	assert_eq!(7400, sell_depth);

	contract.dynamic_market_sell(0, 1, share_balance - 5);
	
	// let (sell_depth, shares_fillable) = contract.get_market_sell_depth(0, 1, 10000);
	// assert_eq!(0, shares_fillable);
	// assert_eq!(0, sell_depth);
	
	// let share_balance = contract.get_outcome_share_balance(0, 1, carol());
	// assert_eq!(89, share_balance);
}

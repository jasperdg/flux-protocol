use super::*;

fn init_with_buyer_and_seller() -> (String, String, Markets) {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());
	testing_env!(get_context(alice(), current_block_timestamp()));
	contract.claim_fdai();

	return(alice(), carol(), contract)
}
// #[test]
fn simplest_binary_order_matching_test() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

	contract.place_order(0, 0, 5000, 50);
	contract.place_order(0, 1, 5000, 50);

	let open_no_orders = contract.get_open_orders(0, 0);
	let open_yes_orders = contract.get_open_orders(0, 1);
	assert_eq!(open_no_orders.len(), 0);
	assert_eq!(open_yes_orders.len(), 0);
	let filled_no_orders = contract.get_filled_orders(0, 0);
	let filled_yes_orders = contract.get_filled_orders(0, 1);
	assert_eq!(filled_no_orders.len(), 1);
	assert_eq!(filled_yes_orders.len(), 1);
}

fn partial_binary_order_matching_test() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

	contract.place_order(0, 0, 5000, 50);
	contract.place_order(0, 1, 5000, 50);

	contract.place_order(0, 1, 5000, 50);
	contract.place_order(0, 1, 2750, 50);
	contract.place_order(0, 0, 7777, 50);

	let open_no_orders = contract.get_open_orders(0, 0);
	let open_yes_orders = contract.get_open_orders(0, 1);
	assert_eq!(open_no_orders.len(), 0);
	assert_eq!(open_yes_orders.len(), 0);
	let filled_no_orders = contract.get_filled_orders(0, 0);
	let filled_yes_orders = contract.get_filled_orders(0, 1);
	assert_eq!(filled_no_orders.len(), 1);
	assert_eq!(filled_yes_orders.len(), 2);
}
use super::*;

#[test]
fn test_market_orders() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

	// simplest binary fill scenario
	contract.place_order("bid", 0, 1, 5000, 50, None); // 0
	contract.place_order("bid", 0, 1, 5000, 50, None); // 1

	let mut yes_market_price = contract.get_market_price(0, 0);
	assert_eq!(yes_market_price, 50);

	contract.place_order("bid", 0, 1, 5000, 60, None); // 2
	yes_market_price= contract.get_market_price(0, 0);
	assert_eq!(yes_market_price, 40);

	contract.cancel_order(0, 1, 2);
	yes_market_price = contract.get_market_price(0, 0);
	assert_eq!(yes_market_price, 50);

	contract.cancel_order(0, 1, 1);
	yes_market_price = contract.get_market_price(0, 0);
	assert_eq!(yes_market_price, 50);

	contract.cancel_order(0, 1, 0);
	yes_market_price = contract.get_market_price(0, 0);
	assert_eq!(yes_market_price, 100);

}

use super::*;

#[test]
fn test_contract_creation() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
}

#[test]
fn test_market_creation() {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.create_market("Hi!".to_string(), empty_string(), 4.into(), outcome_tags(4), categories(), market_end_timestamp_ms().into(), 0.into(), 0.into(), "test".to_string());
}

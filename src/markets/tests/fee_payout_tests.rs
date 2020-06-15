use super::*;

fn init_tests() -> Markets {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 4, outcome_tags(4), categories(), market_end_timestamp_ms(), 4, 50, "test".to_string());
	return contract;
}

#[test]
fn fee_distribution_test() {
	let mut contract = init_tests();
	let one_dai = to_dai(1);

	testing_env!(get_context(alice(), current_block_timestamp()));
	contract.claim_fdai();

	contract.place_order(0, 0, 5 * one_dai, 50, Some(affiliate()));
	contract.place_order(0, 1, 5 * one_dai, 50, Some(affiliate()));

	contract.place_order(0, 1, 5 * one_dai, 50, Some(affiliate()));
	contract.place_order(0, 1, 5 * one_dai, 50, Some(affiliate()));
	contract.place_order(0, 1, 5 * one_dai, 50, Some(affiliate()));
	contract.place_order(0, 1, 5 * one_dai, 50, Some(affiliate()));

	let markets = contract.get_markets_by_id(vec![0]);
	assert_eq!(markets[&0].filled_volume, 10 * one_dai);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.claim_fdai();
	contract.resolute_market(0, Some(1), 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, Some(1));

	testing_env!(get_context(alice(), market_end_timestamp_ns() + 1800000000000));
	let market = contract.get_markets_by_id(vec![0])[&0];


	// let creator_balance_before_claim = contract.get_fdai_balance(carol());
	// let resolutor_balance_before_claim = contract.get_fdai_balance(bob());
	// let trader_balance_before_claim = contract.get_fdai_balance(alice());
	// // let affiliate_balance_before_claim = contract.get_fdai_balance(affiliate());
	// let claimable_trader = contract.get_claimable(0, alice());
	// let expected_claimable_trader_excl_fees = 30 * one_dai;
	// let fees = 10 * one_dai * market.resolution_fee_percentage / 100 + 10 * one_dai * market.creator_fee_percentage / 100;

	// println!("trader claimable: {}", claimable_trader);

	contract.claim_earnings(0, alice());
	// contract.claim_earnings(0, bob());
	// contract.claim_earnings(0, affiliate());
	// contract.claim_earnings(0, carol());

	// let creator_balance_after_claim = contract.get_fdai_balance(carol());
	// let resolutor_balance_after_claim = contract.get_fdai_balance(bob());
	// let trader_balance_after_claim = contract.get_fdai_balance(alice());
	// // let affiliate_balance_after_claim = contract.get_fdai_balance(affiliate());

	// let market = contract.get_markets_by_id(vec![0])[&0];
	// assert_eq!(creator_balance_after_claim, creator_balance_before_claim + 10 * one_dai * (market.creator_fee_percentage / 2) / 100);
	// // assert_eq!(affiliate_balance_after_claim, affiliate_balance_before_claim + 10 * one_dai * (market.creator_fee_percentage / 2) / 100);
	// assert_eq!(resolutor_balance_after_claim, resolutor_balance_before_claim + 10 * one_dai * market.resolution_fee_percentage / 100 + 5 * one_dai);
	// // assert_eq!(trader_balance_after_claim, trader_balance_before_claim + expected_claimable_trader_excl_fees - fees);


}
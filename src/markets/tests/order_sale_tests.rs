use super::*;
use std::cmp;

fn init_with_buyer_and_seller() -> (String, String, Markets) {
	let mut contract = Markets::default();
	
	testing_env!(get_context(carol(), current_block_timestamp()));
	contract.claim_fdai();
	
	testing_env!(get_context(alice(), current_block_timestamp()));
	contract.claim_fdai();

	testing_env!(get_context(bob(), current_block_timestamp()));
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 2, outcome_tags(0), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());

	return(alice(), carol(), contract)
}

fn simplest_order_sale() -> (String, String, Markets) {
	let (buyer, seller, mut contract) = init_with_buyer_and_seller();
	testing_env!(get_context(seller.to_string(), current_block_timestamp()));
	contract.place_order(0, 0, 10000, 50, None);
	contract.place_order(0, 1, 10000, 50, None);
	let buy_price = 50;
	// Alice places 100 shares worth of buy orders
	testing_env!(get_context(buyer.to_string(), current_block_timestamp()));
	contract.place_order(0, 1, 100 * buy_price, buy_price, None);

	testing_env!(get_context(seller.to_string(), current_block_timestamp()));

	// Record dai balance pre-sale
	let seller_dai_balance = contract.get_fdai_balance(seller.to_string());

	let seller_share_balance = contract.get_outcome_share_balance(0, 1, carol());
	assert_eq!(200, seller_share_balance);

	let (sell_depth, shares_fillable) = contract.get_market_sell_depth(0, 1, 10000);
	assert_eq!(100, shares_fillable);
	assert_eq!(100 * buy_price, sell_depth);

	let buyer_share_balance = contract.get_outcome_share_balance(0, 1, alice());
	assert_eq!(buyer_share_balance, 0);

	contract.dynamic_market_sell(0, 1, seller_share_balance);

	// check share balance post sell
	let seller_share_balance = contract.get_outcome_share_balance(0, 1, carol());
	assert_eq!(seller_share_balance, 100);
	
	// check dai balance post sell
	let seller_dai_balance_post_sell = contract.get_fdai_balance(seller.to_string());
	assert_eq!(seller_dai_balance_post_sell, seller_dai_balance + 5000);

	let buyer_share_balance = contract.get_outcome_share_balance(0, 1, alice());
	assert_eq!(buyer_share_balance, 100);

	return (buyer, seller, contract);
}

fn partial_buy_order_fill_through_sale(buy_price: u128) -> (String, String, Markets) {
	let (buyer, seller, mut contract) = init_with_buyer_and_seller();
	
	// 200 yes & no shares TODO test w/ uneven share amount
	testing_env!(get_context(seller.to_string(), current_block_timestamp()));
	contract.place_order(0, 0, 10000, 50, None);
	contract.place_order(0, 1, 10000, 50, None);
	
	// buyer places 1000 shares worth of buy orders
	testing_env!(get_context(buyer.to_string(), current_block_timestamp()));
	contract.place_order(0, 1, 1000 * buy_price, buy_price, None);

	testing_env!(get_context(seller.to_string(), current_block_timestamp()));

	// Record dai balance pre-sale
	let seller_dai_balance = contract.get_fdai_balance(seller.to_string());

	let seller_share_balance = contract.get_outcome_share_balance(0, 1, seller.to_string());
	assert_eq!(200, seller_share_balance);
	
	let (sell_depth, shares_fillable) = contract.get_market_sell_depth(0, 1, 10000);
	assert_eq!(1000, shares_fillable);
	assert_eq!(1000 * buy_price, sell_depth);

	let buyer_share_balance = contract.get_outcome_share_balance(0, 1, buyer.to_string());
	assert_eq!(buyer_share_balance, 0);

	contract.dynamic_market_sell(0, 1, seller_share_balance);

	// check share balance post sell
	let seller_share_balance = contract.get_outcome_share_balance(0, 1, seller.to_string());
	assert_eq!(seller_share_balance, 0);
	
	// check dai balance post sell
	let seller_dai_balance_post_sell = contract.get_fdai_balance(seller.to_string());
	assert_eq!(seller_dai_balance_post_sell, seller_dai_balance + 200 * cmp::min(buy_price, 50));

	let buyer_share_balance = contract.get_outcome_share_balance(0, 1, buyer.to_string());
	assert_eq!(buyer_share_balance, 200);

	return (buyer, seller, contract);
}

#[test]
fn test_simplest_order_sale() {
	simplest_order_sale();
}


#[test]
fn test_partial_buy_order_fill_through_sale() {
	partial_buy_order_fill_through_sale(60);
}

#[test]
fn test_simple_market_order_sale_payout_valid() {
	let (buyer, seller, mut contract) = simplest_order_sale();
	let one_dai = to_dai(1);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(1), 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, Some(1));

	let claimable_seller = contract.get_claimable(0, seller.to_string());
	let expected_claimable_seller = 9900;
	assert_eq!(claimable_seller, expected_claimable_seller);
	
	let claimable_buyer = contract.get_claimable(0, buyer.to_string());
	let expected_claimable_buyer = 9900;
	assert_eq!(claimable_buyer, expected_claimable_buyer);
}

#[test]
fn test_simple_market_order_sale_payout_invalid() {
	let (buyer, seller, mut contract) = simplest_order_sale();
	let one_dai = to_dai(1);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.resolute_market(0, None, 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, None);

	let claimable_seller = contract.get_claimable(0, seller.to_string());
	let expected_claimable_seller = 14850;
	
	let claimable_buyer = contract.get_claimable(0, buyer.to_string());
	let expected_claimable_buyer = 4950;
	
	assert_eq!(claimable_seller, expected_claimable_seller);
	assert_eq!(claimable_buyer, expected_claimable_buyer);
}


#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_valid() {
	let (buyer, seller, mut contract) = partial_buy_order_fill_through_sale(40);
	let one_dai = to_dai(1);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(1), 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, Some(1));

	let claimable_seller = contract.get_claimable(0, seller.to_string());
	let expected_claimable_seller = 0;
	
	let claimable_buyer = contract.get_claimable(0, buyer.to_string());
	let expected_claimable_buyer = 51800;
	
	assert_eq!(claimable_seller, expected_claimable_seller);
	assert_eq!(claimable_buyer, expected_claimable_buyer);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_invalid() {
	let (buyer, seller, mut contract) = partial_buy_order_fill_through_sale(40);
	let one_dai = to_dai(1);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.resolute_market(0, None, 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, None);

	let claimable_seller = contract.get_claimable(0, seller.to_string());
	let expected_claimable_seller = 11880;
	
	let claimable_buyer = contract.get_claimable(0, buyer.to_string());
	let expected_claimable_buyer = 39600;
	
	assert_eq!(claimable_seller, expected_claimable_seller);
	assert_eq!(claimable_buyer, expected_claimable_buyer);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_profit_payout_valid() {
	let (buyer, seller, mut contract) = partial_buy_order_fill_through_sale(60);
	let one_dai = to_dai(1);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(1), 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, Some(1));

	let claimable_seller = contract.get_claimable(0, seller.to_string());
	let expected_claimable_seller = 0;
	
	let claimable_buyer = contract.get_claimable(0, buyer.to_string());
	let expected_claimable_buyer = 67800;
	
	assert_eq!(claimable_seller, expected_claimable_seller);
	assert_eq!(claimable_buyer, expected_claimable_buyer);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_profit_payout_invalid() {
	let (buyer, seller, mut contract) = partial_buy_order_fill_through_sale(60);
	let one_dai = to_dai(1);

	testing_env!(get_context(bob(), market_end_timestamp_ns()));
	contract.resolute_market(0, None, 5 * one_dai);
	testing_env!(get_context(bob(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, None);

	let claimable_seller = contract.get_claimable(0, seller.to_string());
	let expected_claimable_seller = 9900;
	
	let claimable_buyer = contract.get_claimable(0, buyer.to_string());
	let expected_claimable_buyer = 59400;
	
	assert_eq!(claimable_seller, expected_claimable_seller);
	assert_eq!(claimable_buyer, expected_claimable_buyer);
}



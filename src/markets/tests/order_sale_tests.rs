use super::*;
use std::cmp;

fn simplest_order_sale() -> (Vec<ExternalUser>, ExternalUser, RuntimeStandalone) {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let buyer = &accounts[0];
	let seller = &accounts[1];

	buyer.transfer(&mut runtime, seller.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	buyer.transfer(&mut runtime, root.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	buyer.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	seller.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	let buy_price = 50;
	seller.place_order(&mut runtime, U64(0), U64(0), U128(2000 * buy_price), U128(buy_price), None).expect("order placement failed unexpectedly"); 
	seller.place_order(&mut runtime, U64(0), U64(1), U128(2000 * buy_price), U128(buy_price), None).expect("order placement failed unexpectedly"); 
	
	buyer.place_order(&mut runtime, U64(0), U64(1), U128(1000 * buy_price), U128(buy_price), None).expect("order placement failed unexpectedly");

	let initial_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();

	let share_balance_seller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(2000, share_balance_seller);
	
	let (spendable, shares_fillable) = seller.get_market_sell_depth(&mut runtime, U64(0), U64(1), U128(10000));
	let spendable: u128 = spendable.into();
	let shares_fillable: u128 = shares_fillable.into();
	assert_eq!(1000, shares_fillable);
	assert_eq!(1000 * buy_price, spendable);
	
	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(0, share_balance_buyer);

	seller.dynamic_market_sell(&mut runtime, U64(0), U64(1), U128(share_balance_seller)).expect("market sell failed unexpectedly");

	let dai_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();
	assert_eq!(dai_balance_seller, initial_balance_seller + 50000 - 500);

	// check share balance post sell
	let share_balance_seller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(share_balance_seller, 1000);

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(share_balance_buyer, 1000);
	return (accounts, root, runtime);
}

fn partial_buy_order_fill_through_sale(buy_price: u128) -> (Vec<ExternalUser>, ExternalUser, RuntimeStandalone) {
	let (mut runtime, root, accounts) = init_runtime_env();
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), U64(2), outcome_tags(0), categories(), U64(market_end_timestamp_ms()), U128(0), U128(0), "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let buyer = &accounts[0];
	let seller = &accounts[1];

	buyer.transfer(&mut runtime, seller.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	buyer.transfer(&mut runtime, root.get_account_id(), ntoy(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	buyer.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	seller.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");	

	seller.place_order(&mut runtime, U64(0), U64(0), U128(2000 * 50), U128(50), None).expect("order placement failed unexpectedly"); // 10.000
	seller.place_order(&mut runtime, U64(0), U64(1), U128(2000 * 50), U128(50), None).expect("order placement failed unexpectedly"); // 0
	
	buyer.place_order(&mut runtime, U64(0), U64(1), U128(10000 * buy_price), U128(buy_price), None).expect("order placement failed unexpectedly"); // 48.000 / 12.000 

	let initial_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();

	let share_balance_sller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(2000, share_balance_sller);
	
	let (spendable, shares_fillable) = seller.get_market_sell_depth(&mut runtime, U64(0), U64(1), U128(10000));
	let spendable: u128 = spendable.into();
	let shares_fillable: u128 = shares_fillable.into();
	assert_eq!(10000, shares_fillable);
	assert_eq!(10000 * buy_price, spendable);

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(0, share_balance_buyer);

	let tx_res = seller.dynamic_market_sell(&mut runtime, U64(0), U64(1), U128(share_balance_sller)).expect("market sell failed unexpectedly");

	// check share balance post sell
	let share_balance_seller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(share_balance_seller, 0);

	let dai_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();
	let sell_fee = 1 * 2000 * cmp::min(buy_price, 50) / 100;
	assert_eq!(dai_balance_seller, initial_balance_seller + 2000 * cmp::min(buy_price, 50) - sell_fee);

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), U64(1)).into();
	assert_eq!(share_balance_buyer, 2000);
	return (accounts, root, runtime);
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
	let (accounts, root, mut runtime) = simplest_order_sale();

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(U64(1)), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), Some(U64(1))).expect("market resolution failed unexpectedly");

	
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();

	let expected_claimable_seller = 99000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 99000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_simple_market_order_sale_payout_invalid() {
	let (accounts, root, mut runtime) = simplest_order_sale();

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();

	let expected_claimable_seller = 148500;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 49500;
	assert_eq!(claimable_buyer, expected_claimable_buyer);
	
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}


#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_valid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(40);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(U64(1)), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), Some(U64(1))).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 0;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 518000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_invalid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(40);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 118800;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 399200;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_profit_payout_valid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(60);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(U64(1)), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), Some(U64(1))).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 19800;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 678000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_profit_payout_invalid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(60);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 1800000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 99000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 598800;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}



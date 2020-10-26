use super::*;
use std::cmp;

fn simplest_order_sale() -> (Vec<ExternalUser>, ExternalUser, RuntimeStandalone) {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	
	let buyer = &accounts[0];
	let seller = &accounts[1];
	
	buyer.transfer(&mut runtime, seller.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	buyer.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	buyer.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	seller.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = root.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	
	let buy_price = 50;
	let res = seller.place_order(&mut runtime, U64(0), 0, U128(2000), U128(buy_price), None).expect("order placement failed unexpectedly");
	let res = seller.place_order(&mut runtime, U64(0), 1, U128(2000), U128(buy_price), None).expect("order placement failed unexpectedly");  
	let res = buyer.place_order(&mut runtime, U64(0), 1, U128(1000), U128(buy_price), None).expect("order placement failed unexpectedly"); 

	let initial_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();

	let share_balance_seller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(2000, share_balance_seller);
	
	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(0, share_balance_buyer);

	let res = seller.dynamic_market_sell(&mut runtime, U64(0), 1, U128(share_balance_seller), U128(1)).expect("market sell failed unexpectedly");

	let dai_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();
	assert_eq!(dai_balance_seller, initial_balance_seller + 50000);

	// check share balance post sell
	let share_balance_seller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_seller, 1000);

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_buyer, 1000);

	let market_volume = accounts[0].get_market_volume(&mut runtime, U64(0));
	assert_eq!(market_volume, U128(200000));

	return (accounts, root, runtime);
}

fn partial_buy_order_fill_through_sale(buy_price: u128) -> (Vec<ExternalUser>, ExternalUser, RuntimeStandalone) {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	
	let buyer = &accounts[0];
	let seller = &accounts[1];
	buyer.transfer(&mut runtime, seller.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	buyer.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	buyer.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	seller.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");	
	let tx_res = root.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string()).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	// bp 40
	
	seller.place_order(&mut runtime, U64(0), 0, U128(2000), U128(50), None).expect("order placement failed unexpectedly"); // 100.000 own 2000 shares
	seller.place_order(&mut runtime, U64(0), 1, U128(2000), U128(50), None).expect("order placement failed unexpectedly"); // 100.000 own 2000 shares
	
	buyer.place_order(&mut runtime, U64(0), 1, U128(10000), U128(buy_price), None).expect("order placement failed unexpectedly"); // 400.000 10000 shares buyer expects 400 000 back

	let initial_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();

	let share_balance_sller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(2000, share_balance_sller);
	
	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(0, share_balance_buyer);

	let tx_res = seller.dynamic_market_sell(&mut runtime, U64(0), 1, U128(share_balance_sller), U128(1)).expect("market sell failed unexpectedly");

	// check share balance post sell
	let share_balance_seller: u128 = seller.get_outcome_share_balance(&mut runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_seller, 0);

	let dai_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();
	assert_eq!(dai_balance_seller, initial_balance_seller + 2000 * cmp::min(buy_price, 50));

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&mut runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_buyer, 2000);
	
	let market_volume = accounts[0].get_market_volume(&mut runtime, U64(0));
	assert_eq!(market_volume, U128(200000));

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
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();

	let expected_claimable_seller = 99000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 99000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	// assert_eq!(contract_balance, 5250000000000200000);

	let res = buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let res = seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	
	let res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
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
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();

	let expected_claimable_seller = 148500;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 49500;
	assert_eq!(claimable_buyer, expected_claimable_buyer);
	
	let validity_bond = to_dai(25) / 100;
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, validity_bond);
}


#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_valid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(40);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 0;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 518000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	let res = buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
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
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	
	let expected_claimable_seller = 118800;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 399200;
	assert_eq!(claimable_buyer, expected_claimable_buyer);
	
	let res = buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let res = seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	let validity_bond = to_dai(25) / 100;
	assert_eq!(contract_balance, validity_bond);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_profit_payout_valid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(60);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5))).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 20000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 678000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	let res = seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let res = buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");
	let res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

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
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 99000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = 598800;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();


	let res = seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let res = buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id()).expect("claim_earnigns tx failed unexpectedly");

	let res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id()).expect("claim_earnigns tx failed unexpectedly");


	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	let validity_bond = to_dai(25) / 100;
	assert_eq!(contract_balance, validity_bond);
}



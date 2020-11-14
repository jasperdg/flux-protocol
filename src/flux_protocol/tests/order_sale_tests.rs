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
	let tx_res = root.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));
	
	let buy_price = 50;
	seller.place_order(&mut runtime, U64(0), 0, U128(to_shares(2)), buy_price, None, None).expect("order placement failed unexpectedly");
	seller.place_order(&mut runtime, U64(0), 1, U128(to_shares(2)), buy_price, None, None).expect("order placement failed unexpectedly");  
	buyer.place_order(&mut runtime, U64(0), 1, U128(to_shares(1)), buy_price, None, None).expect("order placement failed unexpectedly"); 

	let initial_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();

	let share_balance_seller: u128 = seller.get_outcome_share_balance(&runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(to_shares(2), share_balance_seller);
	
	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(0, share_balance_buyer);

	seller.dynamic_market_sell(&mut runtime, U64(0), 1, U128(share_balance_seller), 1, None).expect("market sell failed unexpectedly");

	let dai_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();
	let expected_balance_seller: u128 = to_shares(1) * cmp::min(buy_price, 50) as u128;
	let sell_fee = expected_balance_seller / 100;
	assert_eq!(dai_balance_seller, initial_balance_seller + expected_balance_seller - sell_fee);

	// check share balance post sell
	let share_balance_seller: u128 = seller.get_outcome_share_balance(&runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_seller, to_shares(1));

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_buyer, to_shares(1));

	let market_volume = accounts[0].get_market_volume(&runtime, U64(0));
	let expected_volume = to_shares(4) * 50 + to_shares(1) * u128::from(buy_price);

	assert_eq!(market_volume, U128(expected_volume));

	(accounts, root, runtime)
}

fn partial_buy_order_fill_through_sale(buy_price: u16) -> (Vec<ExternalUser>, ExternalUser, RuntimeStandalone) {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	
	let buyer = &accounts[0];
	let seller = &accounts[1];
	buyer.transfer(&mut runtime, seller.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	buyer.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	buyer.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	seller.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");	
	let tx_res = root.create_market(&mut runtime, empty_string(), empty_string(), 2, outcome_tags(0), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	
	seller.place_order(&mut runtime, U64(0), 0, U128(to_shares(2)), 50, None, None).expect("order placement failed unexpectedly"); 
	seller.place_order(&mut runtime, U64(0), 1, U128(to_shares(2)), 50, None, None).expect("order placement failed unexpectedly"); 
	
	buyer.place_order(&mut runtime, U64(0), 1, U128(to_shares(10)), buy_price, None, None).expect("order placement failed unexpectedly");

	let initial_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();

	println!("initial balance {}", initial_balance_seller);
	println!("params balance {} bp{}", (to_shares(2) * cmp::min(buy_price, 50) as u128), buy_price);

	let share_balance_seller: u128 = seller.get_outcome_share_balance(&runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(to_shares(2), share_balance_seller);
	
	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(0, share_balance_buyer);

	let tx_res = seller.dynamic_market_sell(&mut runtime, U64(0), 1, U128(share_balance_seller), 1, None).expect("market sell failed unexpectedly");
	println!("sell transaction result {:?}", tx_res);
	// check share balance post sell
	let share_balance_seller: u128 = seller.get_outcome_share_balance(&runtime, seller.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_seller, 0);

	let dai_balance_seller: u128 = seller.get_balance(&mut runtime, seller.get_account_id()).into();
	let expected_balance_seller: u128 = to_shares(2) * cmp::min(buy_price, 50) as u128;
	let sell_fee = expected_balance_seller / 100;
	assert_eq!(dai_balance_seller, initial_balance_seller + expected_balance_seller - sell_fee);

	let share_balance_buyer: u128 = buyer.get_outcome_share_balance(&runtime, buyer.get_account_id(), U64(0), 1).into();
	assert_eq!(share_balance_buyer, to_shares(2));
	
	let market_volume = accounts[0].get_market_volume(&runtime, U64(0));
	let expected_volume = to_shares(4) * 50 + to_shares(2) * u128::from(buy_price);
	assert_eq!(market_volume, U128(expected_volume));

	(accounts, root, runtime)
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
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();

	let expected_claimable_seller = to_dai(1) - to_dai(1) / 100;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = to_dai(1) - to_dai(1) / 100;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 7255000000000000000);

	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_simple_market_order_sale_payout_invalid() {
	let (accounts, root, mut runtime) = simplest_order_sale();

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();

	let expected_claimable_seller = to_dai(1485) / 1000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = to_dai(49500) / 100000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);
	
	let validity_bond = to_dai(25) / 100;
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, validity_bond);
}


#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_valid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(40);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = 0;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = to_dai(518) / 100;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_loss_payout_invalid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(40);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();
	
	let expected_claimable_seller = to_dai(1188) / 1000;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = to_dai(3992) / 1000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);
	
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");

	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");

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
	root.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let seller_fee = to_dai(2) / 1000;
	let expected_claimable_seller = to_dai(2) / 10 - seller_fee;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = to_dai(678) / 100;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");


	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	let root_res = root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	println!("contract balance: {} root_tx: {:?}", contract_balance, root_res);

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

#[test]
fn test_dynamically_priced_market_order_sale_for_profit_payout_invalid() {
	let (accounts, root, mut runtime) = partial_buy_order_fill_through_sale(60);

	let buyer = &accounts[0];
	let seller = &accounts[1]; 

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	root.resolute_market(&mut runtime, U64(0), None, U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), None).expect("market resolution failed unexpectedly");

	let claimable_seller: u128 = seller.get_claimable(&mut runtime, U64(0), seller.get_account_id()).into();
	let claimable_buyer: u128 = buyer.get_claimable(&mut runtime, U64(0), buyer.get_account_id()).into();

	let expected_claimable_seller = to_dai(1) - to_dai(1) / 100;
	assert_eq!(claimable_seller, expected_claimable_seller);
	let expected_claimable_buyer = to_dai(5988) / 1000;
	assert_eq!(claimable_buyer, expected_claimable_buyer);

	seller.claim_earnings(&mut runtime, U64(0), seller.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	buyer.claim_earnings(&mut runtime, U64(0), buyer.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	let validity_bond = to_dai(25) / 100;
	assert_eq!(contract_balance, validity_bond);
}



use super::*;

#[test]
fn test_dispute_valid() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.place_order(&mut runtime, U64(0), 0, U128(to_dai(1) / 10), 70, None, None).expect("order placement failed unexpectedly");
	alice.place_order(&mut runtime, U64(0), 3, U128(to_dai(1) / 10), 10, None, None).expect("order placement failed unexpectedly");
	
	carol.place_order(&mut runtime, U64(0), 1, U128(to_dai(1) / 10), 10, None, None).expect("order placement failed unexpectedly");
	carol.place_order(&mut runtime, U64(0), 2, U128(to_dai(1) / 10), 10, None, None).expect("order placement failed unexpectedly");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(1)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	carol.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(4)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(10)), None).expect("market dispute failed unexpectedly"); 
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed unexpectedly"); 

	let expected_claimable_alice = to_dai(11) - ((to_dai(10) / 100) / 5) * 4;
	let expected_claimable_carol = to_dai(4) + ((to_dai(10) / 100) / 5) * 4;
	
	let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let initially_claimable_carol: u128 = alice.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();
	let validity_bond = to_dai(25) / 100;

	assert_eq!(initially_claimable_alice, expected_claimable_alice + validity_bond);
	assert_eq!(initially_claimable_carol, expected_claimable_carol);

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let initial_balance_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into();
	
	alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	carol.claim_earnings(&mut runtime, U64(0), carol.get_account_id(), None).expect("claim_earnings tx failed unexpectedly");
	let balance_after_claim_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let balance_after_claim_carol: u128 = alice.get_balance(&mut runtime, carol.get_account_id()).into();
	
	assert_eq!(initial_balance_alice + initially_claimable_alice, balance_after_claim_alice);
	assert_eq!(initial_balance_carol + initially_claimable_carol, balance_after_claim_carol);

	let contract_balance: u128 = alice.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, to_dai(10));

}

#[test]
#[should_panic(expected = "market isn't resoluted yet")]
fn test_market_not_resoluted() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	alice.dispute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("dispute failed");
}

#[test]
#[should_panic(expected = "market is already finalized")]
fn test_finalized_market() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed unexpectedly"); 

	alice.dispute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("dispute failed");
}

#[test]
#[should_panic(expected = "dispute window still open")]
fn test_market_finalization_pre_dispute_window_close() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 900000000000;
	root.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed as expected"); 
}

#[test]
#[should_panic(expected = "dispute window is closed, market can be finalized")]
fn test_dispute_after_dispute_window() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	let dispute_res = alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("dispute failed");
	println!("dispute res: {:?}", dispute_res);
}

#[test]
#[should_panic(expected = "only the judge can resolute disputed markets")]
fn test_finalize_as_not_owner() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(10)), None).expect("dispute failed");

	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	alice.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed as expected"); 
}

#[test]
#[should_panic(expected = "invalid winning outcome")]
fn test_invalid_dispute_outcome() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(4), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
}

#[test]
#[should_panic(expected = "same outcome as last resolution")]
fn test_dispute_with_same_outcome() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	alice.dispute_market(&mut runtime, U64(0), Some(0), U128(to_dai(10)), None).expect("dispute failed");
}

#[test]
#[should_panic(expected = "for this version, there's only 1 round of dispute")]
fn test_dispute_escalation_failure() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(10)), None).expect("dispute failed");
	alice.dispute_market(&mut runtime, U64(0), Some(2), U128(to_dai(20)), None).expect("dispute failed");
}

#[test]
fn test_stake_refund() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	let pre_resolution_balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	
	let expected_post_resolution_balance = pre_resolution_balance - to_dai(5);

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(7)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 

	let post_resolution_balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	assert_eq!(post_resolution_balance, expected_post_resolution_balance);
	
	let expected_post_dispute_balance = post_resolution_balance - to_dai(10);
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(15)), None).expect("dispute failed");
	let post_dispute_balance: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();

	assert_eq!(post_dispute_balance, expected_post_dispute_balance);
}

#[test]
#[should_panic(expected = "previous promise failed")]
fn test_insufficient_balance() {
	let (mut runtime, _root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];

	alice.set_allowance(&mut runtime, flux_protocol(), U128(ntoy(101))).expect("allowance couldn't be set");
	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(ntoy(101)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
} 

#[test]
#[should_panic(expected = "you cant cancel dispute stake for bonded outcome")]
fn test_cancel_dispute_participation() {

	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];

	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	
	alice.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(4)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); // carol resolutes correctly - should have 1 % of 10 dai as claimable 
	
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(10)), None).expect("market dispute failed unexpectedly"); 
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed unexpectedly"); 

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let expected_balance_after_withdrawl = initial_balance_alice + to_dai(4);
	alice.withdraw_resolution_stake(&mut runtime, U64(0), 0, Some(1), None).expect("dispute stake claim failed");
	let balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	
	assert_eq!(expected_balance_after_withdrawl, balance_alice);
	alice.withdraw_resolution_stake(&mut runtime, U64(0), 0, Some(0), None).expect("dispute stake claim failed");
}

#[test]
fn test_cancel_dispute_participation_non_bonded_winning_outcome() {

	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];

	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(4)), None).expect("market resolution failed unexpectedly");
	alice.resolute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly");
	
	alice.dispute_market(&mut runtime, U64(0), Some(0), U128(to_dai(10)), None).expect("market dispute failed unexpectedly"); 
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed unexpectedly"); 
	

	let initial_balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();
	let expected_balance_after_withdrawl = initial_balance_alice + to_dai(4);
	alice.withdraw_resolution_stake(&mut runtime, U64(0), 0, Some(0), None).expect("dispute stake claim failed");
	let balance_alice: u128 = alice.get_balance(&mut runtime, alice.get_account_id()).into();

	assert_eq!(expected_balance_after_withdrawl, balance_alice);
}

#[test]
fn test_crowdsourced_dispute_correct_resolution() {
	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");

	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	root.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, None, None).expect("order placement failed unexpectedly");
	root.place_order(&mut runtime, U64(0), 0, U128(to_dai(5) / 50), 50, None, None).expect("order placement failed unexpectedly");

	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	
	carol.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(3)), None).expect("market resolution failed unexpectedly"); 
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(2)), None).expect("market resolution failed unexpectedly"); 
	
	carol.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); 
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); 
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(0)).expect("market finalization failed unexpectedly"); 

	let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let initially_claimable_carol: u128 = carol.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();

	let expected_claimable_carol = 100000000000000000 * to_dai(3) / to_dai(5) + to_dai(3);
	let expected_claimable_alice = 100000000000000000 * to_dai(2) / to_dai(5) + to_dai(2);
	
	let validity_bond = to_dai(25) / 100;
	assert_eq!(initially_claimable_carol, expected_claimable_carol);
	assert_eq!(initially_claimable_alice, expected_claimable_alice + validity_bond);
}

#[test]
fn test_crowdsourced_dispute_incorrect_resolution() {

	let (mut runtime, root, accounts) = init_runtime_env();
	accounts[0].set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	let tx_res = accounts[0].create_market(&mut runtime, empty_string(), empty_string(), 4, outcome_tags(4), categories(), U64(market_end_timestamp_ms()), 0, 0, "test".to_string(), None).unwrap();
	assert_eq!(tx_res.status, ExecutionStatus::SuccessValue(b"0".to_vec()));

	let alice = &accounts[0];
	let carol = &accounts[1];
	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	carol.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	alice.transfer(&mut runtime, carol.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");
	alice.transfer(&mut runtime, root.get_account_id(), to_dai(30).into()).expect("transfer failed couldn't be set");

	root.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");
	root.place_order(&mut runtime, U64(0), 1, U128(to_dai(5) / 50), 50, None, None).expect("order placement failed unexpectedly");
	root.place_order(&mut runtime, U64(0), 0, U128(to_dai(5) / 50), 50, None, None).expect("order placement failed unexpectedly");

	alice.set_allowance(&mut runtime, flux_protocol(), U128(to_dai(30))).expect("allowance couldn't be set");

	runtime.current_block().block_timestamp = market_end_timestamp_ns();
	
	carol.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(3)), None).expect("market resolution failed unexpectedly"); 
	alice.resolute_market(&mut runtime, U64(0), Some(0), U128(to_dai(2)), None).expect("market resolution failed unexpectedly"); 
	
	carol.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); 
	alice.dispute_market(&mut runtime, U64(0), Some(1), U128(to_dai(5)), None).expect("market resolution failed unexpectedly"); 
	
	runtime.current_block().block_timestamp = market_end_timestamp_ns() + 43200000000000;
	root.finalize_market(&mut runtime, U64(0), Some(1)).expect("market finalization failed unexpectedly"); 

	let initially_claimable_alice: u128 = alice.get_claimable(&mut runtime, U64(0), alice.get_account_id()).into();
	let initially_claimable_carol: u128 = carol.get_claimable(&mut runtime, U64(0), carol.get_account_id()).into();
	let total_res_fee: u128 = to_dai(10) / 100;

	let expected_claimable_carol = to_dai(75) / 10 + total_res_fee / 2;
	let expected_claimable_alice = to_dai(75) / 10 + total_res_fee / 2;

	let validity_bond = to_dai(25) / 100;
	assert_eq!(initially_claimable_carol, expected_claimable_carol);
	assert_eq!(initially_claimable_alice, expected_claimable_alice + validity_bond);

	alice.claim_earnings(&mut runtime, U64(0), alice.get_account_id(), None).expect("claim earnings failed unexpectedly");
	carol.claim_earnings(&mut runtime, U64(0), carol.get_account_id(), None).expect("claim earnings failed unexpectedly");
	root.claim_earnings(&mut runtime, U64(0), root.get_account_id(), None).expect("claim earnings failed unexpectedly");

	let contract_balance: u128 = root.get_balance(&mut runtime, flux_protocol()).into();
	assert_eq!(contract_balance, 0);
}

// TODO: add coverage for withdrawing dispute stakes in further rounds
// TODO: add coverage for withdrawing dispute stakes for rounds where you pariticipated inthe winning round but that wasn't the bonded outcome that round
// TODO: add more generic dispute stake withdrawl tests
// TODO: test "change refund" when crowdfunding dispute resolution
use super::*;

fn init_tests() -> Markets {
	testing_env!(get_context(carol(), current_block_timestamp()));
	let mut contract = Markets::default();
	contract.claim_fdai();
	contract.create_market("Hi!".to_string(), empty_string(), 4, outcome_tags(4), categories(), market_end_timestamp_ms(), 0, 0, "test".to_string());
	return contract;
}

#[test]
fn test_dispute_valid() {
	let mut contract = init_tests();

	contract.place_order(0, 0, to_dai(7), 70, None);
	contract.place_order(0, 3, to_dai(1), 10, None);

	testing_env!(get_context(alice(), current_block_timestamp()));
	contract.claim_fdai();
	contract.place_order(0, 1, to_dai(1), 10, None);
	contract.place_order(0, 2, to_dai(1), 10, None);

	// carol tries to resolute towards her own order
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(0), to_dai(5));
	
	// alice disputes carol's outcome
    testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(10));

	// Judge agrees w/ carol
    testing_env!(get_context(judge(), market_end_timestamp_ns()));
    contract.finalize_market(0, Some(0));

    let claimable_carol = contract.get_claimable(0, carol()) ;
    let claimable_alice = contract.get_claimable(0, alice()) ;

    assert_eq!(claimable_carol, to_dai(15));
	assert_eq!(claimable_alice, 0);

	let fdai_before_claim_alice = contract.get_fdai_balance(alice());
	let fdai_before_claim_carol = contract.get_fdai_balance(carol());
	
	contract.claim_earnings(0, carol());
	contract.claim_earnings(0, alice());
	
	let fdai_after_claim_alice = contract.get_fdai_balance(alice());
	let fdai_after_claim_carol = contract.get_fdai_balance(carol());
	
	assert_eq!(fdai_before_claim_alice + claimable_alice, fdai_after_claim_alice);
	assert_eq!(fdai_before_claim_carol + claimable_carol, fdai_after_claim_carol);

	assert_eq!(contract.get_claimable(0, carol()), 0);
	assert_eq!(contract.get_claimable(0, alice()), 0);
}

#[test]
#[should_panic(expected = "market isn't resoluted yet")]
fn test_market_not_resoluted() {
	let mut contract = init_tests();
	contract.dispute_market(0, Some(0), to_dai(5));
}

#[test]
#[should_panic(expected = "market is already finalized")]
fn test_finalized_market() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(0), to_dai(5));
	testing_env!(get_context(judge(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, None);
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(5));
}

#[test]
#[should_panic(expected = "dispute window still open")]
fn test_market_finalization_pre_dispute_window_close() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
    contract.resolute_market(0, Some(0), to_dai(5));
	contract.finalize_market(0, None);
}

#[test]
#[should_panic(expected = "dispute window is closed, market can be finalized")]
fn test_dispute_after_dispute_window() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(0), to_dai(5));
	testing_env!(get_context(carol(), market_end_timestamp_ns() + 1800100000000));
	contract.dispute_market(0, None, to_dai(5));
}

#[test]
#[should_panic(expected = "only the judge can resolute disputed markets")]
fn test_finalize_as_not_owner() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(0), to_dai(5));
	contract.dispute_market(0, None, to_dai(10));
	testing_env!(get_context(carol(), market_end_timestamp_ns() + 1800000000000));
	contract.finalize_market(0, None);
}

#[test]
#[should_panic(expected = "invalid winning outcome")]
fn test_invalid_dispute_outcome() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(4), to_dai(5));
}

#[test]
#[should_panic(expected = "same oucome as last resolution")]
fn test_dispute_with_same_outcome() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(3), to_dai(5));
	contract.dispute_market(0, Some(3), to_dai(10));
}

#[test]
#[should_panic(expected = "for this version, there's only 1 round of dispute")]
fn test_dispute_escalation_failure() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(3), to_dai(5));
	contract.dispute_market(0, Some(2), to_dai(10));
	contract.dispute_market(0, Some(3), to_dai(20));
}

#[test]
fn test_stake_refund() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));

	let pre_resolution_balance = contract.get_fdai_balance(carol());
	let post_resolution_expected_balance = pre_resolution_balance - to_dai(5);
	
	contract.resolute_market(0, Some(3), to_dai(7));

	let post_resolution_balance = contract.get_fdai_balance(carol());

	assert_eq!(post_resolution_balance, post_resolution_expected_balance);

	let expected_post_dispute_balance = post_resolution_balance - to_dai(10);

	contract.dispute_market(0, Some(1), to_dai(15));

	let post_dispute_balance = contract.get_fdai_balance(carol());

	assert_eq!(expected_post_dispute_balance, post_dispute_balance);	
}

#[test]
#[should_panic(expected = "not enough balance to cover stake")]
fn test_insufficient_balance() {
	let mut contract = init_tests();
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(3), to_dai(101));

}

#[test]
#[should_panic(expected = "you cant cancel dispute stake for winning outcome")]
fn test_cancel_dispute_participation() {
	let mut contract = init_tests();

	contract.place_order(0, 0, to_dai(10), 70, None);
	contract.place_order(0, 3, to_dai(1), 10, None);

	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.claim_fdai();
	contract.resolute_market(0, Some(1), to_dai(4));
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
    contract.resolute_market(0, Some(0), to_dai(5));
	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(10));
    testing_env!(get_context(judge(), market_end_timestamp_ns()));
	contract.finalize_market(0, Some(0));
	
	contract.claim_earnings(0, alice());

	let fdai_before_withdrawl_alice = contract.get_fdai_balance(alice());

	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.withdraw_dispute_stake(0, 0, Some(1));
	let fdai_after_withdrawl_alice = contract.get_fdai_balance(alice());
	assert_eq!(fdai_after_withdrawl_alice, fdai_before_withdrawl_alice + to_dai(4));

	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.withdraw_dispute_stake(0, 1, Some(0));
	
}

#[test]
fn test_crowdsourced_dispute_correct_resolution() {
	let mut contract = init_tests();
	testing_env!(get_context(bob(), current_block_timestamp()));
	contract.claim_fdai();

	contract.place_order(0, 0, to_dai(5), 50, None);
	contract.place_order(0, 1, to_dai(5), 50, None);
	
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(0), to_dai(3));
	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.claim_fdai();
	contract.resolute_market(0, Some(0), to_dai(2));
	
	let resolution_window_0 = contract.get_active_resolution_window(0);
	assert_eq!(resolution_window_0.expect("None value instead of 1st dispute window").round, 1);

	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(5));
	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(5));

	let resolution_window_1 = contract.get_active_resolution_window(0);
	assert_eq!(resolution_window_1.expect("None value instead of 2nd dispute window").round, 2);

	testing_env!(get_context(judge(), market_end_timestamp_ns()));
	contract.finalize_market(0, Some(0));


	let claimable_carol = contract.get_claimable(0, carol()) ;
	let claimable_alice = contract.get_claimable(0, alice()) ;

	let expected_claimable_carol = 10000000000000000 * to_dai(3) / to_dai(5) + to_dai(3);
	let expected_claimable_alice = 10000000000000000 * to_dai(2) / to_dai(5) + to_dai(2);

	assert_eq!(claimable_carol, expected_claimable_carol);
	assert_eq!(claimable_alice, expected_claimable_alice);
}

#[test]
fn test_crowdsourced_dispute_incorrect_resolution() {
	let mut contract = init_tests();
	testing_env!(get_context(bob(), current_block_timestamp()));
	contract.claim_fdai();

	contract.place_order(0, 0, to_dai(5), 50, None);
	contract.place_order(0, 1, to_dai(5), 50, None);
	
	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.resolute_market(0, Some(0), to_dai(3));
	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.claim_fdai();
	contract.resolute_market(0, Some(0), to_dai(2));
	
	let resolution_window_0 = contract.get_active_resolution_window(0);
	assert_eq!(resolution_window_0.expect("None value instead of 1st dispute window").round, 1);

	testing_env!(get_context(carol(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(5));
	testing_env!(get_context(alice(), market_end_timestamp_ns()));
	contract.dispute_market(0, Some(1), to_dai(5));

	let resolution_window_1 = contract.get_active_resolution_window(0);
	assert_eq!(resolution_window_1.expect("None value instead of 2nd dispute window").round, 2);

	testing_env!(get_context(judge(), market_end_timestamp_ns()));
	contract.finalize_market(0, Some(1));

	let total_res_fee: u128 = 10000000000000000;

	let claimable_carol = contract.get_claimable(0, carol()) ;
	let claimable_alice = contract.get_claimable(0, alice()) ;

	let expected_claimable_carol = to_dai(75) / 10 + total_res_fee / 2;
	let expected_claimable_alice = to_dai(75) / 10 + total_res_fee / 2;

	assert_eq!(claimable_carol, expected_claimable_carol);
	assert_eq!(claimable_alice, expected_claimable_alice);
}
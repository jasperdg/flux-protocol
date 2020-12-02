use super::*;

#[test]
fn test_franklins_market_test() {
    let (mut runtime, _root, accounts) = init_runtime_env();
    accounts[0].inc_allowance(&mut runtime, flux_protocol(), U128(to_dai(300))).expect("allowance couldn't be set");

    let market_tx_res = accounts[0].create_market(
        &mut runtime, 
        empty_string(),
        empty_string(), 
        2, 
        outcome_tags(0), 
        categories(), 
        U64(market_end_timestamp_ms()), 
        0, 
        0, 
        "test".to_string(),
        None
    ).unwrap();

    let place_order_1_tx_res = accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_shares(20)), 50, None, None);
    let place_order_2_tx_res = accounts[0].place_order(&mut runtime, U64(0), 0, U128(to_shares(50)), 55, None, None);
    let place_order_3_tx_res = accounts[0].place_order(&mut runtime, U64(0), 1, U128(to_shares(20)), 50, None, None);
    let place_order_4_tx_res = accounts[0].place_order(&mut runtime, U64(0), 0, U128(to_shares(20)), 50, None, None);
    let dynamic_sell_tx = accounts[0].dynamic_market_sell(&mut runtime, U64(0), 0, U128(to_shares(5)), 20, None);
    
    println!("place order 4 {:?}", place_order_4_tx_res);
    println!("place order 2 {:?}", place_order_2_tx_res);
    println!("Dynamic sell {:?}", dynamic_sell_tx);
}


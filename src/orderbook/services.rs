use near_sdk::{
    AccountId,
    collections::{
        UnorderedMap,
    },
};

use crate::orderbook::Orderbook;

/**
 * Checks all orderbooks and calculates how much money is still left in open orders
 * 
 * @return the amount of money in open orders and the total amount spent
 */
pub fn get_money_left_in_open_orders(account_id: &AccountId, orderbooks: &UnorderedMap<u8, Orderbook>) -> (u128, u128) {
    let mut money_in_open_orders = 0;
    let mut total_spent = 0;
    
    for(_, orderbook) in orderbooks.iter() {
        let account_data = match orderbook.account_data.get(account_id) {
            Some(user) => user,
            None => continue,
        };

        money_in_open_orders += account_data.to_spend - account_data.spent;
        total_spent += account_data.spent;
    }

    (money_in_open_orders, total_spent)
}
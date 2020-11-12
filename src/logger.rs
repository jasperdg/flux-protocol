use near_sdk::{
	env,
	AccountId,
	json_types::{
		U64,
		U128
	},
};
use serde_json::json;

use crate::order;
use crate::market;

type Order = order::Order;
type Market = market::Market;

pub fn log_order_filled(order: &Order, shares_to_fill: u128, market_id: u64, outcome: u8) {
	env::log(
		json!({
		"type": "order_filled".to_string(),
			"params": {
				"market_id": U64(market_id),
				"outcome": outcome,
				"order_id": U128(order.id),
				"account_id": order.creator,
				"shares_filling": U128(shares_to_fill),
				"filled": U128(order.filled),
				"price": order.price,
				"fill_price": order.price,
				"shares_filled": U128(order.shares_filled),
				"block_height": U64(env::block_index())
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_order_closed(order: &Order, market_id: u64, outcome: u8) {
	env::log(
		json!({
		"type": "order_closed".to_string(),
			"params": {
				"market_id": U64(market_id),
				"outcome": outcome,
				"order_id": U128(order.id),
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_market_creation(market: &Market, description: String, extra_info: String, outcome_tags: Vec<String>, categories: Vec<String>) {
	env::log(
		json!({
			"type": "market_creation".to_string(),
			"params": {
				"id": U64(market.id),
				"creator": market.creator,
				"description": description,
				"extra_info": extra_info,
				"outcomes": market.outcomes,
				"outcome_tags": outcome_tags,
				"categories": categories,
				"end_time": U64(market.end_time),
				"creator_fee_percentage": market.creator_fee_percentage,
				"resolution_fee_percentage": market.resolution_fee_percentage,
				"affiliate_fee_percentage": market.affiliate_fee_percentage,
				"api_source": market.api_source,
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_dispute_withdraw(market_id: u64, sender: &AccountId, dispute_round: u8, outcome: Option<u8>) {
	env::log(
		json!({
			"type": "withdrawn_unbounded_dispute_stake".to_string(),
			"params": {
				"market_id": U64(market_id),
				"sender": sender,
				"dispute_round": dispute_round,
				"outcome": outcome,
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_earnings_claimed(market_id: u64, sender: &AccountId, amount: u128) {
	env::log(
		json!({
			"type": "earnings_claimed".to_string(),
			"params": {
				"market_id": U64(market_id),
				"account_id": sender,
				"earned": U128(amount),
			}
		})
		.to_string()
		.as_bytes()
	);		
}

pub fn log_affiliate_earnings_claimed(sender: &AccountId, amount: u128) {
	env::log(
		json!({
			"type": "affiliate_earnings_claimed".to_string(),
			"params": {
				"account_id": sender,
				"earned": U128(amount),
			}
		})
		.to_string()
		.as_bytes()
	);		
}

pub fn log_market_resoluted(market_id: u64, sender: &AccountId, round: u8, staked: u128, outcome: u8) {
	env::log(
		json!({
			"type": "market_resoluted".to_string(),
			"params": {
				"market_id": U64(market_id),
				"sender": sender,
				"round": round,
				"staked": U128(staked),
				"outcome": outcome,
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_new_resolution_window(market_id: u64, round: u8, bond_size: u128, end_time: u64) {
	env::log(
		json!({
			"type": "new_resolution_window".to_string(),
			"params": {
				"market_id": U64(market_id),
				"round": round,
				"required_bond_size": U128(bond_size),
				"end_time": U64(end_time),	
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_staked_on_resolution(market_id: u64, sender: &AccountId, round: u8, staked: u128, outcome: u8) {
	env::log(
		json!({
			"type": "staked_on_resolution".to_string(),
			"params": {
				"market_id": U64(market_id),
				"sender": sender,
				"round": round,
				"staked": U128(staked),
				"outcome": outcome,
			}
		})
		.to_string()
		.as_bytes()
	)
}

pub fn log_resolution_disputed(market_id: u64, sender: &AccountId, round: u8, staked: u128, outcome: u8) {
	env::log(
		json!({
			"type": "resolution_disputed".to_string(),
			"params": {
				"market_id": U64(market_id),
				"sender": sender,
				"round": round,
				"staked": U128(staked),
				"outcome": outcome,
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_staked_on_dispute(market_id: u64, sender: &AccountId, round: u8, staked: u128, outcome: u8) {
	env::log(
		json!({
			"type": "staked_on_dispute".to_string(),
			"params": {
				"market_id": U64(market_id),
				"sender": sender,
				"round": round,
				"staked": U128(staked),
				"outcome": outcome,
			}
		})
		.to_string()
		.as_bytes()
	)
}

pub fn log_finalized_market(market_id: u64, winning_outcome: u8) {
	env::log(
		json!({
			"type": "market_finalized".to_string(),
			"params": {
				"market_id": U64(market_id),
				"winning_outcome": winning_outcome
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_update_user_balance(account_id: &AccountId, market_id: u64, outcome: u8, balance: u128, to_spend: u128, spent: u128) {
	env::log(
		json!({
			"type": "updated_user_balance".to_string(),
			"params": {
				"account_id": account_id,
				"market_id": U64(market_id),
				"outcome": outcome,
				"balance": U128(balance),
				"to_spend": U128(to_spend),
				"spent": U128(spent),
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_order_filled_at_placement(order: &Order, outcome: u8, fill_price: u128) {
	env::log(
		json!({
			"type": "order_filled_at_placement".to_string(),
			"params": {
				"order_id": U128(order.id),
				"market_id": U64(order.market_id),
				"account_id": order.creator, 
				"outcome": outcome, 
				"spend":  U128(order.spend),
				"shares":  U128(order.shares),
				"fill_price": U128(fill_price),
				"price": order.price,
				"filled": U128(order.filled), 
				"shares_filling": U128(order.shares_filled),
				"shares_filled": U128(order.shares_filled),
				"affiliate_account_id": order.affiliate_account_id,
				"block_height": U64(env::block_index())
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_order_placed(order: &Order, outcome: u8, fill_price: u128) {
	env::log(
		json!({
			"type": "order_placed".to_string(),
			"params": {
				"order_id": U128(order.id),
				"market_id": U64(order.market_id),
				"account_id": order.creator, 
				"outcome": outcome, 
				"spend":  U128(order.spend),
				"shares":  U128(order.shares),
				"fill_price": U128(fill_price),
				"price":  order.price,
				"filled": U128(order.filled), 
				"shares_filling": U128(order.shares_filled),
				"shares_filled": U128(order.shares_filled),
				"affiliate_account_id": order.affiliate_account_id,
				"block_height": U64(env::block_index())
			}
		})
		.to_string()
		.as_bytes()
	);
}
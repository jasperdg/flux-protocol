pub const TOKEN_DENOMINATION: u128 = 1_000_000_000_000_000_000;

/**
 * @notice A hardcoded amount of gas that's used for external transactions
 * @dev Currently set to a third of the maximum gas allowed to attach to a tx
 */
pub const SINGLE_CALL_GAS: u64 = 100_000_000_000_000;

pub const PERCENTAGE_PRECISION: u32 = 10_000;

/* Twelve hours in mili seconds */
pub const TWELVE_HOURS: u64 = 43_200_000;

/* A precision of 1e9 because it won't overflow with tokens < 100b total supply at 18 decimals */
pub const EARNINGS_PRECISION: u128 = 1_000_000_000;
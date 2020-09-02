#[cfg(feature = "wee_alloc")]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
#[allow(dead_code)]
mod logger;
mod order;
mod orderbook;
mod market;
mod markets;

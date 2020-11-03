#![warn(clippy::all)]
#![allow(dead_code, clippy::tabs_in_doc_comments, clippy::too_many_arguments)]
#[cfg(feature = "wee_alloc")]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[macro_use]
mod logger;
mod utils;
mod order;
mod orderbook;
mod market;
mod flux_protocol;

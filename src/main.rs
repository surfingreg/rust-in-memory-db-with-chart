/*

docker run -d \
--rm \
--name market_watcher \
-e RUST_LOG="info" \
-e COINBASE_URL=wss://ws-feed.pro.coinbase.com \
-e COIN_TRADE_LOG_DB_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin_test \
-e TRADE_SIZE_TARGET=0.01 \
market_watcher:latest

*/

// https://github.com/rust-lang/rust/issues/62924
// cargo +nightly run
#![feature(map_first_last)]

mod market;
mod market_structs;
mod ticker;
mod algorithm;
mod db;

fn main() {

	dotenv::dotenv().ok();
	env_logger::init();

	market::Market::start();
}

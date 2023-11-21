//! main/main.rs

/*

docker run -d \
--rm \
--name market_watcher \
-e RUST_LOG="info" \
-e COINBASE_URL=wss://ws-feed.pro.coinbase.rs.com \
-e COIN_TRADE_LOG_DB_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin_test \
-e COINBASE_URL=wss://ws-feed-public.sandbox.pro.coinbase.rs.com \
-e COIN_TRADE_LOG_DB_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin_test \
-e TRADE_SIZE_TARGET=0.01 \
market_watcher:latest

*/


use arrow_lib::arrow_db;
use common_lib::init::init;
use coinbase_websocket::websocket;
use common_lib::operator;

fn main() {
	// general logging stuff I always do
	init(env!("CARGO_MANIFEST_DIR"));

	// operator thread
	let tx_operator = operator::run();

	// database

	let _tx_db = arrow_db::run();

	// coinbase websocket thread
	let tx1 = tx_operator.clone();
	let h = websocket::run(tx1);
	h.join().unwrap();

}


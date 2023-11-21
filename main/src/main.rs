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

use std::time::Duration;
use crossbeam::channel::tick;
// use coinbase_websocket::coinbase::Coinbase::Ticker;
use common_lib::init::init;
use coinbase_websocket::websocket;
use common_lib::operator;
use common_lib::operator::Msg;
use coinbase_websocket::coinbase::Ticker;

fn main() {
	init(env!("CARGO_MANIFEST_DIR"));

	let (tx,rx) = operator::get_operator_comms();
	let _h = operator::run::<Ticker>(rx);

	let tx1 = tx.clone();
	let _h = websocket::run(tx1);

	// heartbeat
	let tx2=tx.clone();
	let h = std::thread::spawn(move ||{

		let ticker = tick(Duration::from_millis(2000));
		loop{
			tx2.send(Msg::Ping).unwrap();
			// tracing::debug!("[main] sending ping to operator");
			ticker.recv().unwrap();
		}

	});
	let _ = h.join();
}

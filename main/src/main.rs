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
use coinbase_websocket::websocket;
use common_lib::init::init;
use common_lib::operator;
use visual::http_server;

fn main() {

    // general logging stuff I always do
    init(env!("CARGO_MANIFEST_DIR"));

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        // .worker_threads(7)
        .on_thread_start(|| {})
        .on_thread_stop(|| {})
        .thread_name("actix")
        .enable_all()
        .build()
        .expect("Tokio runtime didn't start");

    // database thread
    let tx_db = arrow_db::run(tokio_runtime.handle().clone());

    // operator thread
    let tx_operator = operator::run(tx_db);

    // coinbase websocket thread
    let h = websocket::run(tx_operator.clone());

    let tx_operator2 = tx_operator.clone();
    http_server::run(tx_operator2, tokio_runtime.handle().clone());

    h.join().unwrap();
    // loop {};
}

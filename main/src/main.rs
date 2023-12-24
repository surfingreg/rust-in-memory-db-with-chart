//! main/chat_main

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
use coinbase_websocket::ws_inbound;
use common_lib::init::init;
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

    // websocket thread
    let h = ws_inbound::run(tx_db.clone());

    let tx_db2 = tx_db.clone();
    tokio_runtime.block_on(async {
        tracing::info!("[main] web server starting on http://127.0.0.1:8080");
        match http_server::run(tx_db2).await{
            Ok(_) => tracing::debug!("[main] web server started on http://127.0.0.1:8080"),
            Err(e) => tracing::debug!("[main] web server not started: {:?}", &e),
        }
    });

    h.join().unwrap();
    // loop {};
}

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

use std::error::Error;
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::sync::oneshot;
use coinbase_websocket::ws_inbound;
use common_lib::{ChartDataset, UniversalError, Msg, ProductId};
use common_lib::init::init;
use db::arrow_db;
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

    // outbound websocket
    let (server_tx, server_rx) = crossbeam_channel::unbounded::<ws_broadcast::command::Cmd>();
    let _h1 = std::thread::spawn(move || {

        std::thread::spawn(|| {
            let mut server = ws_broadcast::server::Server::new();
            server.run(server_rx);
        });
    });


    let tx_db2 = tx_db.clone();
    tokio_runtime.block_on(async {


        let tx_db3 = tx_db2.clone();
        tokio::spawn(async move {
            // todo: outbound websocket test; move somewhere else
            // todo: get the date of the last chart data sent and send


            // a long, long time ago...
            let most_recent_date: DateTime<Utc> = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("2023-12-24T04:08:00-00:00").unwrap());

            loop {
                // let since = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("2023-12-24T04:08:00-00:00").unwrap());
                match request_chart_multi_data_since(tx_db3.clone(), most_recent_date).await{
                    Ok(chart_vec) => {

                        // TODO: remove; just results in a lot of unnecessary javascript work; revisit with WASM
                        // most_recent_date = get_most_recent_date(chart_vec.clone());
                        // tracing::debug!("[main] chart_data total: {}", &json_str);

                        if let Ok(json_str) = serde_json::to_string::<Vec<ChartDataset>>(&chart_vec){

                            // tracing::debug!("[main] chart in ws_broadcast thread: {}", &json_str);

                            // server_tx.send(ws_broadcast::command::Cmd::Broadcast(format!("hello from test: {i}"))).unwrap();
                            server_tx.send(ws_broadcast::command::Cmd::Broadcast(json_str)).unwrap();

                        }

                    },
                    Err(_) => {}
                }

                std::thread::sleep(Duration::from_secs(1));
            }

        });









        tracing::info!("[main] web server starting on http://127.0.0.1:8080");
        match http_server::run(tx_db2).await{
            Ok(_) => tracing::debug!("[main] web server started on http://127.0.0.1:8080"),
            Err(e) => tracing::debug!("[main] web server not started: {:?}", &e),
        }
    });

    h.join().unwrap();
    // loop {};
}


/// TODO: extremely inefficient copying just to get the most recent date
fn _get_most_recent_date(chart_data: Vec<ChartDataset>) -> DateTime<Utc> {


    // todo: de-clone() all this
    let test: Vec<Vec<DateTime<Utc>>> = chart_data.iter().map(|a| a.data.iter().map(|b| b.x).collect()).collect();
    tracing::debug!("[main] chart_vec, test len: {}", &test.len());

    let test2: Vec<DateTime<Utc>> = test.iter().map(|x|x.clone()).flatten().collect::<Vec<DateTime<Utc>>>();

    tracing::debug!("[main] chart_vec, test2 len: {}", &test2.len());


    let max:DateTime<Utc> = match test2.iter().max(){
        Some(m) => m.clone(),
        None => DateTime::<Utc>::from(DateTime::parse_from_rfc3339("2023-12-01T04:08:00-00:00").unwrap())
    };

    tracing::debug!("[get_most_recent_date] {}", max);

    max

}

/// duplicated from visual
async fn request_chart_multi_data_since(tx_db: crossbeam_channel::Sender<Msg>, since: DateTime<Utc>) -> Result<Vec<ChartDataset>, Box<dyn Error>> {
    let (sender, rx) = oneshot::channel();
    match tx_db.send(Msg::RqstChartMultiSince {sender, filter_prod_id: vec![ProductId::BtcUsd], since}) {
        Ok(_)=> {
            let chart = rx.await?;
            Ok(chart)
        },
        Err(_)=> Err(Box::new(UniversalError::SendError))
    }
}
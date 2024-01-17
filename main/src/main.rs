//! main

use std::error::Error;
use std::time::Duration;
use chrono::{DateTime, Utc};
use crossbeam_channel::Sender;
use tokio::sync::oneshot;
use common_lib::{ChartDataset, UniversalError, DbMsg, ProductId};
use common_lib::cb_ticker::TickerSource;
use common_lib::init::init;
use db::arrow_db;
use visual::http_server;
use ws::connect::ConnectSource;
use ws_broadcast::command::Cmd;

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

    // run coinbase and alpaca threads
    let mut handles = vec!();
    handles.push(ws::connect::run(ConnectSource::Coinbase, tx_db.clone()));
    handles.push(ws::connect::run(ConnectSource::Alpaca, tx_db.clone()));

    // broadcast websocket
    let (server_tx, server_rx) = crossbeam_channel::unbounded::<ws_broadcast::command::Cmd>();
    let _h1 = std::thread::spawn(move || {
        std::thread::spawn(|| {
            let mut server = ws_broadcast::server::Server::new();
            server.run(server_rx);
        });
    });

    // tokio runtime herein
    let tx_db2 = tx_db.clone();
    tokio_runtime.block_on(async {
        // start 1-minute chart poll for websocket
        let tx_db3 = tx_db2.clone();
        spawn_chart_refresher(tx_db3, server_tx);

        // start web server
        tracing::info!("[main] web server starting on http://127.0.0.1:8080");
        match http_server::run(tx_db2).await{
            Ok(_) => tracing::debug!("[main] web server started on http://127.0.0.1:8080"),
            Err(e) => tracing::debug!("[main] web server not started: {:?}", &e),
        }
    });

    for h in handles {
        h.join().unwrap();
    }

}

/// todo: potentially only send data since last websocket broadcast (results in annoying client side
/// javascript so this current sends all chart data every second)
fn spawn_chart_refresher(tx_db: Sender<DbMsg>, server_tx: Sender<Cmd>) {
    tokio::spawn(async move {
        // a long, long time ago...
        let most_recent_date: DateTime<Utc> = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("2023-12-24T04:08:00-00:00").unwrap());
        loop {
            match request_chart_multi_data_since(tx_db.clone(), most_recent_date).await{
                Ok(chart_vec) => {
                    if let Ok(json_str) = serde_json::to_string::<Vec<ChartDataset>>(&chart_vec){
                        server_tx.send(Cmd::Broadcast(json_str)).unwrap();
                    }
                },
                Err(_) => {}
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

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
///
/// TODO: currently coinbase-specific
async fn request_chart_multi_data_since(tx_db: crossbeam_channel::Sender<DbMsg>, since: DateTime<Utc>) -> Result<Vec<ChartDataset>, Box<dyn Error>> {
    let (sender, rx) = oneshot::channel();
    match tx_db.send(DbMsg::RqstChartMultiSince { ticker_source: TickerSource::Coinbase, sender, filter_prod_id: vec![ProductId::BtcUsd], since}) {
        Ok(_)=> {
            let chart = rx.await?;
            Ok(chart)
        },
        Err(_)=> Err(Box::new(UniversalError::SendError))
    }
}
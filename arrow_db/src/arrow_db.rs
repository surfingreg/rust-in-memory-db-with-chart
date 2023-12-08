//! arrow_db.rs

use common_lib::cb_ticker::Ticker;
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::{Msg, VisualResultSet};
use crossbeam_channel::{unbounded, Sender};
use logger::event_log::{EventBook, EventLog};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::oneshot;

/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub fn run(tr: Handle) -> Sender<Msg> {
    tracing::debug!("[run]");
    let (tx, rx) = unbounded();
    let event_book = Arc::new(EventBook::new());
    let tx2 = tx.clone();
    std::thread::spawn(move || {
        tracing::debug!("[run] inside thread::spawn 0");
        let _ = start_heartbeat(tx2);
        loop {
            tracing::debug!("[run] inside loop");
            match rx.recv() {
                Ok(message) => {
                    tracing::debug!("[run] message: {:?}", &message);
                    let evt_book = event_book.clone();

                    // new thread to prevent processing blocking the websocket
                    process_message(message, &evt_book, tr.clone());
                }
                Err(e) => tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

/// Run this in another thread in response to a incoming websocket packet or http request.
fn process_message(message: Msg, evt_book: &EventBook, tr: Handle) {
    tracing::debug!("[arrow_db::process_message] msg:{:?}", &message);

    match message {
        Msg::Ping => tracing::debug!("[arrow_db] PING"),

        Msg::Post(ticker) => {
            post_ticker(&ticker, evt_book);
            run_calculations(&ticker.product_id.to_string(), evt_book);
        }

        Msg::GetChartForOne { key, sender } => {
            tracing::debug!("[process_message] Msg::VisualGetOne:{}", &key);
            visual_data_one(&key.to_string(), evt_book, sender, tr);
        }

        Msg::GetChartForAll { key, sender } => {
            tracing::debug!("[process_message] Msg::VisualGetOne:{}", &key);
            visual_data_all(&key.to_string(), evt_book, sender, tr);
        }

        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message),
    }
}

fn visual_data_all(key: &str, evt_book: &EventBook, sender: oneshot::Sender<VisualResultSet>, tr: Handle) {
    let evt_book_read_lock = evt_book.book.read().unwrap();
    let e_log_result = evt_book_read_lock.get(key);

    match e_log_result {
        Some(evt_log) => {
            let result_set = tr.block_on(async {
                match evt_log.query_sql_all().await {
                    Ok(df) => VisualResultSet {
                        data: Some(df),
                        error: None,
                    },
                    Err(e) => VisualResultSet {
                        data: None,
                        error: Some(format!("[visual_data_for_one] error: {:?}", &e)),
                    },
                }
            });

            let _ = sender.send(result_set);
        }
        None => {
            tracing::error!("[visual_data_for_one] event log for {} doesn't exist yet", key);
            sender
                .send(VisualResultSet {
                    data: None,
                    error: Some(format!("event log for {} doesn't exist yet", key)),
                })
                .unwrap();
        }
    }
}

fn visual_data_one(key: &str, evt_book: &EventBook, sender: oneshot::Sender<VisualResultSet>, tr: Handle) {
    let evt_book_read_lock = evt_book.book.read().unwrap();
    let e_log_result = evt_book_read_lock.get(key);

    match e_log_result {
        Some(evt_log) => {
            let result_set = tr.block_on(async {
                match evt_log.calc_with_sql().await {
                    Ok(df) => VisualResultSet {
                        data: Some(df),
                        error: None,
                    },
                    Err(e) => VisualResultSet {
                        data: None,
                        error: Some(format!("[visual_data_for_one] error: {:?}", &e)),
                    },
                }
            });

            let _ = sender.send(result_set);
        }
        None => {
            tracing::error!("[visual_data_for_one] event log for {} doesn't exist yet", key);
            sender
                .send(VisualResultSet {
                    data: None,
                    error: Some(format!("event log for {} doesn't exist yet", key)),
                })
                .unwrap();
        }
    }
}

/// locks the event book to get the event log for the new ticker
fn post_ticker(ticker: &Ticker, evt_book: &EventBook) {
    tracing::debug!("[arrow_db] POST {:?}", ticker);
    let _ = evt_book.push(&ticker.product_id.to_string(), ticker);
}

/// read lock
fn run_calculations(key: &str, evt_book: &EventBook) {
    let evt_book_read_lock = evt_book.book.read().unwrap();
    let evt_log: &EventLog = evt_book_read_lock.get(key).unwrap();
    evt_log.calc_curve_diff(4, 100);
    evt_log.calc_curve_diff(4, 300);
    // evt_log.calc_curve_diff(4, 500);
    // evt_log.calc_curve_diff(20, 100);
    // evt_log.calc_curve_diff(20, 300);
    // evt_log.calc_curve_diff(20, 500);
    println!("\n");

    // this take 1-5 milliseconds
    // let count = event_log.calc_with_sql().await.unwrap();
    //
    // if let Err(e) = count.show().await{
    //     tracing::error!("[process_message] sql_count error: {:?}", e);
    // }

    // only need read lock on the individual evt_book
    // same calculation without DataFusion/SQL takes .02 milliseconds (100x faster)
    // evt_book.book.get(&ticker.product_id.to_string()).unwrap().
}

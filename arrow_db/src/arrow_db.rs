//! arrow_db.rs

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, unbounded};
use common_lib::cb_ticker::Ticker;
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;
use logger::event_log::{EventBook, EventLog};


/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub async fn run() -> Sender<Msg> {
    let (tx, rx) = unbounded();

    let event_book = Arc::new(EventBook::new());

    let tx2 = tx.clone();
    tokio::spawn(async move {
        let _ = start_heartbeat(tx2);

        loop{
            match rx.recv(){
                Ok(message)=> {
                    let evt_book = event_book.clone();

                    // new thread to prevent processing blocking the websocket
                    tokio::spawn(async move {
                        process_message(message, &evt_book).await;
                    });
                },
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

/// Run this in another thread in response to a websocket packet.
async fn process_message(message:Msg, evt_book: &EventBook){
    match message{
        Msg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        Msg::Post(ticker)=>{
            post_ticker(&ticker, evt_book);
            post_calculations(&ticker.product_id.to_string(), evt_book);
        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}

/// locks the event book to get the event log for the new ticker
fn post_ticker(ticker:&Ticker, evt_book: &EventBook){

    tracing::debug!("[arrow_db] POST {:?}", ticker);
    let _ = evt_book.push(&ticker.product_id.to_string(), ticker);

}

/// read lock
fn post_calculations(key: &str, evt_book: &EventBook){

    let evt_book_read_lock = evt_book.book.read().unwrap();

    // return a reference to the hashmap key's value
    let evt_log: &EventLog = evt_book_read_lock.get(key).unwrap();

    evt_log.calc_curve_diff(4, 100);
    evt_log.calc_curve_diff(4, 300);
    evt_log.calc_curve_diff(4, 500);
    evt_log.calc_curve_diff(20, 100);
    evt_log.calc_curve_diff(20, 300);
    evt_log.calc_curve_diff(20, 500);
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
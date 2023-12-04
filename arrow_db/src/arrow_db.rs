//! arrow_db.rs

use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender, unbounded};
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;
use logger::event_log::EventLog;


/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub async fn run() -> Sender<Msg> {
    let (tx, rx) = unbounded();

    let event_log = Arc::new(Mutex::new(EventLog::new()));

    let tx2 = tx.clone();
    tokio::spawn(async move {
        let _ = start_heartbeat(tx2);

        loop{
            match rx.recv(){
                Ok(message)=> {
                    let mut event_log = event_log.clone();

                    // new thread to prevent processing blocking the websocket
                    tokio::spawn(async move {
                        process_message(message, &mut event_log).await;
                    });
                },
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

/// Run this in another thread in response to a websocket packet.
async fn process_message(message:Msg, event_log: &mut Arc<Mutex<EventLog>>){
    match message{
        Msg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        Msg::Post(ticker)=>{
            let mut event_log = event_log.lock().unwrap();
            let _ = event_log.push(&ticker);
            tracing::debug!("[arrow_db] POST {:?}", &ticker);
            // this take 1-5 milliseconds
            // let count = event_log.calc_with_sql().await.unwrap();
            //
            // if let Err(e) = count.show().await{
            //     tracing::error!("[process_message] sql_count error: {:?}", e);
            // }

            // same calculation without DataFusion/SQL takes .02 milliseconds (100x faster)
            event_log.calc_curve_diff(4, 100);
            event_log.calc_curve_diff(4, 300);
            event_log.calc_curve_diff(4, 500);
            event_log.calc_curve_diff(20, 100);
            event_log.calc_curve_diff(20, 300);
            event_log.calc_curve_diff(20
                                      , 500);
            println!("\n");

        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}

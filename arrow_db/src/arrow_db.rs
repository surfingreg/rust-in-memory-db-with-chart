//! arrow_db.rs

use crossbeam_channel::{Sender, unbounded};
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;
use logger::event_log::EventLog;


/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub async fn run() -> Sender<Msg> {
    let (tx, rx) = unbounded();

    let mut event_log = EventLog::new();

    let tx2 = tx.clone();
    tokio::spawn(async move {
        let _ = start_heartbeat(tx2);

        loop{
            match rx.recv(){
                Ok(message)=> {
                    process_message(message, &mut event_log).await;
                },
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

async fn process_message(message:Msg, event_log: &mut EventLog){
    match message{
        Msg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        Msg::Post(ticker)=>{
            tracing::debug!("[arrow_db] POST {:?}", &ticker);

            let _ = event_log.push(&ticker);

            // event_log._print_record_batch();
            let count = event_log.sql_count_all().await.unwrap();
            count.show().await;


        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}

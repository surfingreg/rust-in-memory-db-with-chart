//! arrow_db.rs

use crossbeam_channel::{Sender, unbounded};
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;

/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub fn run() -> Sender<Msg> {
    let (tx, rx) = unbounded();
    let tx2 = tx.clone();
    std::thread::spawn( move ||{
        let _ = start_heartbeat(tx2);
        loop{
            match rx.recv(){
                Ok(message)=> process_message(message),
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

fn process_message(message:Msg){
    match message{
        Msg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        Msg::Post(msg)=>{
            tracing::debug!("[arrow_db] POST {:?}", &msg);
            // tracing::debug!("[Coinbase::Ticker] {:?}", &t);
        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}

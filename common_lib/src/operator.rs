//! operator.rs

use std::fmt::Debug;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender, unbounded};

#[derive(Debug)]
pub enum Msg<T:Debug> {
    // Post(T),
    Log(T),
    // PostAndLog(T),
    Ping,
    Pong,
    Start,
    Stop,
}

pub fn get_operator_comms<Msg>() -> (Sender<Msg>, Receiver<Msg>){
    unbounded()
}

/// spawn a thread to listen for messages
pub fn run<T:Debug+Send+Sync+'static>(rx:Receiver<Msg<T>>) -> JoinHandle<()> {
    let h = std::thread::spawn(move ||{
        loop{
            match rx.recv(){
                Ok(message)=> process_message(&message),
                Err(e)=> tracing::debug!("[operator] error {:?}", &e),
            }
        }
    });
    h

}

fn process_message<T:Debug>(message:&Msg<T>){
    match message{
        Msg::Ping => {
            tracing::debug!("[operator] PING");
        },
        Msg::Log(msg)=>{
            tracing::debug!("[operator] LOG {:?}", &msg);
            // tracing::debug!("[Coinbase::Ticker] {:?}", &t);
        },
        _ => {
            tracing::debug!("[operator] {:?} UNKNOWN ", &message);

        }
    }

}
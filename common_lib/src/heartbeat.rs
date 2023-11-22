//! heartbeat.rs

use std::thread::JoinHandle;
use std::time::Duration;
use crossbeam_channel::{Sender, tick};
use crate::operator::Msg;

const PING_MS:u64 = 10000;

pub fn start_heartbeat(tx:Sender<Msg>) -> JoinHandle<()> {
    std::thread::spawn(move ||{
        let ticker = tick(Duration::from_millis(PING_MS));
        loop{
            tx.send(Msg::Ping).unwrap();
            ticker.recv().unwrap();
        }
    })
}
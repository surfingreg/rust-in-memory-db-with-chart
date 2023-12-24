//! lib.rs
//!
//!

pub mod client;
pub mod server;
pub mod command;

pub const TEST_SHUTDOWN_TIMER_SEC:u64 = 1;
pub const SERVER_READ_POLL_MS:u64 = 1;
pub const CLIENT_READ_POLL_MS:u64 = 1;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use common_lib::init;
    use crate::{client, init, server};
    use crate::command::Cmd;
    use crate::server::Server;

    /// client PINGs server once per second; server replies with PONG
    /// Client gets told to shutdown after four seconds
    #[test]
    fn simulated_main() {
        init::init("no-block-websocket");
        tracing::debug!("[main]");


        let (server_tx, server_rx) = crossbeam_channel::unbounded::<Cmd>();

        let h1 = std::thread::spawn(|| {
            let mut server = Server::new();
            server.run(server_rx);
        });

        // let (client_tx, client_rx) = crossbeam_channel::unbounded::<Cmd>();
        // let h2 = std::thread::spawn(|| {
        //     let mut c = client::Client::new(client_rx);
        //     c.run();
        // });
        
        // test
        // client_tx.send(Cmd::StartPing).unwrap();
        // std::thread::sleep(Duration::from_secs(4));
        // client_tx.send(Cmd::Shutdown).unwrap();


        for i in 0..100 {
            server_tx.send(Cmd::Broadcast(format!("hello from test: {i}"))).unwrap();
            std::thread::sleep(Duration::from_secs(1));
        }

        h1.join().unwrap();
        // h2.join().unwrap();
    }
}
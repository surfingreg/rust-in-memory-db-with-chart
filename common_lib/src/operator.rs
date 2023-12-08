// //! operator.rs
// //! decommissioned since it only forwards to the database thread anyway
//
// /// spawn a thread to listen for messages; return a way to send it crossbeam messages
// pub fn run(tx_db: Sender<Msg>) -> Sender<Msg> {
//     let (tx, rx) = unbounded();
//     let tx2 = tx.clone();
//     std::thread::spawn(move || loop {
//         match rx.recv() {
//             Ok(message) => process_message(message, tx_db.clone()),
//             Err(e) => tracing::debug!("[operator] error {:?}", &e),
//         }
//     });
//     let tx3 = tx2.clone();
//     let _h = start_heartbeat(tx3);
//     tx
// }
//
// /// TODO: get rid of this unnecessary indirection
// fn process_message(message: Msg, tx_db: Sender<Msg>) {
//     match message {
//         Msg::Ping => tracing::debug!("[operator] PING"),
//         Msg::Post(msg) => tx_db.send(Msg::Post(msg)).unwrap(),
//         Msg::GetChartForOne { key, sender } => {
//             tracing::debug!("[operator] GetChartForOne");
//             match tx_db.send(Msg::GetChartForOne { key, sender }) {
//                 Ok(_) => tracing::debug!("[operator] GetChartForOne sent to tx_db"),
//                 Err(e) => tracing::error!("[operator] GetChartForOne send error: {:?}", &e),
//             }
//         }
//         Msg::GetChartForAll { key, sender } => {
//             tracing::debug!("[operator] GetChartForOne");
//             match tx_db.send(Msg::GetChartForAll { key, sender }) {
//                 Ok(_) => tracing::debug!("[operator] GetChartForOne sent to tx_db"),
//                 Err(e) => tracing::error!("[operator] GetChartForOne send error: {:?}", &e),
//             }
//         },
//         Msg::RequestChart {chart_type, sender} => {
//             match chart_type{
//                 ChartType::Basic=>{
//                     match tx_db.send(Msg::RequestChart { sender }) {
//                         Ok(_) => tracing::debug!("[RequestChart] RequestChart sent to tx_db"),
//                         Err(e) => tracing::error!("[RequestChart] RequestChart send error: {:?}", &e),
//                     }
//
//                 },
//                 ChartType::Test=>{
//                     match tx_db.send(Msg::RequestChartTest { sender }) {
//                         Ok(_) => tracing::debug!("[RequestChart] RequestChart sent to tx_db"),
//                         Err(e) => tracing::error!("[RequestChart] RequestChart send error: {:?}", &e),
//                     }
//
//                 }
//             }
//
//         },
//         _ => tracing::debug!("[operator] {:?} UNKNOWN ", &message),
//     }
// }

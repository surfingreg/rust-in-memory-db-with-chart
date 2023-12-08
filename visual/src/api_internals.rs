//! api_internals.rs



use actix_web::{Responder, web};
use crossbeam_channel::Sender;
use common_lib::cb_ticker::ProductId;
use common_lib::operator::{Msg, VisualResultSet};

// /// clear the mechanics of sending a cross-thread message out of the HTTP handler
// pub async fn request_index_data(tx: web::Data<Sender<Msg>>) -> impl Responder{
//     let (tx_web, rx_web) = tokio::sync::oneshot::channel::<VisualResultSet>();
//     match tx.send(Msg::GetChartForAll {
//         key: ProductId::BtcUsd,
//         sender: tx_web,
//     }) {
//         Ok(_) => match rx_web.await {
//             Ok(visual_result_set) => {
//                 use datafusion::arrow::util::pretty::pretty_format_batches;
//
//                 if let Some(df) = visual_result_set.data {
//                     pretty_format_batches(&df.collect().await.unwrap())
//                         .unwrap()
//                         .to_string()
//                 } else {
//                     "[index] no dataframe returned".to_string()
//                 }
//             }
//             Err(e) => {
//                 tracing::error!("[index] receive error: {:?}", &e);
//                 format!("[index] send error: {:?}", &e)
//             }
//         },
//         Err(e) => {
//             tracing::error!("[index] send error: {:?}", &e);
//             format!("[index] send error: {:?}", &e)
//         }
//     }
// }


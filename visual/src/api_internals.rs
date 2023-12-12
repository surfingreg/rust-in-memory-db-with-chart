//! api_internals.rs

use actix_web::{Responder, web};
use crossbeam_channel::Sender;
use common_lib::Msg;
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::dataframe::DataFrame;

/// clear the mechanics of sending a cross-thread message out of the HTTP handler
pub async fn request_raw_data(tx: web::Data<Sender<Msg>>) -> impl Responder{
    let (tx_web, rx_web) = tokio::sync::oneshot::channel::<DataFrame>();

    match tx.send(Msg::RqstRaw { sender: tx_web}) {
        Ok(_) => match rx_web.await {
            Ok(df) => {
                pretty_format_batches(&df.collect().await.unwrap())
                    .unwrap()
                    .to_string()
            }
            Err(e) => {
                tracing::error!("[request_raw_data] receive error: {:?}", &e);
                format!("[request_raw_data] send error: {:?}", &e)
            }
        },
        Err(e) => {
            tracing::error!("[request_raw_data] send error: {:?}", &e);
            format!("[request_raw_data] send error: {:?}", &e)
        }
    }
}


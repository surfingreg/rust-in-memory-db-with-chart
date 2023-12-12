//! api_internals.rs

use actix_web::{Responder, web};
use crossbeam_channel::Sender;
use common_lib::Msg;
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::dataframe::DataFrame;

/// clear the mechanics of sending a cross-thread message out of the HTTP handler
pub async fn request_index_data(tx: web::Data<Sender<Msg>>) -> impl Responder{
    let (tx_web, rx_web) = tokio::sync::oneshot::channel::<DataFrame>();

    match tx.send(Msg::GetRaw { sender: tx_web}) {
        Ok(_) => match rx_web.await {

            Ok(df) => {

                // if let Some(df) = data {
                    pretty_format_batches(&df.collect().await.unwrap())
                        .unwrap()
                        .to_string()
                // } else {
                //     "[index] no dataframe returned".to_string()
                // }
            }
            Err(e) => {
                tracing::error!("[index] receive error: {:?}", &e);
                format!("[index] send error: {:?}", &e)
            }
        },
        Err(e) => {
            tracing::error!("[index] send error: {:?}", &e);
            format!("[index] send error: {:?}", &e)
        }
    }
}


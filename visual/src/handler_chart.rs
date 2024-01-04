//! analysis
//!

use std::error::Error;
use actix_web::{web, HttpResponse, Responder};
use crossbeam_channel::Sender;
use handlebars::Handlebars;
use serde_json::json;
use tokio::sync::oneshot;
use common_lib::{ChartDataset, UniversalError, Msg, ProductId};

const CHART_MULTI_NAME:&str = "chart_multi";

/**************** HTTP handlers ********************************************************************/

/// GET '/raw'
pub async fn get_raw(tx: web::Data<Sender<Msg>>) -> impl Responder {
    request_raw_data(tx).await
}

pub async fn redirect_home() -> HttpResponse {
    tracing::debug!("[redirect_home]");
    // redirect to home via 302
    // https://docs.rs/actix-web/latest/actix_web/http/struct.StatusCode.html#associatedconstant.FOUND
    // https://www.rfc-editor.org/rfc/rfc7231#section-6.4.3
    HttpResponse::Found()
        .append_header(("location", "/"))
        .append_header(("Cache-Control", "no-store"))
        .finish()
}

/// show multiple datasets on the same chart, regardless of x-axis count
pub async fn present_chart_multi_line(tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
    // tracing::debug!("[present_chart]");
    let tx_db = tx_db.into_inner().as_ref().clone();

    match request_chart_multi_data(tx_db).await {
        Ok(vec_chart2)=> {
            // tracing::debug!("[present_chart_multi_line] data: {:?}", &vec_chart2);
            match serde_json::to_string(&vec_chart2) {
                Ok(data_vec_json) =>{
                    let data = json!({
                        "title": "",
                        "parent": "base0",
                        "is_logged_in": true,
                        "chart_title": "Coinbase",
                        "data_vec": data_vec_json,
                    });
                    let body = hb.render(CHART_MULTI_NAME, &data).unwrap();
                    HttpResponse::Ok().append_header(("cache-control", "no-store")).body(body)
                },
                Err(e)=>{
                    tracing::error!("[present_chart_multi_line] json serialization error: {:?}", e);
                    redirect_home().await
                }
            }
        }
        Err(e)=>{
            tracing::error!("[present_chart_multi_line] database error getting chart data: {:?}", e);
            redirect_home().await
        }
    }
}



/**************** Message Passing ******************************************************************/


/// Ask the database for data for the chart
/// TODO: add an enum for the kind of chart to fetch
///
/// TODO: Currently selects ALL product_id
async fn request_chart_multi_data(tx_db: Sender<Msg>) -> Result<Vec<ChartDataset>, Box<dyn Error>> {
    let (sender, rx) = oneshot::channel();

    use strum::IntoEnumIterator;
    let prods:Vec<ProductId> = ProductId::iter().collect();

    // send a list of product IDs we want to filter on; here it's 'select *'

    match tx_db.send(Msg::RqstChartMulti {sender, filter_prod_id: prods }) {
        Ok(_)=> {
            let chart = rx.await?;
            Ok(chart)
        },
        Err(_)=> Err(Box::new(UniversalError::SendError))
    }
}


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

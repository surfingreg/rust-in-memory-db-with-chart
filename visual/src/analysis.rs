//! analysis
//!

use actix_web::{web, HttpResponse};
use crossbeam_channel::Sender;
use handlebars::Handlebars;
use serde_json::json;
use tokio::sync::oneshot;
use common_lib::{Chart, ChartAsJson, ChartType, KitchenSinkError, Msg};


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


/// TODO: is converting to json even necessary here?
pub async fn present_chart_rust(tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
    tracing::debug!("[present_chart]");
    let tx_db = tx_db.into_inner().as_ref().clone();
    let chart_0_json_result = request_chart_rust(tx_db).await;

    match chart_0_json_result {
        Ok(chart_0)=> {
            let chart_0_columns = serde_json::to_string(&chart_0.columns).unwrap();
            let chart_0_data = serde_json::to_string(&chart_0.chart_data).unwrap();

            let data = json!({
                "title": "Analysis",
                "parent": "base0",
                "is_logged_in": true,
                // "session_username": &session_username,
                "chart_0_columns": chart_0_columns,
                "chart_0_data": chart_0_data,
            });
            let body = hb.render("analysis", &data).unwrap();
            HttpResponse::Ok().append_header(("cache-control", "no-store")).body(body)
        },
        Err(e)=>{
            // TODO: figure out how to do a match with two results
            tracing::error!("[present_chart_rust] database error getting chart data: {:?}", e);
            redirect_home().await
        }
    }
}

/// Ask the database for data for the chart
/// TODO: add an enum for the kind of chart to fetch
async fn request_chart_rust(tx_db:Sender<Msg>) -> Result<Chart, KitchenSinkError> {
    let (sender, rx) = oneshot::channel();
    match tx_db.send(Msg::RequestChartRust{sender}) {
        Ok(_)=> {
            match rx.await {
                Ok(chart) => Ok(chart),
                Err(e) => {
                    tracing::error!("[request_chart_rust] {:?}", &e);
                    Err(KitchenSinkError::RecvError)
                },
            }
        },
        Err(_)=>Err(KitchenSinkError::SendError),
    }
}



//
// pub async fn _present_chart(tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
//     tracing::debug!("[present_chart]");
//     let tx_db = tx_db.into_inner().as_ref().clone();
//
//     // TODO: this is dumb, convert to json, then convert it back to struct then immediate back to json????
//     // get data in json format (totally unnecessary)
//     let chart_0_json_result = _request_chart_json(ChartType::BasicAsJson, tx_db).await;
//     if chart_0_json_result.is_ok() {
//
//         // struct back to json for handlebars
//         let chart_0: ChartAsJson = serde_json::from_value::<ChartAsJson>(chart_0_json_result.unwrap()).unwrap();
//         let chart_0_columns = serde_json::to_string(&chart_0.columns).unwrap();
//         let chart_0_data = serde_json::to_string(&chart_0.chart_data).unwrap();
//
//         let data = json!({
//                 "title": "Analysis",
//                 "parent": "base0",
//                 "is_logged_in": true,
//                 // "session_username": &session_username,
//                 "chart_0_columns": chart_0_columns,
//                 "chart_0_data": chart_0_data,
//             });
//         let body = hb.render("analysis", &data).unwrap();
//         HttpResponse::Ok().append_header(("cache-control", "no-store")).body(body)
//     } else {
//         // TODO: figure out how to do a match with two results
//         tracing::error!("[_present_chart] database error getting chart data");
//         redirect_home().await
//     }
// }

//
// /// Ask the database for data for the chart
// /// TODO: add an enum for the kind of chart to fetch
// async fn _request_chart_json(chart_type: ChartType, tx_db:Sender<Msg>) -> Result<serde_json::Value, KitchenSinkError> {
//     let (sender, rx) = oneshot::channel::<serde_json::Value>();
//     match tx_db.send(Msg::RequestChartJson{chart_type, sender}) {
//         Ok(_)=> {
//             match rx.await {
//                 Ok(chart) => Ok(chart),
//                 Err(_) => Err(KitchenSinkError::RecvError),
//             }
//         },
//         Err(_)=>Err(KitchenSinkError::RecvError),
//     }
// }



/*
/// Ask the database for data for the chart
/// TODO: add an enum for the kind of chart to fetch
async fn request_chart_test(chart_type: ChartType, tx_db:Sender<Msg>) -> Result<Chart, KitchenSinkError> {
    let (sender, rx) = oneshot::channel::<Chart>();
    match tx_db.send(Msg::RequestChartJson {chart_type: chart_type, sender: sender}) {
        Ok(_)=> {
            match rx.await {
                Ok(chart) => Ok(chart),
                Err(_) => Err(KitchenSinkError::ChannelError),
            }
        },
        Err(_)=>Err(KitchenSinkError::ChannelError),
    }
}

/// GET /profit
/// print a table of stocks P/L
pub async fn present_chart_test(tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
    tracing::debug!("[get_profit]");

    // logged in?
    // if let Ok(Some(session_username)) = session.get::<String>(SESSION_USERNAME) {

        // tracing::debug!("[get_analysis] logged in with session id: {}", &session_username);

        let tx_db = tx_db.into_inner().as_ref().clone();
        // let tx_db_1 = tx_db.clone();
        // let tx_db_2 = tx_db.clone();
        // let tx_db_3 = tx_db.clone();
        // let tx_db_4 = tx_db.clone();

        // get both charts' data from database
        let chart_0_result = request_chart_test(ChartType::Test, tx_db).await;
        // let chart_1_result = analysis_chart_avg_profit(tx_db_1).await;
        // let chart_2_result = chart_profit_daily(tx_db_2).await;
        // let chart_3_result = chart_orders_daily(tx_db_3).await;
        // let chart_4_result = chart_value_traded(tx_db_4).await;

        if chart_0_result.is_ok() /*&& chart_1_result.is_ok() && chart_2_result.is_ok() && chart_3_result.is_ok() && chart_4_result.is_ok()*/ {

            tracing::debug!("[get_analysis] all chart data ok");

            let chart_0 = chart_0_result;
            // let chart_0 = chart_0_result.unwrap();
            // let chart_1 = chart_1_result.unwrap();
            // let chart_2 = chart_2_result.unwrap();
            // let chart_3 = chart_3_result.unwrap();
            // let chart_4 = chart_4_result.unwrap();

            // tracing::debug!("[get_analysis] db result: {:?}", &avg_profit);
            // TODO: remove unwrap
            let chart_0 = chart_0.unwrap();
            let chart_0_columns = serde_json::to_string(&chart_0.columns).unwrap();
            let chart_0_data = serde_json::to_string(&chart_0.chart_data).unwrap();

            // let chart_1_columns = serde_json::to_string(&chart_1.columns).unwrap();
            // let chart_1_data = serde_json::to_string(&chart_1.chart_data).unwrap();
            //
            // let chart_2_columns = serde_json::to_string(&chart_2.columns).unwrap();
            // let chart_2_data = serde_json::to_string(&chart_2.chart_data).unwrap();
            //
            // let chart_3_columns = serde_json::to_string(&chart_3.columns).unwrap();
            // let chart_3_data = serde_json::to_string(&chart_3.chart_data).unwrap();
            //
            // let chart_4_columns = serde_json::to_string(&chart_4.columns).unwrap();
            // let chart_4_data = serde_json::to_string(&chart_4.chart_data).unwrap();

            let data = json!({
                "title": "Analysis",
                "parent": "base0",
                "is_logged_in": true,
                // "session_username": &session_username,
                "chart_0_columns": chart_0_columns,
                "chart_0_data": chart_0_data,
                // "chart_1_columns": chart_1_columns,
                // "chart_1_data": chart_1_data,
                // "chart_2_columns": chart_2_columns,
                // "chart_2_data": chart_2_data,
                // "chart_3_columns": chart_3_columns,
                // "chart_3_data": chart_3_data,
                // "chart_4_columns": chart_4_columns,
                // "chart_4_data": chart_4_data,
            });

            let body = hb.render("analysis", &data).unwrap();
            HttpResponse::Ok().append_header(("cache-control", "no-store")).body(body)

        } else {
            // TODO: figure out how to do a match with two results
            tracing::error!("[get_analysis] database error getting chart data");
            redirect_home().await
        }
    // } else {
    //     tracing::info!("[get_analysis] not logged in redirecting home");
    //     redirect_home().await
    // }
}
*/
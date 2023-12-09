//! analysis
//!

use actix_web::{web, HttpResponse};
use crossbeam_channel::Sender;
use datafusion::prelude::DataFrame;
use handlebars::Handlebars;
use serde_json::json;
use tokio::sync::oneshot;
use common_lib::{Chart, ChartType, KitchenSinkError, Msg};


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


pub async fn present_chart(tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
    tracing::debug!("[present_chart]");
    let tx_db = tx_db.into_inner().as_ref().clone();
    let chart_0_result = request_chart(ChartType::Basic, tx_db).await;
    if chart_0_result.is_ok() {

        // todo: convert dataframe to json
        let chart_0 = convert_data_from_database(chart_0_result.unwrap()).await;

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
    } else {
        // TODO: figure out how to do a match with two results
        tracing::error!("[get_analysis] database error getting chart data");
        redirect_home().await
    }
}

async fn convert_data_from_database(json :serde_json::Value) -> Chart {


    tracing::info!("[convert_data_from_database] json: {:?}", &json);

    // &df.show().await.unwrap();
    // let batches = &df.collect().await.unwrap();


    let test_chart:Chart = serde_json::from_value(json).unwrap();
    tracing::info!("[convert_data_from_database] test_chart: {:?}", test_chart);

    test_chart


    // serde_json::from_str::<Chart>(r#"
    //     {
    //         "columns":["2023-08-14", "2023-08-15", "2023-08-16", "2023-08-17", "2023-08-18", "2023-08-21", "2023-08-22", "2023-08-23", "2023-08-24", "2023-08-25", "2023-08-28", "2023-08-29", "2023-08-30", "2023-08-31", "2023-09-01", "2023-09-05", "2023-09-06", "2023-09-07", "2023-09-08", "2023-09-11", "2023-09-12", "2023-09-13", "2023-09-14", "2023-09-15", "2023-09-18", "2023-09-19", "2023-09-20", "2023-09-21", "2023-09-22", "2023-09-25", "2023-09-26", "2023-09-27", "2023-09-28", "2023-09-29", "2023-10-02", "2023-10-03", "2023-10-04", "2023-10-05", "2023-10-06", "2023-10-09", "2023-10-10", "2023-10-11", "2023-10-16", "2023-10-17", "2023-10-18", "2023-10-19", "2023-10-20", "2023-10-23", "2023-10-24", "2023-10-25", "2023-10-26", "2023-10-27", "2023-10-30", "2023-10-31", "2023-11-03", "2023-11-06", "2023-11-07", "2023-11-08", "2023-11-09", "2023-11-10", "2023-11-13", "2023-11-14", "2023-11-15", "2023-11-17", "2023-11-20", "2023-11-27", "2023-11-28", "2023-11-29", "2023-11-30", "2023-12-01"],
    //         "chart_data":[
    //             {
    //                 "key" : "aapl",
    //                 "val" : [0.0177, 0.0650, -0.0260, -0.0408, -0.0055, -0.0079, 0.0073, 0.2684, 0.0, -0.0078, 0.3213, 0.5855, 0.3809, 0.5024, 0.4700, 0.2817, 0.0, -24.2500, -0.2045, 0.1322, -0.3000, 0.0, -0.6600, 0.0, 0.0283, -0.1167, -7.1800, -0.3925, -0.2133, 0.2886, -7.1500, 0.4960, 0.1594, 0.2790, 0.1800, 0.1050, -0.4800, 0.1321, 0.1048, 0.2733, -0.3864, 0.0, -0.4240, 0.0, 0.0, -0.2371, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.2478, 0.7418, 0.2986, 0.3938, -0.4300, 0.3300, -2.9533, 1.3705, 0.1983, 0.3046, 16.9000, 0.3982, -0.6790, -3.1500, -0.7750, 0.2633]}, {"key" : "amzn", "val" : [0.0372, -0.0300, -0.0537, -0.0142, -0.0062, 0.0027, -0.0205, -0.0692, -0.4325, 0.4794, 0.1750, 0.3212, 0.2354, 0.5859, -0.4818, 0.0310, 0.1150, 0.1667, 0.2677, 0.2070, 0.0400, 0.4937, 0.0, 0.0, 0.3656, -5.2150, -0.1867, -0.9789, -0.0371, 0.3600, -0.3050, -1.6250, 0.3800, 1.3862, -0.0506, 0.0, -11.9900, -0.0600, 0.1829, -0.6379, 0.2578, 0.0, -0.0015, -14.5000, -1.6014, 0.5000, 0.0, 0.0, 0.0, -59.0400, 0.0, 3.0400, 0.4917, 0.2750, 8.7100, -0.2900, 0.2327, -0.2364, -0.6225, 0.3926, 0.2190, 1.7533, -27.0400, -0.2117, 2.7100, -0.1505, 0.0414, 0.9308, 1.6255, 0.0579]}, {"key" : "bbai", "val" : [-0.0100, 0.0, -0.0126, -0.0100, -0.0100, -0.0300, 0.0050, -0.0100, -0.0067, -0.1950, 0.0100, 0.0200, -0.0138, -0.0450, 0.0180, -0.0233, 0.0100, -0.9000, 0.0350, 0.0033, 0.0433, -0.0400, -0.0400, 0.0, 0.0050, 0.0, -0.1633, -0.0200, -0.0050, -0.0150, -0.0300, -0.0200, -0.0050, -0.0267, -0.5267, -0.0200, 0.0133, 0.0150, -0.0200, -0.0050, 0.0, -0.0900, -0.0050, 0.0600, -0.0150, 0.0033, -0.0400, -0.0267, 0.0050, 0.0, -0.0500, -0.0200, 0.0, -0.0133, 0.0, -0.0367, -0.0838, -0.0125, 0.0100, -0.0800, 0.0963, 0.0500, -0.0020, 0.0, 0.0, 0.0100, -0.3000, -0.0067, -0.0300, 0.0025]}, {"key" : "dis", "val" : [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1877, 0.0825, 0.4193, 0.0929, -2.4233, -0.0408, 0.0, 0.0, -0.1900, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.9900, 0.0250, 0.0, 0.1389, 0.2400, -0.4067, -0.3807, 0.1000, 0.1275, 0.0600, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6700, -0.6650, 0.2100, -0.2889, 2.1167, 0.7800, -0.0548, 0.4453, 0.5746, -0.5840, 0.0300, -1.1167, -3.5920, -0.3494, -0.3140, -1.0000]}, {"key" : "nio", "val" : [0.0053, -0.2600, -0.0392, -0.0076, -0.0348, -0.0050, -0.0113, -0.0214, -0.0675, 0.3100, 0.0500, 0.1420, -0.0767, -0.3189, 0.2189, -0.0175, 0.0064, -0.5100, -0.0867, 0.0207, -0.0971, -0.9750, 0.1200, 0.0, -0.0117, -0.5140, -0.0650, -0.0967, 0.1583, 0.1520, -0.0118, 0.0050, 0.1185, -0.0814, -0.3000, -0.3038, 0.0004, -0.0050, 0.0050, -0.0220, 0.0753, 0.0, 0.0050, 0.0000, -0.0200, -0.9050, -0.0800, -0.2133, 0.0352, -0.4700, 0.0086, 0.0700, -0.0280, -0.1375, 0.0100, -0.2120, 0.0030, -0.0060, 0.0100, 0.0, 0.0, 0.0, 0.0360, -0.0180, 0.2800, -0.0150, 0.0133, 0.0650, 0.0, 0.0]}, {"key" : "pacw", "val" : [-0.0100, -0.2900, -0.0214, -0.0067, -0.0200, -0.0108, -0.0200, -0.0358, 0.0000, 0.0600, 0.0233, -0.0183, 0.0000, -0.1820, 0.0164, 0.0, -0.0033, -0.2100, 0.0300, -0.0040, 0.0020, -0.0420, 0.0000, 0.0, -0.1660, -0.2533, 0.0000, -0.0038, -0.0022, 0.0518, 0.1600, -0.8067, -0.0158, 0.1157, -0.1025, -0.6933, 0.0150, -0.1450, 0.1375, -0.0214, 0.0586, -0.0200, -0.0145, 2.3000, -0.0983, -0.0100, 1.5600, -0.1300, -0.2260, -1.6000, -0.0188, 0.0, 0.0, 0.1000, -0.0425, 0.0800, -0.2400, -0.3040, 0.0450, 0.0, 0.0, 0.1641, -0.0500, -0.0060, 0.5000, 0.0014, -0.0320, -0.2480, -0.1667, 0.0]}, {"key" : "rivn", "val" : [0.0105, 0.0050, -0.0070, -0.0136, -0.0155, -0.0065, -0.0241, -0.0236, -0.0867, 0.2777, 0.3440, 0.3618, 0.2373, 0.1556, 0.1280, 0.0600, 0.0, -0.5000, -0.4100, 0.0078, -0.7786, -0.0327, 0.0500, 0.0, -0.0550, -2.6933, -1.4900, -0.0144, 0.1340, 0.0, 0.0392, 0.2230, -0.0329, 0.1279, 0.2682, -1.7800, 0.1208, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]}, {"key" : "t", "val" : [-0.0155, -0.1600, -0.0103, -0.0096, -0.0139, -0.0057, -0.0174, -0.0167, 0.0178, -0.0050, 0.0733, 0.0300, -0.0400, -0.0138, -0.3625, -0.0220, 0.0342, 0.0350, -0.3400, 0.0000, 0.0227, -0.0186, 0.0, 0.8800, 0.0180, -0.0145, 0.0227, -0.0114, -0.2233, -0.0040, -0.5200, -0.0108, 0.0217, -0.0080, -0.6067, -0.0033, -0.0127, 0.0200, -0.2000, 0.0300, 0.0340, 0.0, 0.0280, -0.1400, -0.2580, 0.0827, 0.3000, -0.4400, 0.0900, 0.0700, -0.0550, -0.1267, 0.2080, 0.0429, -0.0033, 0.0, -0.1633, -0.1000, -0.0887, 0.0036, -0.0600, 0.0555, -0.0620, -0.0025, -0.0200, 0.0117, -0.3550, 0.0083, 0.0900, 0.0300]}, {"key" : "tsla", "val" : [0.0295, -0.0733, -0.0345, -0.0994, -0.0168, 0.0746, -0.0429, 0.1759, 0.4217, -0.0909, 0.7257, 0.5605, 0.1517, 0.2135, 1.1150, 0.5218, 0.0, -36.8100, -0.6758, 0.3050, 0.5300, 0.4392, 42.5700, 0.0, -0.0820, 0.3977, 0.0800, 0.5473, -0.1233, -4.8800, 0.6380, -1.2037, 0.4044, -0.2179, 0.6480, 0.0, 0.9333, -0.0264, 0.0938, 0.0989, -0.8228, 0.0, -0.0511, 0.0, -5.1743, 3.2000, 0.0700, -12.7400, -0.0604, -26.5500, 0.0, 0.0, 0.0, 0.0, 1.8229, 2.4900, -1.8033, 0.8751, 0.0, 0.0, 0.1635, 1.1853, 0.1947, -2.1267, 1.1000, -0.0416, 0.5980, -2.0569, -2.6856, -2.7100]}, {"key" : "uber", "val" : [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.3375, -0.4500, 0.2609, 0.0527, 0.0400, 0.0467, 1.7600, 0.0, -0.2629, 0.0693, 0.0378, -3.6100, 0.2364, 0.6833, -0.7425, 0.0070, 0.1110, -0.0671, -0.5275, 0.1317, 0.0019, -0.5167, 0.1525, 0.0633, 0.3560, -0.2100, 0.0760, 0.0100, -0.0310, -0.1325, 0.0600, 0.1250, 0.6313, -18.4000, -0.0250, 0.3978, 0.8613, 0.0781, 7.9883, 0.1757, 0.1371, 0.3000, 0.2794, 0.2950, -0.1511, 0.4220, -0.0929, 0.0475, -5.2100, -0.0650, 0.1488, 0.1058, -0.0513, 0.0786]
    //             }
    //         ]
    //     }
    // "#).unwrap()

}

/// Ask the database for data for the chart
/// TODO: add an enum for the kind of chart to fetch
async fn request_chart(chart_type: ChartType, tx_db:Sender<Msg>) -> Result<serde_json::Value, KitchenSinkError> {
    let (sender, rx) = oneshot::channel::<serde_json::Value>();
    match tx_db.send(Msg::RequestChart {chart_type: chart_type, sender: sender}) {
        Ok(_)=> {
            match rx.await {
                Ok(chart) => Ok(chart),
                Err(_) => Err(KitchenSinkError::ChannelError),
            }
        },
        Err(_)=>Err(KitchenSinkError::ChannelError),
    }
}

// /// Ask the database for data for the chart
// /// TODO: add an enum for the kind of chart to fetch
// async fn request_chart_test(chart_type: ChartType, tx_db:Sender<Msg>) -> Result<Chart, KitchenSinkError> {
//     let (sender, rx) = oneshot::channel::<Chart>();
//     match tx_db.send(Msg::RequestChartJson {chart_type: chart_type, sender: sender}) {
//         Ok(_)=> {
//             match rx.await {
//                 Ok(chart) => Ok(chart),
//                 Err(_) => Err(KitchenSinkError::ChannelError),
//             }
//         },
//         Err(_)=>Err(KitchenSinkError::ChannelError),
//     }
// }

// /// GET /profit
// /// print a table of stocks P/L
// pub async fn present_chart_test(tx_db: web::Data<Sender<Msg>>, hb: web::Data<Handlebars<'_>>/*, session: Session*/) -> HttpResponse {
//     tracing::debug!("[get_profit]");
//
//     // logged in?
//     // if let Ok(Some(session_username)) = session.get::<String>(SESSION_USERNAME) {
//
//         // tracing::debug!("[get_analysis] logged in with session id: {}", &session_username);
//
//         let tx_db = tx_db.into_inner().as_ref().clone();
//         // let tx_db_1 = tx_db.clone();
//         // let tx_db_2 = tx_db.clone();
//         // let tx_db_3 = tx_db.clone();
//         // let tx_db_4 = tx_db.clone();
//
//         // get both charts' data from database
//         let chart_0_result = request_chart_test(ChartType::Test, tx_db).await;
//         // let chart_1_result = analysis_chart_avg_profit(tx_db_1).await;
//         // let chart_2_result = chart_profit_daily(tx_db_2).await;
//         // let chart_3_result = chart_orders_daily(tx_db_3).await;
//         // let chart_4_result = chart_value_traded(tx_db_4).await;
//
//         if chart_0_result.is_ok() /*&& chart_1_result.is_ok() && chart_2_result.is_ok() && chart_3_result.is_ok() && chart_4_result.is_ok()*/ {
//
//             tracing::debug!("[get_analysis] all chart data ok");
//
//             let chart_0 = chart_0_result;
//             // let chart_0 = chart_0_result.unwrap();
//             // let chart_1 = chart_1_result.unwrap();
//             // let chart_2 = chart_2_result.unwrap();
//             // let chart_3 = chart_3_result.unwrap();
//             // let chart_4 = chart_4_result.unwrap();
//
//             // tracing::debug!("[get_analysis] db result: {:?}", &avg_profit);
//             // TODO: remove unwrap
//             let chart_0 = chart_0.unwrap();
//             let chart_0_columns = serde_json::to_string(&chart_0.columns).unwrap();
//             let chart_0_data = serde_json::to_string(&chart_0.chart_data).unwrap();
//
//             // let chart_1_columns = serde_json::to_string(&chart_1.columns).unwrap();
//             // let chart_1_data = serde_json::to_string(&chart_1.chart_data).unwrap();
//             //
//             // let chart_2_columns = serde_json::to_string(&chart_2.columns).unwrap();
//             // let chart_2_data = serde_json::to_string(&chart_2.chart_data).unwrap();
//             //
//             // let chart_3_columns = serde_json::to_string(&chart_3.columns).unwrap();
//             // let chart_3_data = serde_json::to_string(&chart_3.chart_data).unwrap();
//             //
//             // let chart_4_columns = serde_json::to_string(&chart_4.columns).unwrap();
//             // let chart_4_data = serde_json::to_string(&chart_4.chart_data).unwrap();
//
//             let data = json!({
//                 "title": "Analysis",
//                 "parent": "base0",
//                 "is_logged_in": true,
//                 // "session_username": &session_username,
//                 "chart_0_columns": chart_0_columns,
//                 "chart_0_data": chart_0_data,
//                 // "chart_1_columns": chart_1_columns,
//                 // "chart_1_data": chart_1_data,
//                 // "chart_2_columns": chart_2_columns,
//                 // "chart_2_data": chart_2_data,
//                 // "chart_3_columns": chart_3_columns,
//                 // "chart_3_data": chart_3_data,
//                 // "chart_4_columns": chart_4_columns,
//                 // "chart_4_data": chart_4_data,
//             });
//
//             let body = hb.render("analysis", &data).unwrap();
//             HttpResponse::Ok().append_header(("cache-control", "no-store")).body(body)
//
//         } else {
//             // TODO: figure out how to do a match with two results
//             tracing::error!("[get_analysis] database error getting chart data");
//             redirect_home().await
//         }
//     // } else {
//     //     tracing::info!("[get_analysis] not logged in redirecting home");
//     //     redirect_home().await
//     // }
// }

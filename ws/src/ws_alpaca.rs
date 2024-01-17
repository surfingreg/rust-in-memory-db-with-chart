//! ws_alpaca.rs
//!
//! Alpaca Websocket (for crypto)
//!
//! ref: https://docs.alpaca.markets/docs/real-time-crypto-pricing-data
//!
use std::net::TcpStream;
use chrono::{DateTime, Utc};
use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tungstenite::{Message, WebSocket};
use tungstenite::stream::MaybeTlsStream;
use common_lib::{DbMsg};

// /// fn<'de, D>(D) -> Result<T, D::Error> where D: Deserializer<'de>
// /// https://serde.rs/field-attrs.html
// /// https://stackoverflow.com/questions/46753955/how-to-transform-fields-during-deserialization-using-serde
// fn f64_from_str<'de, D>(deserializer: D) -> Result<f64, D::Error>
//     where D: serde::Deserializer<'de>,
// {
//     let s: &str = serde::Deserialize::deserialize(deserializer)?;
//     f64::from_str(&s).map_err(D::Error::custom)
// }

fn stock_list_to_uppercase(lower_stock: &Vec<String>) -> Vec<String> {
    lower_stock.iter().map(|x| x.to_uppercase()).collect()
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestAuthenticate {
    pub action: RequestAction,
    pub key: String,
    pub secret: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RequestAction {
    Auth,
    Listen,
    TradeUpdates,
    AccountUpdates,
    Subscribe,
    Ping,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "T")]
pub enum DataMessage{
    Success(DataMesgSuccess),
    Subscription(DataMesgSubscriptionListCrypto),
    #[serde(rename = "t")]
    Trade(AlpacaTrade),
    #[serde(rename = "b")]
    Bar(AlpacaBar),
    #[serde(rename = "q")]
    Quote(AlpacaQuote),
    #[serde(rename = "d")]
    DailyBar,
    #[serde(rename = "s")]
    Status,
    Error,

}

/// [{"T":"success","msg":"connected"}]
/// [{"T":"success","msg":"authenticated"}]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag="msg")]
pub enum DataMesgSuccess {
    Connected,
    Authenticated,
}

///
/// [{"T":"subscription","trades":["BTC/USD"],"quotes":[],"orderbooks":[],"bars":[],"updatedBars":[],"dailyBars":[]}]
///
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataMesgSubscriptionListCrypto {
    pub trades:Vec<String>,
    pub quotes:Vec<String>,
    #[serde(rename="bars")]
    pub bars:Vec<String>,
    #[serde(rename="updatedBars")]
    pub updated_bars:Vec<String>,
    #[serde(rename="dailyBars")]
    pub daily_bars:Vec<String>,
}

///
/// [{"T":"t","S":"BTC/USD","p":26711.38,"s":0.01328,"t":"2023-03-17T12:32:12.121Z","i":6403854,"tks":"S"}]
///
/// [{"T":"t","S":"BTC/USD","p":41853,"s":0.01,"t":"2024-01-14T23:36:23.17799008Z","i":8102483281662395354,"tks":"B"}]
///
/// [{"T":"t","S":"BTC/USD","p":41927.9,"s":0.002028277,"t":"2024-01-14T23:46:29.667996827Z","i":4327835828814694871,"tks":"S"}]
///
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlpacaTrade {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub id_trade: u64,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: f64,
    #[serde(rename = "t")]
    pub dtg: DateTime<Utc>,
    pub tks: String,

}

/// [{"T":"q","S":"BTC/USD","bp":42135.56,"bs":0.27779,"ap":42176.435,"as":0.550171,"t":"2024-01-14T23:06:25.205996645Z"}]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlpacaQuote {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "bp")]
    pub quote_bp: f64,
    #[serde(rename = "bs")]
    pub quote_bs: f64,
    #[serde(rename = "ap")]
    pub quote_ap: f64,
    #[serde(rename = "as")]
    pub quote_as: f64,
    #[serde(rename = "t")]
    pub dtg: DateTime<Utc>,
}

/// [{"T":"b","S":"BTC/USD","o":42006.2005,"h":42051.4725,"l":42006.2005,"c":42051.4725,"v":0,"t":"2024-01-14T23:30:00Z","n":0,"vw":0}]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlpacaBar {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "o")]
    pub open: f64,
    #[serde(rename = "h")]
    pub high: f64,
    #[serde(rename = "l")]
    pub low: f64,
    #[serde(rename = "c")]
    pub close: f64,
    #[serde(rename = "v")]
    pub volume: f64,
    #[serde(rename = "vw")]
    pub bar_vw: f64,
    #[serde(rename = "n")]
    pub num_trades: f64,
    #[serde(rename = "t")]
    pub dtg: DateTime<Utc>,
}

pub fn parse(mut ws: WebSocket<MaybeTlsStream<TcpStream>>, _tx_db: Sender<DbMsg>) {

    let _ = ws.send(Message::Text(authenticate().to_string()));

    loop {

        let msg_result = ws.read();

        match msg_result {
            Ok(Message::Ping(p)) => tracing::debug!("[parse][ping] {:?}", &p),
            Ok(Message::Text(t_msg)) => {
                // tracing::debug!("[parse] txt: {}", &t_msg);

                /*

                2024-01-14T21:56:35.710485Z DEBUG ws::ws_alpaca: [parse] txt: [{"T":"success","msg":"connected"}]
                2024-01-14T21:56:35.815223Z DEBUG ws::ws_alpaca: [parse] txt: [{"T":"error","code":402,"msg":"auth failed"}]
                or
                2024-01-14T22:02:24.336270Z DEBUG ws::ws_alpaca: [parse] txt: [{"T":"success","msg":"authenticated"}]

                 */

                let json_vec = serde_json::from_str::<Vec<DataMessage>>(&t_msg);

                match json_vec {
                    Ok(data_vec)=>{
                        for data in data_vec{
                            match data{

                                // [{"T":"success","msg":"connected"}]
                                // [{"T":"success","msg":"authenticated"}]
                                DataMessage::Success(success_data)=>{
                                    match success_data{
                                        DataMesgSuccess::Connected=> tracing::debug!("[parse] connected"),
                                        DataMesgSuccess::Authenticated=>{
                                            tracing::debug!("[parse] authenticated");
                                            subscribe(&mut ws);
                                        },
                                    }
                                },

                                DataMessage::Trade(trade)=>{
                                    tracing::error!("[parse][trade] {:?}", &trade);
                                    // let _ = tx_db.send(DbMsg::TradeAlpaca(trade.to_owned()));
                                },
                                DataMessage::Bar(b)=>{
                                    tracing::error!("[parse][bar] {:?}", &b);
                                },
                                DataMessage::Quote(q)=>{
                                    // [{"T":"q","S":"BTC/USD","bp":42226.056,"bs":0.27826,"ap":42256.5,"as":0.2754,"t":"2024-01-14T22:42:13.326734394Z"}
                                    tracing::debug!("[parse][quote] {:?}", &q);

                                },
                                // DataMessage::DailyBar=>{},
                                // DataMessage::Status=>{},
                                DataMessage::Subscription(list) => tracing::info!("[parse][text][subscription] {:?}", &list),
                                DataMessage::Error => tracing::error!("[parse][text][error] error: {:?}", &data),
                                _ => {
                                    tracing::debug!("[parse] txt: {}", &t_msg);
                                }
                            }
                        }
                    },
                    Err(e)=>{
                        tracing::debug!("[parse] txt: {}", &t_msg);
                        tracing::error!("[parse][text][error] data message parse error: {:?}", &e);
                    }
                }
            },
            _ => {}
        }
    }
}



/// Generate the websocket message needed to authenticate/authorize.
///
/// https://alpaca.markets/docs/api-references/trading-api/streaming/
/// https://alpaca.markets/docs/api-references/market-data-api/stock-pricing-data/realtime/
///
/// Authenticate using:
/// {"action": "auth", "key": "{KEY_ID}", "secret": "{SECRET}"}
///
/// Response:
/// [{"T":"success","msg":"authenticated"}]
///
///                 // authenticate example (old credentials)
///
///   $ wscat -c wss://stream.data.alpaca.markets/v2/iex
///     connected (press CTRL+C to quit)
/// {"action": "auth","key":"","secret":""}
///                    < {"stream":"authorization","data":{"action":"authenticate","status":"authorized"}}
///                    >  {"action": "listen", "data": {"streams": ["T.SPY"]}}
///                    < {"stream":"listening","data":{"streams":["T.SPY"]}}
///
fn authenticate() -> serde_json::Value {
    // {"action": "authenticate","data": {"key_id": "???", "secret_key": "???"}}

    // TODO: add database setting "use_paper_or_live_key"
    let api_key = std::env::var("ALPACA_API_ID").expect("ALPACA_API_ID not found");
    let api_secret = std::env::var("ALPACA_API_SECRET").expect("ALPACA_API_SECRET not found");

    let json_obj = RequestAuthenticate {
        action: RequestAction::Auth, // "auth".to_owned()
        key: api_key,
        secret: api_secret,
    };

    let j: serde_json::Value = serde_json::to_value(&json_obj).expect("[gen_subscribe_json] json serialize failed");
    j
}

/// subscribe to stock feeds
/// https://alpaca.markets/docs/api-references/market-data-api/stock-pricing-data/realtime/#subscribe
fn subscribe(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) {

    let symbols = vec!("BTC/USD".to_string());

    let json = json!({
        "action": RequestAction::Subscribe,
        "trades": stock_list_to_uppercase(&symbols),
        "quotes": stock_list_to_uppercase(&symbols),
        "bars": stock_list_to_uppercase(&symbols),
    });
    tracing::debug!("[subscribe] sending subscription request...\n{}", &json);
    let result = ws.send(Message::Text(json.to_string()));
    tracing::info!("[subscribe] subscription request sent: {:?}", &result);
}



use crate::market_structs::{Stat, Trade};
use std::thread::JoinHandle;
use chrono::{DateTime, Utc};



#[derive(Debug)]
pub enum Msg{
	// Statistic(Stat),
	StatVector(Vec<Stat>),
	Trade(Trade),

}

/// Listen on a different thread for messages to persist to postgresql
/// Each db network call takes 150-300ms on LAN/wifi
pub fn db_thread(client: &mut postgres::Client,rcv:crossbeam::channel::Receiver<Msg>)-> JoinHandle<()>{
	loop {
		// this has the capacity to receive different messages from different transmitters, overkill right now
		crossbeam::channel::select!{
			recv(rcv) -> result => {
				if let Ok(msg) = &result {
					match msg {
						Msg::StatVector(v) => {
							// db_insert_stats(client, &v);
						},
						Msg::Trade(trade) => {

							db_log_insert_trade(client, &trade)
						}
					}
				}
			}
		}
	}
}

/// Insert up to 1000 Stats in one network/database call
fn db_insert_stats(client:&mut postgres::Client, vec_stat:&Vec<Stat>){

	let start = std::time::Instant::now();

	// let mut sql = r#"insert
	// 	into t_stat (dtg, dtg_last_tick, price, diff_ema, diff_ema_roc, spread, ema1, ema2, min_sell, max_buy)
	// 	values ('2020-10-27 21:37:20.488852 UTC', '2020-10-27 21:37:20.488852 UTC', 0.0, 0.0,0.0,0.0,0.0,0.0,0.0,0.0);"#;

	let insert = r#"insert into t_stat (dtg, dtg_last_tick, seq_last_tick, price, diff_ema, diff_ema_roc, spread, ema1, ema2, min_sell, max_buy) values"#;
	let mut sql = format!("{}", insert);

	for st in vec_stat {

		sql = format!("{} ('{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {} ), ", sql,
					  st.dtg,
					  st.dtg_last_tick.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.seq_last_tick.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.price.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.diff_ema.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.diff_ema_roc.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.spread.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.ema1.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.ema2.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.min_sell.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str(),
					  st.max_buy.map(|o| { format!("{}", o)}).unwrap_or("null".to_owned()).as_str()
		);
	}
	sql.pop();
	sql.pop();
	sql = format!("{};", &sql);

	println!("[stats_to_db] insert: {}, elapsed: {:?}", &vec_stat.len(), start.elapsed());

	let _ = client.simple_query(&sql);

}

pub fn db_connect(db_url:&str) -> Option<postgres::Client> {
	println!("[db_connect]");
	let client_result = postgres::Client::connect(&db_url, postgres::NoTls);
	match client_result {
		Err(e) 	=> {
			println!("[db_connect] postgresql connection failed: {}", &e);
			None
		},
		Ok(client) 	=> {
			println!("[db_connect] Connected to postgresql");
			Some(client)
		}
	}
}



/// TODO: excessive unwraps
fn db_log_insert_trade(client:&mut postgres::Client, t: &Trade) {

	let dtg_now: String = Utc::now().to_rfc3339();
	let b = &(t.ticker_buy);
	let s = t.ticker_sell.as_ref().unwrap();

	let sql = format!(r#"
		insert into t_trade
		(
			dtg,
			seq_buy,
			seq_sell,
			net_ticker,
			price_buy,
			price_sell,
			ema1_buy,
			ema2_buy,
			diff_buy,
			diff_roc_buy,
			ema1_sell,
			ema2_sell,
			diff_sell,
			diff_roc_sell,
			
			price_buy_market,
			price_sell_market,
			size_mkt_available_to_buy,
			size_mkt_available_to_sell
			
			
		) values (
			'{}',{},
			{},{},
			{},{},{},
			{},{},{},{},
			{},{},{},
			{},{},{},{}
		);
	"#,
					  dtg_now,
					  // &t.id_buy,
		b.sequence,
					  s.sequence,
					  &t.net_ticker,
					  //TODO: need net_ideal vs net_actual, price_buy_actual, price_sell_actual
		b.price,
					  s.price,
					  b.ema1.unwrap(),
					  b.ema2.unwrap(),
					  b.diff_ema.unwrap(),
					  b.diff_ema_roc.map( | roc_opt |{ format!("{}",roc_opt) }).unwrap_or("null".to_owned()),
					  s.ema1.unwrap(),
					  s.ema2.unwrap(),
					  s.diff_ema.unwrap(),
					  s.diff_ema_roc.map( | roc_opt |{ format!("{}",roc_opt) }).unwrap_or("null".to_owned()),

					  t.price_mkt_for_buy.map( |roc_opt |{ format!("{}", roc_opt) }).unwrap_or("null".to_owned()),
					  t.price_mkt_for_sell.map( |roc_opt |{ format!("{}", roc_opt) }).unwrap_or("null".to_owned()),
					  t.size_mkt_for_buy.map( |roc_opt |{ format!("{}", roc_opt) }).unwrap_or("null".to_owned()),
					  t.size_mkt_for_sell.map( |roc_opt |{ format!("{}", roc_opt) }).unwrap_or("null".to_owned())


	);

	// println!("[db_log_insert_trade] sql: {}", &sql);

	if let Ok(result_vec) = client.simple_query(&sql) {
		for i in result_vec {
			match i {
				postgres::SimpleQueryMessage::CommandComplete(x) =>
					log::debug!("[db_log_insert_trade] {} row inserted:\n{:?}\nmkt profit: {}", x, &t, &t.price_mkt_for_sell.unwrap()-&t.price_mkt_for_buy.unwrap()),
				postgres::SimpleQueryMessage::Row(_row) => {}
				_ => log::debug!("[db_log_insert_trade] Something weird happened on log query."),
			}
		}
	} else {
		println!("[db_log_insert_trade] log insert failed");
	}
}

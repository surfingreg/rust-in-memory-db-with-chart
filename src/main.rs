/*

docker run -d \
--rm \
--name market_watcher \
-e RUST_LOG="info" \
-e COINBASE_URL=wss://ws-feed.pro.coinbase.rs.com \
-e COIN_TRADE_LOG_DB_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin_test \
-e COINBASE_URL=wss://ws-feed-public.sandbox.pro.coinbase.rs.com \
-e COIN_TRADE_LOG_DB_URL=postgres://postgres:PASSWORD@10.1.1.205:54320/coin_test \
-e TRADE_SIZE_TARGET=0.01 \
market_watcher:latest

*/

// https://github.com/rust-lang/rust/issues/62924
// cargo +nightly run
// #![feature(map_first_last)]

mod market;
mod market_structs;
mod ticker;
mod algorithm;
mod db;


use std::str::FromStr;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};
use strum::{Display, EnumString};

#[derive(Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ConfigLocation{
	Docker,
	NotDocker,
}

pub fn init(package_name: &str) {
	// load .env
	// if docker, load .env from the root directory, otherwise use the cargo workspace directory
	let config_location:ConfigLocation = ConfigLocation::from_str(&std::env::var("CONFIG_LOCATION").unwrap_or_else(|_| "not_docker".to_owned())).expect("CONFIG_LOCATION");
	println!("[init] config_location: {}", &config_location);

	let dot_env_path = match config_location {
		ConfigLocation::Docker => ".env".to_string(),
		ConfigLocation::NotDocker => {
			// backend/.env
			// frontend/.env
			// etc
			format!("{}/.env", &package_name)
		}
	};

	match dotenvy::from_filename(&dot_env_path) {
		Ok(_) => println!("[init] .env found"),
		_ => println!("[init] .env not found"),
	}
	let env_file_version = std::env::var("ENV_FILE_VERSION").unwrap_or_else(|_| ".env not loaded".to_string());
	println!("[init] dot_env_path: {}; env_file_version: {}", &dot_env_path, &env_file_version);

	// start tracing
	tracing_subscriber::registry().with(fmt::layer()).with(EnvFilter::from_default_env()).init();
	tracing::info!("[init] .env file: {}", &dot_env_path);

	println!("[init] done");

}


fn main() {

	init(env!("CARGO_MANIFEST_DIR"));

	market::Market::start();
}

// https://github.com/rust-lang/rust/issues/62924
// cargo +nightly run
#![feature(map_first_last)]

mod market;
mod market_structs;
mod ticker;

fn main() {

	dotenv::dotenv().ok();
	env_logger::init();

	market::Market::start();
}

/**

	Algorithms


*/

use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::ticker::Ticker;

#[derive(Debug)]
pub enum TradeRec {
	Buy,
	Sell,
	Hold,
}

/// Compare current and previous tickers for EMA cross
/// (Did the sign change on the diff_ema since the previous ticker)?
/// Algo: recommend buy if diff_ema changed from negative to positive (EMAs crossed going up)
/// Algo: recommend sell if diff_ema changed from positive to negative (EMAs crossed going down)
pub fn recommend_zero_diff_ema_trade(t0: &Ticker, t1: &Ticker) -> TradeRec {

	let zero = Decimal::from_u8(0).unwrap();

	let result = if let (Some(diff0), Some(diff1)) = (t0.diff_ema, t1.diff_ema) {
		if diff0 > zero && diff1 <= zero {
			// current ema diff is positive and was previously negative, so they crossed
			// println!("[get_algo01_recommendation] BUY");
			// Some("buy".to_owned())
			TradeRec::Buy
		} else if diff0 <= zero && diff1 > zero {
			// current ema diff is negative and was previously positive, so they crossed
			// println!("[get_algo01_recommendation] SELL");
			// Some("sell".to_owned())
			TradeRec::Sell
		} else {
			// None
			TradeRec::Hold
		}
	} else {
		// None
		TradeRec::Hold
	};

	println!("[get_algo01_recommendation] {:?}", &result);
	result
}


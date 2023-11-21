// use bigdecimal::BigDecimal;
// use chrono::{DateTime, Utc};
// use serde::{Deserialize, Serialize};
//
// // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/sorting.html
// #[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
// pub struct Ticker{
// 	#[serde(rename = "time")]
// 	pub dtg:DateTime::<Utc>,
// 	pub sequence:u64,
// 	pub price:BigDecimal,
// 	// 05 day (tighter, more accurate)
// 	pub ema1:Option<BigDecimal>,
// 	// 20 day (smoother, less accurate)
// 	pub ema2:Option<BigDecimal>,
// 	pub diff_ema:Option<BigDecimal>,
// 	// rate of change of the moving average diff (accelerating/decelerating)
// 	pub diff_ema_roc:Option<BigDecimal>,
// 	// how far does price lead?
// 	pub diff_price_ema1:Option<BigDecimal>,
// 	pub diff_price_ema2:Option<BigDecimal>,
//
// }
//
// impl Ticker{
// 	pub fn new(
// 		dtg:DateTime::<Utc>,
// 		sequence:u64,
// 		price:BigDecimal,
// 		ema1:Option<BigDecimal>,
// 		ema2:Option<BigDecimal>,
// 		diff_ema:Option<BigDecimal>,
// 		diff_ema_roc:Option<BigDecimal>,
// 		diff_price_ema1:Option<BigDecimal>,
// 		diff_price_ema2:Option<BigDecimal>,
// 	)->Ticker{
// 		Ticker{
// 			dtg,
// 			sequence,
// 			price,
// 			ema1,
// 			ema2,
// 			diff_ema,
// 			diff_ema_roc,
// 			diff_price_ema1,
// 			diff_price_ema2,
// 		}
// 	}
//
// 	pub fn clone(&self) -> Ticker{
// 		Ticker::new(
// 			self.dtg,
// 			self.sequence,
// 			self.price.clone(),
// 			self.ema1.clone(),
// 			self.ema2.clone(),
// 			self.diff_ema.clone(),
// 			self.diff_ema_roc.clone(),
// 			self.diff_price_ema1.clone(),
// 			self.diff_price_ema2.clone(),
// 		)
// 	}
// }
//

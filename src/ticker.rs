
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// https://rust-lang-nursery.github.io/rust-cookbook/algorithms/sorting.html
#[derive(Deserialize, Serialize, Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct Ticker{
	#[serde(rename = "time")]
	pub dtg:DateTime::<Utc>,
	pub sequence:u64,
	pub price:rust_decimal::Decimal,
	// 05 day (tighter, more accurate)
	pub ema1:Option<rust_decimal::Decimal>,
	// 20 day (smoother, less accurate)
	pub ema2:Option<rust_decimal::Decimal>,
	pub diff_ema:Option<rust_decimal::Decimal>,
	// rate of change of the moving average diff (accelerating/decelerating)
	pub diff_ema_roc:Option<rust_decimal::Decimal>,
	// how far does price lead?
	pub diff_price_ema1:Option<rust_decimal::Decimal>,
	pub diff_price_ema2:Option<rust_decimal::Decimal>,

}

impl Ticker{
	pub fn new(
		dtg:DateTime::<Utc>,
		sequence:u64,
		price:rust_decimal::Decimal,
		ema1:Option<rust_decimal::Decimal>,
		ema2:Option<rust_decimal::Decimal>,
		diff_ema:Option<rust_decimal::Decimal>,
		diff_ema_roc:Option<rust_decimal::Decimal>,
		diff_price_ema1:Option<rust_decimal::Decimal>,
		diff_price_ema2:Option<rust_decimal::Decimal>,
	)->Ticker{
		Ticker{
			dtg,
			sequence,
			price,
			ema1,
			ema2,
			diff_ema,
			diff_ema_roc,
			diff_price_ema1,
			diff_price_ema2,
		}
	}

	pub fn clone(&self) -> Ticker{
		Ticker::new(
			self.dtg,
			self.sequence,
			self.price,
			self.ema1,
			self.ema2,
			self.diff_ema,
			self.diff_ema_roc,
			self.diff_price_ema1,
			self.diff_price_ema2,
		)
	}
}


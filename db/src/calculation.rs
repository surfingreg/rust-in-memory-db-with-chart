//! calculation.rs
//!


use std::time::Instant;
use common_lib::{CalculationId, ProductId};
use common_lib::cb_ticker::{TickerCalc, TickerSource};
use crate::event_book::EventBook;
use crate::event_log::{EventLog, EventLogError};

/// read lock
pub fn refresh_calculations(ticker_src: TickerSource, evt_book: &EventBook, prod_id: ProductId) ->Result<(), EventLogError> {

    tracing::debug!("[refresh_calculations]");
    let start = Instant::now();
    let mut temp = vec![];

    {
        let evt_book_read_lock = evt_book.book.read().unwrap();
        let evt_log: &EventLog = evt_book_read_lock.get(&ticker_src).unwrap();

        // moving averages
        let ma_0010 = evt_log.calculate_moving_avg_n(&CalculationId::MovingAvg0010, &prod_id)?;
        let ma_0100 = evt_log.calculate_moving_avg_n(&CalculationId::MovingAvg0100, &prod_id)?;
        let ma_1000 = evt_log.calculate_moving_avg_n(&CalculationId::MovingAvg1000, &prod_id)?;

        // calculate the moving average diff (ie in an EMA diff algorithm, positive means trending upward, negative means turning down)
        // let ma_diff_0010_1000 = TickerCalc {
        //     dtg: (&ma_0100).dtg.clone(),
        //     prod_id: (&ma_0100).prod_id.clone(),
        //     calc_id: CalculationId::MovAvgDiff0010_1000,
        //     val: (&ma_0010).val.clone() - (&ma_1000).val.clone(),
        // };

        let ma_diff_0100_1000 = TickerCalc {
            dtg: (&ma_0100).dtg.clone(),
            prod_id: (&ma_0100).prod_id.clone(),
            calc_id: CalculationId::MovAvgDiff0100_1000,
            val: (&ma_0100).val.clone() - (&ma_1000).val.clone(),
        };

        temp.push(ma_0010);
        temp.push(ma_0100);
        temp.push(ma_1000);
        // temp.push(ma_diff_0010_1000);
        temp.push(ma_diff_0100_1000);

        if let Ok(slope_100) = evt_log.calculate_diff_slope(&CalculationId::MovAvgDiff0100_1000, &CalculationId::MovAvgDiffSlope0100_1000, &ProductId::BtcUsd) {
            temp.push(slope_100);
        }

        // ...release read lock (holding read blocks write lock)
    };

    // write lock...
    for c in temp.iter() {
        let _ = evt_book.push_calc(&ticker_src, &c);
    }

    tracing::debug!("[update_moving_averages] {:?}ms", start.elapsed().as_micros() as f64 / 1000.0);

    Ok(())
}



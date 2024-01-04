//! chat_main
//!
//! Goal: use SQL to query an in-memory dataset (via Apache DataFusion and Apache Arrow)
//!
//! https://docs.rs/datafusion/latest/datafusion/datasource/memory/struct.MemTable.html
//!

use common_lib::cb_ticker::{Ticker, TickerCalc};
use datafusion::arrow::array::{Date64Array, Float64Array, PrimitiveArray, StringArray};
use datafusion::arrow::datatypes::{DataType, Date64Type, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::dataframe::DataFrameWriteOptions;
use datafusion::prelude::*;
use slice_ring_buffer::SliceRingBuffer;
use std::sync::{Arc};
use std::time::{Instant};
use chrono::{DateTime, Utc};
use strum::IntoEnumIterator;
use common_lib::{CalculationId, ChartDataset, ChartTimeSeries, UniversalError, ProductId};

const RING_BUF_SIZE: usize = 100;
const NUM_CALCS: usize = 4;

/// Ring buffer with ability to extract the entire buffer as a slice
/// https://docs.rs/slice-ring-buffer/0.3.3/slice_ring_buffer/
pub struct EventLog {
    log: SliceRingBuffer<Ticker>,
    calc_log: SliceRingBuffer<TickerCalc>,
}

#[allow(dead_code)]
impl EventLog {
    pub fn new() -> EventLog {
        EventLog {
            log: SliceRingBuffer::<Ticker>::with_capacity(RING_BUF_SIZE),
            calc_log: SliceRingBuffer::<TickerCalc>::with_capacity(NUM_CALCS * RING_BUF_SIZE),
        }
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    pub fn push_log(&mut self, ticker: &Ticker) -> Result<(), EventLogError> {
        self.log.push_front((*ticker).clone());
        Ok(())
    }

    /// push into this custom event log; thread safe
    pub fn push_calc(&mut self, ticker: &TickerCalc) -> Result<(), EventLogError> {
        self.calc_log.push_front((*ticker).clone());
        Ok(())
    }

    /// prep for chartjs
    /// limit: limit the number of values returned
    ///
    /// todo: filter by product ID
    ///
    pub async fn get_data_for_multi_line_chart(&self, filter_prod_id: Vec<ProductId>, since: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<ChartDataset>, UniversalError>  {
        let mut data: Vec<ChartDataset> = vec!();

        // "select...group by product_id..."
        for product_id in filter_prod_id {

            // 'group by product_id', limit query target to 1000 (or fewer) after filtering(?)
            let time_series_data: Vec<ChartTimeSeries> = self.log.iter()
                // .rev()
                .filter(|f| {
                    f.product_id == product_id
                        &&
                    if since.is_none() {
                        // no since specified
                        true
                    } else {
                        f.dtg > since.unwrap()
                    }
                })
                .take(limit)
                .map(|x| { ChartTimeSeries { x: x.dtg, y: x.price } })
                .collect();

            let chart = ChartDataset {
                label: product_id.to_string(),
                data: time_series_data,
            };
            data.push(chart);

            // "...group by product_id, calculation_id..."
            for calc_id in CalculationId::iter(){
                // grouped by product ID
                let time_series_f64: Vec<ChartTimeSeries> = self.calc_log
                    .iter()
                    // .rev()
                    .filter(|f|

                        f.prod_id == product_id && f.calc_id == calc_id
                        &&
                        if since.is_none() {
                            // no since specified
                            true
                        } else {
                            f.dtg > since.unwrap()
                        }
                    )
                    .take(
                        limit
                        // todo: since compare like above
                    )
                    .map(|x|{
                        ChartTimeSeries { x: x.dtg, y: x.val }
                    })
                    .collect();

                let chart = ChartDataset {
                    label: format!("{}_{}", product_id.to_string(), calc_id.to_string()),
                    data: time_series_f64,
                };
                data.push(chart);
            }
        }
        Ok(data)
    }

    /// Compute the average of the last N prices
    pub fn calculate_moving_avg_n(&self, calc_id: &CalculationId, prod_id: &ProductId) -> Result<TickerCalc, EventLogError> {
        // tracing::debug!("[calculate_moving_avg_n]");
        let slice_max = calc_id.value();

        // use len if len is less than max slice
        let log_count = self.len();
        let slice_max = if log_count < slice_max {
            log_count
        } else {
            slice_max
        };

        // How many copies is this doing?
        let slice_n:&[Ticker] = &self.log.as_slice()[0..slice_max];
        let slice_n:Vec<Ticker> = slice_n.iter().filter(|x| x.product_id == *prod_id).map(|x| x.clone()).collect();

        let avg_n: f64 = slice_n.iter().map(|x| x.price).sum::<f64>() / slice_n.len() as f64;

        let dtg_of_this_calc:DateTime<Utc> = if slice_n.len() > 0{
            slice_n[0].dtg
        } else {
            Utc::now()
        };

        Ok(TickerCalc{
            dtg: dtg_of_this_calc,
            prod_id: prod_id.clone(),
            calc_id: calc_id.clone(),
            val: avg_n,
        })

    }

    pub fn schema() -> Schema {
        Schema::new(vec![
            Field::new("dtg", DataType::Date64, false),
            Field::new("product_id", DataType::Utf8, false),
            Field::new("price", DataType::Float64, false),
        ])
    }

    /// hacked over from a coinbase websocket stream, hence the product id and price fields
    pub fn record_batch(&self) -> Result<RecordBatch, EventLogError> {
        let dates: Vec<i64>;
        let product_ids: Vec<String>;
        let prices: Vec<f64>;
        {
            // lock inside closure to end lock asap
            // TODO: unwrap
            // match self.read(){
            //     Ok(log_readable)=>{
            dates = self.log.iter().map(|x| x.dtg.timestamp_millis()).collect();
            product_ids = self
                .log
                .iter()
                .map(|x| (x.product_id.to_string()).clone())
                .collect();
            prices = self.log.iter().map(|x| x.price).collect();

        }

        let dates: PrimitiveArray<Date64Type> = Date64Array::from(dates);
        let product_ids: StringArray = StringArray::from(product_ids);
        let prices: Float64Array = Float64Array::from(prices);

        match RecordBatch::try_new(Arc::new(EventLog::schema()), vec![Arc::new(dates), Arc::new(product_ids), Arc::new(prices)]) {
            Ok(x) => {
                tracing::info!("[record_batch] {:?}", &x);
                Ok(x)
            },
            Err(_e) => Err(EventLogError::ArrowError),
        }
    }

    /// select * from table
    pub async fn query_sql_for_chart(&self) -> datafusion::error::Result<DataFrame> {

        tracing::debug!("[query_sql_for_chart]");

        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();

        // select columns as "columns!", chart_data as "chart_data!" from v_analysis_net_profit_chart
        let df = ctx.sql(
                r#"
                    select dtg, product_id, price from t_one order by dtg desc
                "#,
            ).await?;

        Ok(df.clone())
    }

    /// select * from table
    pub async fn query_sql_all(&self) -> datafusion::error::Result<DataFrame> {
        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();
        let df = ctx
            .sql(
                r#"
                    select * from t_one order by dtg desc
                "#,
            )
            .await?;

        Ok(df.clone())
    }

    /// Perform calculations on the in-memory data using DataFusion's SQL
    /// select * from table
    pub async fn calc_with_sql(&self) -> datafusion::error::Result<DataFrame> {
        let start = Instant::now();
        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();

        let df = ctx
            .sql(
                r#"
                    select price_no_order, price_ordered, p4, p10, p4-p10 as diff, count from(
                        select
                            (select price from t_one limit 1) as price_no_order
                            ,(select price from t_one order by dtg desc limit 1) as price_ordered
                            ,(select avg(price) from (select * from t_one order by dtg desc limit 4)) as p4
                            ,(select avg(price) from (select * from t_one order by dtg desc limit 10)) as p10
                            ,(select count(*) from t_one) as count
                    )
                "#,
            )
            .await?;

        // milliseconds elapsed
        tracing::debug!("[sql] elapsed: {} ms", start.elapsed().as_micros() as f64 / 1000.0);
        Ok(df.clone())
    }

    /// FYI: DataFusion doesn't by default print chrono DateTimes with the time
    pub fn _print_record_batch(&self) {
        // https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatch.html
        match self.record_batch() {
            Ok(batch) => {
                println!("[print_record_batch] rows: {}", batch.num_rows());
                println!("{}", pretty_format_batches(&[batch]).unwrap());
            }
            Err(e) => println!("[print_record_batch] error: {:?}", &e),
        }
    }

    /// demonstrate writing an Arrow Batch to CSV
    /// See the resulting output in tests/data/output/[short_uuid].csv
    pub async fn write_csv(&self) {
        let df = self.query_sql_all().await.unwrap();
        df.write_csv("tests/data/output", DataFrameWriteOptions::new(), None)
            .await
            .unwrap();
    }

    /// Query a CSV file using SQL. Pasted straight out of the Datafusion docs.
    pub async fn query_sql_csv() -> datafusion::error::Result<DataFrame> {
        let ctx = SessionContext::new();
        ctx.register_csv("t_one", "tests/data/test.csv", CsvReadOptions::new())
            .await?;
        let df = ctx
            .sql(
                r#"
                    select dtg, description, member, amount, cat as category
                    from t_one
                    order by dtg desc
                    limit 3
                "#,
            )
            .await?;

        Ok(df.clone())
    }
}

/// Not used
#[allow(dead_code)]
#[derive(Debug)]
pub enum EventLogError {
    PushError,
    ReadLockError,
    WriteLockError,
    OtherError,
    ArrowError,
}

#[cfg(test)]
mod tests {
    use crate::event_log::EventLog;
    use chrono::{DateTime, Utc};
    use common_lib::cb_ticker::{Ticker};
    use datafusion::arrow::util::pretty::pretty_format_batches;
    use common_lib::{CalculationId, ProductId};

    #[test]
    fn test_calculate_moving_avg_n(){
        let d1 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
        let mut e_log = EventLog::new();
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 10.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 10.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 10.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 10.0});
        let ticker_calc = e_log.calculate_moving_avg_n(&CalculationId::MovingAvg0004, &ProductId::BtcUsd).unwrap();
        println!("[test_calculate_moving_avg_n] {:?}", ticker_calc);
        assert_eq!(ticker_calc.val, 10.0);

        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 30.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 30.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 30.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 30.0});

        // last 4 average should be 30; last 10 average should be 20
        let ticker_calc = e_log.calculate_moving_avg_n(&CalculationId::MovingAvg0004, &ProductId::BtcUsd).unwrap();
        println!("[test_calculate_moving_avg_n] {:?}", ticker_calc);
        assert_eq!(ticker_calc.val, 30.0);

        let ticker_calc = e_log.calculate_moving_avg_n(&CalculationId::MovingAvg0010, &ProductId::BtcUsd).unwrap();
        println!("[test_calculate_moving_avg_n] {:?}", ticker_calc);
        assert_eq!(ticker_calc.val, 20.0);

    }

    ///  what if we mix in some entries of a different type? Is it reading only the average of the one we asked for?
    #[test]
    fn test_calculate_avg_2(){
        let d1 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());

        let mut e_log = EventLog::new();
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 10.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::BtcUsd, price: 10.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::EthUsd, price: 500.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::EthUsd, price: 500.0});
        let _ = e_log.push_log(&Ticker { dtg: d1.clone(), product_id: ProductId::EthUsd, price: 500.0});
        let ticker_calc = e_log.calculate_moving_avg_n(&CalculationId::MovingAvg0004, &ProductId::BtcUsd).unwrap();
        println!("[test_calculate_moving_avg_n] test 2 mixed prod_id {:?}", ticker_calc);
        assert_eq!(ticker_calc.val, 10.0);

        let ticker_calc = e_log.calculate_moving_avg_n(&CalculationId::MovingAvg0004, &ProductId::EthUsd).unwrap();
        println!("[test_calculate_moving_avg_n] test 2 mixed prod_id {:?}", ticker_calc);
        assert_eq!(ticker_calc.val, 500.0);

    }

    /// create and print an Arrow record batch
    #[test]
    fn test_struct_array_to_batch() {
        let d1 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
        let mut e_log = EventLog::new();
        let _ = e_log.push_log(&Ticker {
            dtg: d1.clone(),
            product_id: ProductId::BtcUsd,
            price: 88.87,
        });
        // let d2 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1997-12-19T16:39:57-08:00").unwrap());
        // let _ = e_log.push(&Ticker{
        //     dtg: d2,
        //     product_id: ProductId::BtcUsd,
        //     price: 99.99,
        // });
        let batch = e_log.record_batch().unwrap();
        // println!("batch: {:?}", &batch);
        let test_case = pretty_format_batches(&[batch]).unwrap().to_string();
        // println!("{}", &test_case);
        let expected_result = "+---------------------+------------+-------+
| dtg                 | product_id | price |
+---------------------+------------+-------+
| 1996-12-20T00:39:57 | btc_usd    | 88.87 |
+---------------------+------------+-------+";
        assert_eq!(test_case, expected_result);
    }

    /// Load an Arrow batch from memory, then query it using SQL (via DataFusion)
    #[tokio::test]
    async fn test_query_memory() -> datafusion::error::Result<()> {
        let mut e_log = EventLog::new();
        let d1 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
        let _ = e_log.push_log(&Ticker {
            dtg: d1.clone(),
            product_id: ProductId::BtcUsd,
            price: 88.87,
        });
        // let d2 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1997-12-19T16:39:57-08:00").unwrap());
        // let _ = e_log.push(&Ticker {
        //     dtg: d2,
        //     product_id: ProductId::BtcUsd,
        //     price: 99.99,
        // });
        let df = e_log.query_sql_all().await.unwrap();
        let vec_record_batch = df.collect().await.unwrap();
        let test_case = pretty_format_batches(vec_record_batch.as_slice())
            .unwrap()
            .to_string();
        let expected_result = "+---------------------+------------+-------+
| dtg                 | product_id | price |
+---------------------+------------+-------+
| 1996-12-20T00:39:57 | btc_usd    | 88.87 |
+---------------------+------------+-------+";
        assert_eq!(test_case, expected_result);
        e_log.write_csv().await;

        Ok(())
    }

    /// Load a batch from CSV, then query it using SQL (via DataFusion)
    #[tokio::test]
    async fn test_query_csv() {
        let df = EventLog::query_sql_csv().await; // .expect("EventLog::query_csv failed");
        let batch_vec = df.unwrap().collect().await.unwrap();
        let test_case = pretty_format_batches(&batch_vec).unwrap().to_string();
        let expected_result = r"+------------+--------------------------------------------+-----------------------+--------+----------+
| dtg        | description                                | member                | amount | category |
+------------+--------------------------------------------+-----------------------+--------+----------+
| 2023-10-14 | 12105 DONNER PASS RDTRUCKEE             CA | Fenstemeier Fudpucker | 11.16  | car_gas  |
| 2023-10-14 | AplPay 12105 DONNER TRUCKEE             CA | Fenstemeier Fudpucker | 39.2   | car_gas  |
| 2023-10-14 | SP LOOP MOUNT       LONDON              GB | Fenstemeier Fudpucker | 69.0   | car      |
+------------+--------------------------------------------+-----------------------+--------+----------+"
            .to_string();

        assert_eq!(test_case, expected_result);
    }
}

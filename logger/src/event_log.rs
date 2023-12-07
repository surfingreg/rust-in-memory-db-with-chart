//! main.rs
//!
//! Goal: use SQL to query an in-memory dataset (via Apache DataFusion and Apache Arrow)
//!
//! https://docs.rs/datafusion/latest/datafusion/datasource/memory/struct.MemTable.html
//!

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use datafusion::arrow::array::{Date64Array, Float64Array, PrimitiveArray, StringArray};
use datafusion::arrow::datatypes::{DataType, Date64Type, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::dataframe::DataFrameWriteOptions;
use datafusion::prelude::*;
use slice_ring_buffer::SliceRingBuffer;
use common_lib::cb_ticker::{Ticker};

/// Container for multiple event logs keyed by a string
pub struct EventBook {
    pub book:Arc<RwLock<HashMap<String, EventLog>>>
}

impl EventBook {
    pub fn new() -> EventBook {
        EventBook {
            book:Arc::new(RwLock::new(HashMap::<String, EventLog>::new()))
        }
    }

    /// get write lock on the entire book and insert a new record
    pub fn push(&self, key:&str, val:&Ticker)->Result<(), BookError>{

        // write lock
        let mut book_writable = self.book.write().unwrap();

        match book_writable.get_mut(key) {
            Some(event_log) => {

                // an event log exists for this key

                // TODO un-unwrap
                let _ = event_log.push(val).unwrap();
                Ok(())
            },
            None => {

                // an event log does not exist for this key; create it

                // 1. create a new event log since there's none for this key
                let mut new_e_log = EventLog::new();
                // 2. put the ticker in the new event log
                new_e_log.push(val).unwrap();
                match new_e_log.push(val){
                    Ok(_)=>{
                        // 3. put the new event log with new ticker in the hashmap
                        // Option<previous> or none returned
                        book_writable.insert(key.to_string(), new_e_log);
                        Ok(())
                    },
                    Err(e)=>{
                        tracing::error!("[push] event log push error: {:?}", &e);
                        Err(BookError::General)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum BookError{
    General
}








#[allow(dead_code)]
const RING_BUF_SIZE:usize=100;

/// Ring buffer with ability to extract the entire buffer as a slice
/// https://docs.rs/slice-ring-buffer/0.3.3/slice_ring_buffer/
pub struct EventLog{
    // log: Arc<RwLock<SliceRingBuffer<Ticker>>>
    log: SliceRingBuffer<Ticker>
}

#[allow(dead_code)]
impl EventLog{
    pub fn new() -> EventLog{
        EventLog{
            // log: Arc::new(RwLock::new(SliceRingBuffer::<Ticker>::with_capacity(RING_BUF_SIZE))),
            log: SliceRingBuffer::<Ticker>::with_capacity(RING_BUF_SIZE)
        }
    }

    // TODO: unwrap
    pub fn len(&self)->usize{
        // self.read().unwrap().len()
        self.log.len()
    }

    // pub fn read(&self) -> Result<RwLockReadGuard<SliceRingBuffer<Ticker>>, EventLogError> {
    //     match self.log.read(){
    //         Ok(x) => Ok(x),
    //         Err(_e)=> Err(EventLogError::ReadLockError),
    //     }
    // }
    //
    // pub fn write(&self) -> Result<RwLockWriteGuard<SliceRingBuffer<Ticker>>, EventLogError> {
    //     match self.log.write(){
    //         Ok(x) => Ok(x),
    //         Err(_e)=> Err(EventLogError::WriteLockError),
    //     }
    // }

    /// push into this custom event log; thread safe
    pub fn push(&mut self, ticker:&Ticker)->Result<(), EventLogError> {

        self.log.push_front((*ticker).clone());
        Ok(())

        // match self.write() {
        //     Ok(mut log_writable) => {
        //         log_writable.push_front((*ticker).clone());
        //         Ok(())
        //     }
        //     Err(e) => {
        //         tracing::error!("[push] lock error: {:?}", &e);
        //         return Err(EventLogError::PushError);
        //     },
        // }
    }

    pub fn schema() -> Schema {
        let schema = Schema::new(vec![
            Field::new("dtg", DataType::Date64, false),
            Field::new("product_id", DataType::Utf8, false),
            Field::new("price", DataType::Float64, false),
        ]);
        schema
    }

    /// hacked over from a coinbase websocket stream, hence the product id and price fields
    pub fn record_batch(&self) -> Result<RecordBatch, EventLogError> {
        let dates: Vec<i64>;
        let product_ids: Vec<String>;
        let prices:Vec<f64>;
        {
            // lock inside closure to end lock asap
            // TODO: unwrap
            // match self.read(){
            //     Ok(log_readable)=>{
                    dates = self.log.iter().map(|x| x.dtg.timestamp_millis()).collect();
                    product_ids = self.log.iter().map(|x| (x.product_id.to_string()).clone()).collect();
                    prices = self.log.iter().map(|x| x.price).collect();
            //     },
            //     Err(e)=>{
            //         return Err(e);
            //     }
            // };
        }

        let dates:PrimitiveArray<Date64Type> = Date64Array::from(dates);
        let product_ids:StringArray = StringArray::from(product_ids);
        let prices:Float64Array = Float64Array::from(prices);

        match RecordBatch::try_new(
            Arc::new(EventLog::schema()),
            vec![
                Arc::new(dates),
                Arc::new(product_ids),
                Arc::new(prices),
            ]
        ){
            Ok(x)=>Ok(x),
            Err(_e)=>Err(EventLogError::ArrowError)
        }
    }

    /// select * from table
    pub async fn query_sql_all(&self) -> datafusion::error::Result<DataFrame> {
        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();
        let df = ctx.sql(r#"
            select * from t_one order by dtg
        "#
        ).await?;

        Ok(df.clone())

    }

    /// Perform calculations on the in-memory data using DataFusion's SQL
    /// select * from table
    pub async fn calc_with_sql(&self) -> datafusion::error::Result<DataFrame> {

        let start = Instant::now();
        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();

        let df = ctx.sql(r#"
            select price_no_order, price_ordered, p4, p10, p4-p10 as diff, count from(
                select
                    (select price from t_one limit 1) as price_no_order
                    ,(select price from t_one order by dtg desc limit 1) as price_ordered
                    ,(select avg(price) from (select * from t_one order by dtg desc limit 4)) as p4
                    ,(select avg(price) from (select * from t_one order by dtg desc limit 10)) as p10
                    ,(select count(*) from t_one) as count
            )
        "#
        ).await?;

        // milliseconds elapsed
        tracing::debug!("[sql] elapsed: {} ms", start.elapsed().as_micros() as f64/1000.0);
        Ok(df.clone())

    }

    /// No SQL solution; calculate the difference between two moving averages of the previous N price changes.
    pub fn calc_curve_diff(&self, curve_n0:usize, curve_n1:usize) {
        let start = Instant::now();
        let avg_0 = self.avg_recent_n(curve_n0).unwrap();
        let avg_1 = self.avg_recent_n(curve_n1).unwrap();
        let diff = avg_0-avg_1;
        let graphic = match diff {
            d if d >= 0.0 => "+++++",
            d if d < 0.0 => "-----",
            _ => "-----"
        };

        let log_count = self.len();
        tracing::debug!("[calc_curve_diff][{:0>4}:{:0>4}] {graphic} diff: {},\tavg_{:0>4}: {},\tavg_{:0>4}: {}, count: {}, elapsed: {} ms", curve_n0, curve_n1, diff, curve_n0, avg_0, curve_n1, avg_1, log_count, start.elapsed().as_micros() as f64/1000.0);

    }



    /// Compute the average of the last N prices
    /// Uses an RwLock
    fn avg_recent_n(&self, n:usize) -> Result<f64, EventLogError> {
        let slice_max = n;

        // use len if len is less than max slice
        let log_count = self.len();
        let slice_max = if log_count < slice_max {
            log_count
        } else {
            slice_max
        };

        // read lock, may not necessarily be perfectly current
        // match  self.read(){
        //     Ok(log_readable) =>{
                let slice_4:&[Ticker] = &self.log.as_slice()[0..slice_max];
                assert_eq!(slice_max, slice_4.len());
                let avg_4:f64 = slice_4.iter().map(|x| {x.price}).sum::<f64>() / slice_4.len() as f64;
                Ok(avg_4)
        //     },
        //     Err(e)=> Err(e)
        // }
    }

    /// FYI: DataFusion doesn't by default print chrono DateTimes with the time
    pub fn _print_record_batch(&self){
        // https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatch.html
        match self.record_batch(){
            Ok(batch)=> {
                println!("[print_record_batch] rows: {}", batch.num_rows());
                println!("{}", pretty_format_batches(&[batch]).unwrap().to_string());

            },
            Err(e)=>println!("[print_record_batch] error: {:?}", &e),
        }
    }

    /// demonstrate writing an Arrow Batch to CSV
    /// See the resulting output in tests/data/output/[short_uuid].csv
    pub async fn write_csv(&self){
        let df = self.query_sql_all().await.unwrap();
        df.write_csv("tests/data/output", DataFrameWriteOptions::new(), None).await.unwrap();
    }

    /// Query a CSV file using SQL. Pasted straight out of the Datafusion docs.
    pub async fn query_sql_csv() ->datafusion::error::Result<DataFrame>{
        let ctx = SessionContext::new();
        ctx.register_csv("t_one", "tests/data/test.csv", CsvReadOptions::new()).await?;
        let df = ctx.sql(r#"
            select dtg, description, member, amount, cat as category
            from t_one
            order by dtg desc
            limit 3
        "#
        ).await?;

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
mod tests{
    use chrono::{DateTime, Utc};
    use datafusion::arrow::util::pretty::pretty_format_batches;
    use crate::event_log::{EventLog};
    use common_lib::cb_ticker::{Ticker, ProductId};

    /// create and print an Arrow record batch
    #[test]
    fn test_struct_array_to_batch(){
        let d1 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
        let mut e_log = EventLog::new();
        let _ = e_log.push(&Ticker{
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
        let expected_result =
            "+---------------------+------------+-------+
| dtg                 | product_id | price |
+---------------------+------------+-------+
| 1996-12-20T00:39:57 | BtcUsd     | 88.87 |
+---------------------+------------+-------+";
        assert_eq!(test_case, expected_result);

    }

    /// Load an Arrow batch from memory, then query it using SQL (via DataFusion)
    #[tokio::test]
    async fn test_query_memory() -> datafusion::error::Result<()>{
        let mut e_log = EventLog::new();
        let d1 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1996-12-19T16:39:57-08:00").unwrap());
        let _ = e_log.push(&Ticker{
            dtg: d1.clone(),
            product_id: ProductId::BtcUsd,
            price: 88.87,
        });
        let d2 = DateTime::<Utc>::from(DateTime::parse_from_rfc3339("1997-12-19T16:39:57-08:00").unwrap());
        let _ = e_log.push(&Ticker{
            dtg: d2,
            product_id: ProductId::BtcUsd,
            price: 99.99,
        });
        let df = e_log.query_sql_all().await.unwrap();
        let vec_record_batch = df.collect().await.unwrap();
        let test_case = pretty_format_batches(vec_record_batch.as_slice()).unwrap().to_string();
        let expected_result =
            "+---------------------+------------+-------+
| dtg                 | product_id | price |
+---------------------+------------+-------+
| 1996-12-20T00:39:57 | BtcUsd     | 88.87 |
| 1997-12-20T00:39:57 | BtcUsd     | 99.99 |
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
        let expected_result =
            r"+------------+--------------------------------------------+-----------------------+--------+----------+
| dtg        | description                                | member                | amount | category |
+------------+--------------------------------------------+-----------------------+--------+----------+
| 2023-10-14 | 12105 DONNER PASS RDTRUCKEE             CA | Fenstemeier Fudpucker | 11.16  | car_gas  |
| 2023-10-14 | AplPay 12105 DONNER TRUCKEE             CA | Fenstemeier Fudpucker | 39.2   | car_gas  |
| 2023-10-14 | SP LOOP MOUNT       LONDON              GB | Fenstemeier Fudpucker | 69.0   | car      |
+------------+--------------------------------------------+-----------------------+--------+----------+".to_string();

        assert_eq!(test_case, expected_result);

    }
}
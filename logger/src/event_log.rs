//! main.rs
//!
//! Goal: use SQL to query an in-memory dataset (via Apache DataFusion and Apache Arrow)
//!
//! https://docs.rs/datafusion/latest/datafusion/datasource/memory/struct.MemTable.html
//!

use std::sync::Arc;
use std::time::Instant;
use datafusion::arrow::array::{Date64Array, Float64Array, PrimitiveArray, StringArray};
use datafusion::arrow::datatypes::{DataType, Date64Type, Field, Schema};
use datafusion::arrow::error::ArrowError;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::dataframe::DataFrameWriteOptions;
use datafusion::prelude::*;
use slice_ring_buffer::SliceRingBuffer;
use common_lib::cb_ticker::{Ticker};

#[allow(dead_code)]
const RING_BUF_SIZE:usize=100;

/// Ring buffer with ability to extract the entire buffer as a slice
/// https://docs.rs/slice-ring-buffer/0.3.3/slice_ring_buffer/
pub struct EventLog{
    log: SliceRingBuffer<Ticker>
}

#[allow(dead_code)]
impl EventLog{
    pub fn new()->EventLog{
        EventLog{
            log:SliceRingBuffer::<Ticker>::with_capacity(RING_BUF_SIZE),
        }
    }

    /// push into this custom event log
    pub fn push(&mut self, ticker:&Ticker)->Result<(), EventLogError>{
        self.log.push_back((*ticker).clone());
        Ok(())
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
    pub fn record_batch(&self) -> Result<RecordBatch, ArrowError> {
        let dates:Vec<i64> = self.log.iter().map(|x| x.dtg.timestamp_millis()).collect();
        let dates:PrimitiveArray<Date64Type> = Date64Array::from(dates);
        let product_ids:Vec<String> = self.log.iter().map(|x| (x.product_id.to_string()).clone()).collect();
        let product_ids:StringArray = StringArray::from(product_ids);
        let prices:Vec<f64> = self.log.iter().map(|x| x.price).collect();
        let prices:Float64Array = Float64Array::from(prices);

        RecordBatch::try_new(
            Arc::new(EventLog::schema()),
            vec![
                Arc::new(dates),
                Arc::new(product_ids),
                Arc::new(prices),
            ]
        )
    }

    /// select * from table
    pub async fn query_sql_all(&self) -> datafusion::error::Result<DataFrame> {
        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();
        let df = ctx.sql(r#"
            select * from t_one
        "#
        ).await?;

        Ok(df.clone())

    }

    /// select * from table
    pub async fn sql_count_all(&self) -> datafusion::error::Result<DataFrame> {
        let start = Instant::now();

        let mem_batch = self.record_batch().unwrap();
        let ctx = SessionContext::new();
        ctx.register_batch("t_one", mem_batch).unwrap();
        let df = ctx.sql(r#"
                select count, p100, p10, p100-p10 as diff from (
                    select
                        count(*) as count
                        ,(select avg(price) from (select price from t_one order by dtg desc limit 100)) as p100
                        ,(select avg(price) from (select price from t_one order by dtg desc limit 10)) as p10
                    from t_one
                )
        "#
        ).await?;

        // µs
        tracing::debug!("[sql] elapsed: {} ms", start.elapsed().as_micros() as f64/1000.0);

        Ok(df.clone())

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
pub enum EventLogError {
    PushError,
    OtherError,
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
        let _ = e_log.push(&Ticker{
            dtg: d1,
            product_id: ProductId::BtcUsd,
            price: 99.99,
        });
        let batch = e_log.record_batch().unwrap();
        // println!("batch: {:?}", &batch);
        let test_case = pretty_format_batches(&[batch]).unwrap().to_string();
        // println!("{}", &test_case);
        let expected_result =
            "+---------------------+------------+-------+
| dtg                 | product_id | price |
+---------------------+------------+-------+
| 1996-12-20T00:39:57 | BtcUsd     | 88.87 |
| 1996-12-20T00:39:57 | BtcUsd     | 99.99 |
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
        let _ = e_log.push(&Ticker{
            dtg: d1,
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
| 1996-12-20T00:39:57 | BtcUsd     | 99.99 |
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
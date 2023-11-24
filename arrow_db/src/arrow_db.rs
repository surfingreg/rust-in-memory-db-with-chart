//! arrow_db.rs

/*
        #[derive(Serialize)]
        struct MyStruct {
            int32: i32,
            string: String,
        }
        // aka i64
        let arrow_dates: Vec<<Date64Type as ArrowPrimitiveType>::Native> = vec![
            "2014-11-28T12:00:09Z".parse::<DateTime<Utc>>().unwrap().timestamp_millis(),
            "2014-11-28T12:00:09Z".parse::<DateTime<Utc>>().unwrap().timestamp_millis(),
            "2014-11-28T12:00:09Z".parse::<DateTime<Utc>>().unwrap().timestamp_millis(),
            "2014-11-28T12:00:09Z".parse::<DateTime<Utc>>().unwrap().timestamp_millis(),
            "2014-11-28T12:00:09Z".parse::<DateTime<Utc>>().unwrap().timestamp_millis()
        ];

        // let arrow_dates: Vec<<Date64Type as ArrowPrimitiveType>::Native> = dates.into_iter().map(|a| { a}).collect();
        let arrow_dates:PrimitiveArray<Date64Type> = arrow::array::Date64Array::from(arrow_dates);

        let chrono_naive_dates:Vec<NaiveDateTime> = arrow_dates.into_iter().map(|a| NaiveDateTime::from_timestamp_millis(a.unwrap() as i64).unwrap()).collect();
        tracing::debug!("**** {:?}", chrono_naive_dates);

        let id_array = Int32Array::from(vec![1, 2, 3, 4, 5]);
        let id_array2 = Int32Array::from(vec![1, 2, 3, 4, 5]);
        let names = arrow::array::StringArray::from(vec!["one", "two", "three", "four", "five"]);

        let schema = Schema::new(vec![
            arrow_schema::Field::new("dates", DataType::Date64, false),
            arrow_schema::Field::new("id", DataType::Int32, false),
            arrow_schema::Field::new("id2", DataType::Int32, false),
            arrow_schema::Field::new("name", DataType::Utf8, false),
        ]);

        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(arrow_dates), Arc::new(id_array), Arc::new(id_array2), Arc::new(names)]
        ).unwrap();


        // https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatch.html
        let dates: &Date64Array = batch.column_by_name("dates").unwrap().as_any().downcast_ref::<Date64Array>().unwrap();
        for date in dates{
            for d in date{
                let d:i64 = d as i64;
                let d = NaiveDateTime::from_timestamp_millis(d).unwrap();
                tracing::debug!("{:?}", d);
            }
        }
*/


use std::sync::Arc;
use arrow::array::{Array, Date64Array, Float64Array, PrimitiveArray, StringArray};
use arrow::datatypes::{Date64Type};
use arrow::record_batch::RecordBatch;
use arrow_schema::{ArrowError, DataType, Schema};
use chrono::{NaiveDateTime};
use crossbeam_channel::{Sender, unbounded};
use common_lib::cb_ticker::{Ticker};
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;
use serde::Serialize;
use slice_ring_buffer::SliceRingBuffer;

// TODO: consider arc/mutex vice serial message parsing
/// fixed-size ring buffer with ability to extract the entire buffer as a slice
/// https://docs.rs/slice-ring-buffer/0.3.3/slice_ring_buffer/
pub struct EventLog{
    // log:Vec<Ticker>
    log: SliceRingBuffer<Ticker>
}

impl EventLog{
    fn new()->EventLog{
        EventLog{
            log:SliceRingBuffer::<Ticker>::with_capacity(100),
            // log:vec![]
        }
    }

    fn push(&mut self, ticker:&Ticker)->Result<(), EventLogError>{
        self.log.push_back((*ticker).clone());
        Ok(())
    }

    fn schema() -> Schema {
        let schema = Schema::new(vec![
            arrow_schema::Field::new("dtg", DataType::Date64, false),
            arrow_schema::Field::new("product_id", DataType::Utf8, false),
            arrow_schema::Field::new("price", DataType::Float64, false),
        ]);
        schema
    }

    fn record_batch(&self) -> Result<RecordBatch, ArrowError> {
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

    fn print_record_batch(&self){
        // https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatch.html

        match self.record_batch(){
            Ok(batch)=> {
                tracing::debug!("[print_record_batch] rows: {}", batch.num_rows());
                let dates: &Date64Array = batch.column_by_name("dtg").unwrap().as_any().downcast_ref::<Date64Array>().unwrap();
                for date in dates {
                    for d in date {
                        let d: i64 = d as i64;
                        let d = NaiveDateTime::from_timestamp_millis(d).unwrap();
                        tracing::debug!("[print_record_batch] {:?}", d);
                    }
                }
            },
            Err(e)=>{
                tracing::debug!("[print_record_batch] error: {:?}", &e);
            }

        }
    }
}

pub enum EventLogError{
    PushError,
    OtherError,
}


/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub fn run() -> Sender<Msg> {
    let (tx, rx) = unbounded();

    let mut event_log = EventLog::new();

    let tx2 = tx.clone();
    std::thread::spawn(move || {
        let _ = start_heartbeat(tx2);

        // state holder v1
        // let mut message_vec:Vec<Ticker> = vec![];
        // let state_ref = &mut message_vec;

        loop{
            match rx.recv(){
                Ok(message)=> {
                    // process_message(message, state_ref)
                    match message{
                        Msg::Ping => {
                            tracing::debug!("[arrow_db] PING");
                        },
                        Msg::Post(ticker)=>{
                            tracing::debug!("[arrow_db] POST {:?}", &ticker);

                            event_log.log.push_back(ticker.clone());
                            event_log.print_record_batch();

                            // state_ref.push(ticker);
                            // tracing::debug!("[arrow_db] total in memory: {}", state_ref.len());
                        },
                        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
                    }
                },
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

fn process_message(message:Msg, state_ref: &mut Vec<Ticker>){
    match message{
        Msg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        Msg::Post(ticker)=>{
            tracing::debug!("[arrow_db] POST {:?}", &ticker);
            state_ref.push(ticker);
            tracing::debug!("[arrow_db] total in memory: {}", state_ref.len());
        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}

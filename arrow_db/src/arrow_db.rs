//! arrow_db.rs

use std::sync::Arc;
use arrow::array::{Date64Array, Float64Array, PrimitiveArray, StringArray};
use arrow::datatypes::{Date64Type};
use arrow::record_batch::RecordBatch;
use arrow_schema::{ArrowError, DataType, Schema};
// use chrono::{NaiveDateTime};
use crossbeam_channel::{Sender, unbounded};
use common_lib::cb_ticker::{Ticker};
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;
use slice_ring_buffer::SliceRingBuffer;


const RING_BUF_SIZE:usize=100;

//// This doesn't remain fixed size???
/// fixed-size ring buffer with ability to extract the entire buffer as a slice
/// https://docs.rs/slice-ring-buffer/0.3.3/slice_ring_buffer/
pub struct EventLog{
    // log:Vec<Ticker>
    log: SliceRingBuffer<Ticker>
}

impl EventLog{
    fn new()->EventLog{
        EventLog{
            log:SliceRingBuffer::<Ticker>::with_capacity(RING_BUF_SIZE),
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

                // print date properly
                // let dates: &Date64Array = batch.column_by_name("dtg").unwrap().as_any().downcast_ref::<Date64Array>().unwrap();
                // for date in dates {
                //     match date {
                //         Some(d)=> {
                //             let d: i64 = d as i64;
                //             let d = NaiveDateTime::from_timestamp_millis(d).unwrap();
                //             tracing::debug!("[print_record_batch] {:?}", d);
                //         },
                //         None=>{}
                //     }
                // }
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

        loop{
            match rx.recv(){
                Ok(message)=> {
                    process_message(message, &mut event_log)
                },
                Err(e)=> tracing::debug!("[arrow_db] error {:?}", &e),
            }
        }
    });
    tx
}

fn process_message(message:Msg, event_log: &mut EventLog){
    match message{
        Msg::Ping => {
            tracing::debug!("[arrow_db] PING");
        },
        Msg::Post(ticker)=>{
            tracing::debug!("[arrow_db] POST {:?}", &ticker);

            let _ = event_log.push(&ticker);
            event_log.print_record_batch();
        },
        _ => tracing::debug!("[arrow_db] {:?} UNKNOWN ", &message)
    }
}

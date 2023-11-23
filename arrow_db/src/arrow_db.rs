//! arrow_db.rs

use std::ops::Deref;
use std::sync::Arc;
use arrow::array::{Array, Date64Array, Int32Array, PrimitiveArray};
use arrow::datatypes::{ArrowPrimitiveType, Date64Type, Int64Type};
use arrow::record_batch::RecordBatch;
use arrow_schema::{DataType, Schema};
use arrow_schema::DataType::Date64;
use chrono::{DateTime, NaiveDateTime, Utc};
use crossbeam_channel::{Sender, unbounded};
use common_lib::cb_ticker::Ticker;
use common_lib::heartbeat::start_heartbeat;
use common_lib::operator::Msg;
use serde::Serialize;


#[derive(Serialize)]
struct MyStruct {
    int32: i32,
    string: String,
}

/// spawn a thread to listen for messages; return the channel to communicate to this thread with.
pub fn run() -> Sender<Msg> {
    let (tx, rx) = unbounded();
    let tx2 = tx.clone();
    std::thread::spawn( move ||{
        let _ = start_heartbeat(tx2);


        // let schema = Schema::new(vec![
        //     Field::new("int32", DataType::Int32, false),
        //     Field::new("string", DataType::Utf8, false),
        // ]);
        //
        // let rows = vec![
        //     MyStruct{ int32: 5, string: "bar".to_string() },
        //     MyStruct{ int32: 8, string: "foo".to_string() },
        // ];
        //

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

            // tracing::debug!("{:?}", &date);

            // tracing::debug!("[arrow] batch: {:?}", chrono::NaiveDateTime::from_timestamp_millis(date as i64));

        }




        // state holder v1
        let mut message_vec:Vec<Ticker> = vec![];

        let state_ref = &mut message_vec;
        loop{
            match rx.recv(){
                Ok(message)=> process_message(message, state_ref),
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

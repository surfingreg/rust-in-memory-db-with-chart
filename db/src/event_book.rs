//! event_book.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use common_lib::cb_ticker::{Ticker, TickerCalc};
use crate::event_log::EventLog;


/// Container for multiple event logs keyed by a string
pub struct EventBook {
    pub book: Arc<RwLock<HashMap<String, EventLog>>>,
}
impl Default for EventBook {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBook {
    pub fn new() -> EventBook {
        EventBook {
            book: Arc::new(RwLock::new(HashMap::<String, EventLog>::new())),
        }
    }

    /// get write lock on the entire book and insert a new record
    pub fn push_log(&self, key: &str, val: &Ticker) -> Result<(), BookError> {
        // write lock
        let mut book_writable = self.book.write().unwrap();

        match book_writable.get_mut(key) {
            Some(event_log) => {
                // an event log exists for this key

                // TODO un-unwrap
                event_log.push_log(val).unwrap();
                Ok(())
            }
            None => {
                // an event log does not exist for this key; create it

                // 1. create a new event log since there's none for this key
                let mut new_e_log = EventLog::new();

                // 2. put the ticker in the new event log
                match new_e_log.push_log(val) {
                    Ok(_) => {
                        // 3. put the new event log with new ticker in the hashmap
                        // Option<previous> or none returned
                        book_writable.insert(key.to_string(), new_e_log);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("[push] event log push error: {:?}", &e);
                        Err(BookError::General)
                    }
                }
            }
        }
    }

    /// get write lock on the entire book and insert a new record
    pub fn push_calc(&self, key: &str, val: &TickerCalc) -> Result<(), BookError> {
        // tracing::debug!("[push_calc]");

        // write lock
        let mut book_writable = self.book.write().unwrap();

        // tracing::debug!("[push_calc] got write lock");
        match book_writable.get_mut(key) {
            Some(calc_log) => {
                // an event log exists for this key
                // TODO un-unwrap
                calc_log.push_calc(val).unwrap();
                // tracing::debug!("[push_calc] calc log size: {}", calc_log.len());
                Ok(())
            }
            None => {
                // an event log does not exist for this key; create it

                // 1. create a new event log since there's none for this key
                let mut new_e_log = EventLog::new();

                // 2. put the ticker in the new event log
                match new_e_log.push_calc(val) {
                    Ok(_) => {
                        // 3. put the new event log with new ticker in the hashmap
                        // Option<previous> or none returned
                        book_writable.insert(key.to_string(), new_e_log);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("[push] event log push error: {:?}", &e);
                        Err(BookError::General)
                    }
                }
            }
        }

    }
}

#[derive(Debug)]
pub enum BookError {
    General,
}

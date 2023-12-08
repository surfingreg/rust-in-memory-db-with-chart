# Time-Series Visualiztion in Rust

## Task
Implement a fast, in-memory, time-series database. Query and visualize the data using simple SQL.

## Key Technology
- Rust--for a fast in-memory "database"
- Apache Arrow, DataFusion--to query plain old data structures with SQL

## Discussion

This project subscribes to an unauthenticated Coinbase feed for price data. The front end at
http://127.0.0.1:8080 presents various naive moving average curves. 

While extremely useful, DataFusion currently takes 100x as long to execute a query like this 
```select avg(price) as from my_table limit 1000``` versus raw Rust code like

```
let slice_max = 100;
let slice_n: &[Ticker] = &self.log.as_slice()[0..slice_max];
let avg_n: f64 = slice_n.iter().map(|x| x.price).sum::<f64>() / slice_4.len() as f64;
```
Obviously parsing SQL on the fly is no joke. And, would never compete with compiled simple math. Plus, I'm surely not using DataFustion (yet) in any kind of smart or optimized way.

## TODO
- [] Convert current Coinbase example into an example
- [] Save previously calculated averages in a secondary in-memory vector
- [] Display the current and average price in frontend chart
# Time-Series Visualiztion in Rust

## Task
Implement a fast, in-memory, time-series database. Query data using SQL and visualize in ChartJS over websocket.

## Key Technology
- Rust--for a fast in-memory "database"
- Apache Arrow, DataFusion--to query plain old data structures with SQL

## Discussion

After reading up on InfluxDB and their recent adoption of the “FDAP” stack, I wanted to try Apache DataFusion for myself. Primarily, I wanted to see if it’d be useful in converting my stock trading platform from Postgres to an a more (bespoke) in-memory database. My initial observations on extremely naive queries is that it’s 10-100 times faster to write simple queries in raw Rust than to do something like ```select avg(price) from my_table```. I definitely don’t claim I don’t have a ton to learn both about the Apache Arrow stack and implementing an in-memory database from scratch. But, here’s a working sample: Coinbase web socket data is collected, stored in a ring vector, then moving averages are built either with Rust filter/map or SQL via DataFusion. I also bolted on a ChartJS display to present the live calculations as a time series broadcast over websocket.

[1] https://www.influxdata.com/blog/flight-datafusion-arrow-parquet-fdap-architecture-influxdb/

## Usage

http://127.0.0.1:8080 

## Results

While extremely useful, DataFusion currently takes 100x as long to execute a query like this 
```select avg(price) as from my_table limit 1000``` versus raw Rust code like

```
let slice_max = 100;
let slice_n: &[Ticker] = &self.log.as_slice()[0..slice_max];
let avg_n: f64 = slice_n.iter().map(|x| x.price).sum::<f64>() / slice_4.len() as f64;
```

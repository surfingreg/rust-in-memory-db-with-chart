## Non-blocking Tungstenite websockets without Tokio

Usage:
```cargo test```

In response to events that occur in another thread, send an outgoing 
websocket broadcast. Use case will be to update a stock ticker chart
with data and calculation data points.


Send ping to the server:
```websocat -v --ping-interval=2 ws://127.0.0.1:3012/socket```

Receive server broadcasts (run in multiple windows):
```websocat -v ws://127.0.0.1:3012/socket```
# Limit.rs

Contains a contrived example of a WASM frontend calling an Axum HTTP server, based heavily on [this fantastic blog post](https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/).

Start the server with

```
cargo run --bin backend -- --static-dir "./dist" --port 8081
```

Serve the frontend with

```
cd frontend; trunk serve --proxy-backend=http://[::1]:8081/api/
```

TODO list:
- add a button to click which calls the backend to get a number
- use [Nucleon](https://github.com/NicolasLM/nucleon) to load balance many backend instances
- use `libp2p`, following the [tutorial on ping](https://docs.rs/libp2p/latest/libp2p/tutorials/ping/index.html), to sync state between backend peers
- add rate limiting based on libp2p sharing


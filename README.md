# Cloud Sync

Small web service with one endpoint: /wait-for-second-party/:unique-id

This endpoint allows two parties to sync.
When one party makes a POST request, the response will be delayed until the second party requests the same URL. In other
words, the first party is blocked until the second party arrives or a timeout occurs (let it be 10 seconds).

Rust: tokio + warp

## Why this project exists

This is project presenting simple stateful api written in Rust created for educational purposes.

Reading this code you can learn how to create server in rust and write tests for it.

## How to run

In first console start server:

```
cargo run
```

In second and third console experiment with http requests like:

```
http POST http://127.0.0.1:3030/wait-for-second-party/test-id
```

You should be able to reproduce steps from demo presented below

![./docs/demo.gif](./docs/demo.gif)
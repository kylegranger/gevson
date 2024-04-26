# gevson

## Introduction

`gevson` is a binary that sits between a client that has proofs to be proven and the Gevulot network.

It receives `ProofRequests` over a websocket connection and returns prover results.

`gevson` has a WebWocket server


## Running `gevson`

Command line options

```
Usage: gevson [OPTIONS]

Options:
  -j, --jsonurl <JSONURL>  RPC url of the Gevulot node [default: http://localhost:9944]
  -d, --datadir <DATADIR>  Data directory to store downloaded files [default: ./data ]
  -p, --port <PORT>        Port for the WebSocket server [default: 8080 ]
  -h, --help               Print help
  -V, --version            Print version
```

This will build and start up `gevson`, running a server on port 8080.


```
cargo run
```

To use `gevson`, just open a websocket connection.

This one website (there are several) for communicating with a ws server from a browser: https://piehost.com/websocket-tester

Just paste in: `ws://localhost:8080`

After you connect, type some random text in and you'll get an error message back.

Here it is formatted:

```
{
    "duration_in_ms": 0,
    "message": "Could not parse message as ProofRequest",
    "success": false,
    "tx_result": null
}
```



## File upload template strings

[todo]

# gevson

## Short term goal

Bang out a first iteration in Rust:

- WS server (bidirectional, session based)
  - configuration (to start off with)
    - Gevulot RPC endpoint
    - S3 parameters
    - GCP integration
    - data directory
- accepts proof requests
  - witness
    - url, or
    - bytes
  - schema (`katla`, `polygon`, `sp1`, `mock`, ...)
  - deployed prover/verifier hashes
  - timeout
  - executes proofs
    - name and upload witness
    - compute checksum
    - call exec
    - poll for task result
    - fetch proof
- returns a proof result
- support multiple sessions, proof requests, active txhashes

For development and testing, I would migrate the currently running Taiko tasker be a light-client that calls this new WS endpoint.

## Environment variables

```
export GEV_UPLOAD_CMD='s3cmd put --acl-public UPLOAD_PATH s3://gevulot-test/UPLOAD_FILE'
export GEV_UPLOAD_URL='https://gevulot-test.eu-central-1.linodeobjects.com/UPLOAD_FILE'


RUST_LOG=trace ./target/debug/gevson -n named  -s katla -f gevulot/testwit.json -t 6 -d ./data


RUST_LOG=trace ./target/debug/gevson -n named  -s katla -u https://gevulot-test.eu-central-1.linodeobjects.com/named -t 6 -d ./data

RUST_LOG=trace ./target/debug/gevson -r gevulot/request.json -d ./data

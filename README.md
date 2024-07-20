# Basin Demo Server

> A simple server for [Basin](https://github.com/textileio/basin) object storage

## Background

This demo server allows a backend wallet to "sponsor" Basin transactions on behalf of a user. Both pushing and getting objects is supported.

## Usage

### Prerequisites

Before getting started, you'll need to make sure you're properly set up on the Basin subnet. Start by installing the Basin CLI tool and creating a new wallet:

```sh
git clone https://github.com/textileio/basin
make build
make install
```

You can either create a new account _or_ use an existing EVM-style (secp256k1) private key. Creating a new account can be done with the CLI:

```sh
adm account create
```

This logs the private key plus its EVM and FVM address:

```
{
  "private_key": "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
  "address": "0x70997970c51812dc3a010c7d01b50e0d17dc79c8",
  "fvm_address": "t410focmxs4gfdajnyoqbbr6qdniobul5y6oirvks3ia"
}
```

Then, make sure you have a wallet with some tFIL (i.e., on Filecoin Calibration) in it. Head over to the Calibration faucet [here](https://faucet.calibnet.chainsafe-fil.io/), request some tFIL, and then `deposit` it into your subnet account. Once the funds have arrived on the subnet, you'll be able to check your balance:

```sh
export PRIVATE_KEY=your_private_key
adm account deposit 10
adm account info
```

We deposit 10 tFIL into the account and after ~30 minutes (the current constraint imposed on the Calibration -> subnet checkpointing process), you should see the balance updated. That is, be sure to wait until `info` logs the balance!

### Setup

Now, we can get things going. Check out the `env.example` file, update the `PRIVATE_KEY`, and create a new `env` file with these values. There is a provided object store address (`OS_ADDRESS`) that you can use, which has fully public write enabled.

```sh
export PRIVATE_KEY=hex_encoded_private_key
export LISTEN=127.0.0.1:8081
export OS_ADDRESS=t2ymaz2yovxlqplqd53tfuiw4umwpdt7tfmbf3v7q
export NETWORK=testnet
```

If you'd like to create your own object store, you can do so with the CLI—and be sure the `PRIVATE_KEY` is set before attempting to send transactions!

```sh
adm os create
```

Be sure to source the `env` file before running the server:

```sh
source env
```

### Running the server

First, build and install the server binary:

```sh
make build
make install
```

Then, you can start it with the `env` settings:

```sh
basin_server -vv
```

The `-vv` enables verbose logging, which can be helpful for debugging:

```sh
2024-07-20T17:49:27.589-04:00 - INFO Starting server at 127.0.0.1:8081
2024-07-20T17:50:12.015-04:00 - INFO {"body":"{\"multipart/form-data; boundary=------------------------u3Cayud8pzT4bsvlrHH4Z5\"}","route":"set"}
2024-07-20T17:50:14.691-04:00 - INFO {"client_addr":"127.0.0.1:50064","duration_ms":2676,"method":"POST","path":"/set","status":200}
2024-07-20T17:50:33.952-04:00 - INFO {"body":"{prefix: Some(\"hello/\"), delimiter: None, offset: None, limit: Some(10)}","route":"list"}
2024-07-20T17:50:34.371-04:00 - INFO {"client_addr":"127.0.0.1:50068","duration_ms":419,"method":"POST","path":"/list","status":200}
```

There are two routes enabled:

- `POST /set`: Upload an object to the object store
- `POST /list`: Get an object from the object store

A maximum value of 100 MB is fixed for the server. Within the `src/server/set.rs` file, you can adjust by changing the `MAX_FILE_SIZE` constant.

### Client requests

To put a file in the object store, use the `/set` endpoint with multipart form data and fields for the uploading:

- `address`: The address of the requesting user (e.g., for attribution purposes).
- `key`: Custom key for the object.
- `file`: The local filepath.

```sh
curl -X POST -H 'Content-Type: multipart/form-data' \
--form 'address=0x79447b8db3a9d23f7db75ae724ba450b7b8dd7b0' \
--form 'key=hello/test' \
--form 'file=@test.dat' \
http://localhost:8081/set
```

This will log the transaction information from the Basin subnet:

```json
{
  "data": "bafy2bzacedxeu3g3uazqpn2ln7yvyfhc6ilj3vi5bf3h6usvygsxaub7paws4",
  "gas_used": 4311212,
  "hash": "1DDBED9D0398C4A7C0B2E0DE99BCE77C34232CC1AD45E9304F990A416ACAF830",
  "height": "956895",
  "status": "committed"
}
```

You can list objects in the object store along with a query filters:

- `prefix`: Prefix to filter objects by.
- `limit`: Maximum number of objects to list.
- `delimiter`: Delimiter used to define object hierarchy.
- `offset`: Offset to start listing objects from.

```sh
curl -X POST -H 'Content-Type: application/json' \
-d '{"prefix": "hello/", "limit": 10}' \
http://localhost:8081/list
```

The response will provide all matching objects under that specific prefix:

```json
{
  "common_prefixes": [],
  "objects": [
    {
      "key": "hello/world",
      "value": {
        "cid": "bafybeid3weurg3gvyoi7nisadzolomlvoxoppe2sesktnpvdve3256n5tq",
        "metadata": {},
        "resolved": true,
        "size": 5
      }
    }
  ]
}
```

Alteratively, you can list all objects with default settings by providing no query parameters:

```sh
curl -X POST -H 'Content-Type: application/json' \
http://localhost:8081/list
```

## Development

Local development isn't _quite_ enabled yet, so you'll have to use the public Filecoin Calibration testnet and Basin subnet setup.

All the available Makefile commands include:

- Build all crates: `make build`
- Install the CLI: `make install`
- Run linter: `make lint`
- Run formatter: `make check-fmt`
- Run clippy: `make check-clippy`
- Do all of the above: `make all`
- Clean dependencies: `make clean`

Only basic `INFO` and `ERROR` logging is implemented. See the `log_request_details` function in `src/server/utils.rs` for more information.

## Contributing

PRs accepted.

Small note: If editing the README, please conform to the [standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT OR Apache-2.0, © 2024 Textile Contributors

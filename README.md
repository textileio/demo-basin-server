# Basin Demo Server

> A simple server for [Basin](https://github.com/textileio/basin) object storage

## Background

This demo server allows a backend wallet to "sponsor" Basin transactions on behalf of a user. Both pushing and getting objects is supported.

## Usage

### Prerequisites

Before getting started, you'll need to make sure you're properly set up on the Basin subnet. Start by installing the Basin CLI tool and creating a new wallet:

```sh
git clone https://github.com/textileio/basin
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

Now, we can get things going. Check out the `env.example` file and update the `PRIVATE_KEY`. There is a provided object store address (`OS_ADDRESS`) that you can use, which has fully public write enabled.

```sh
export PRIVATE_KEY=hex_encoded_private_key
export LISTEN=127.0.0.1:8081
export OS_ADDRESS=t2ymaz2yovxlqplqd53tfuiw4umwpdt7tfmbf3v7q
export NETWORK=testnet
```

If you'd like to create your own object store, you can do so with the CLI—and be sure the `PRIVATE_KEY` is set!

```sh
adm os create
```

### Running the server

Now, you can start the server:

```sh
cargo build
adm_server -vv
```

The `-vv` enables verbose logging, which can be helpful for debugging:

```sh
2024-07-19T18:26:39.162-04:00 - INFO Starting server at 127.0.0.1:8081
2024-07-19T18:26:48.795-04:00 - INFO {"body":"\"multipart/form-data; boundary=------------------------uSkTaZdBvPUiwVg1qOO6fz\"","route":"new req"}
os address: t2ymaz2yovxlqplqd53tfuiw4umwpdt7tfmbf3v7q
2024-07-19T18:26:51.262-04:00 - INFO {"client_addr":"127.0.0.1:60821","duration_ms":2466,"method":"POST","path":"/set","status":200}
2024-07-19T18:45:42.847-04:00 - INFO {"client_addr":"127.0.0.1:61314","duration_ms":636,"method":"POST","path":"/get","status":200}
```

There are two routes enabled:

- `POST /set`: Upload an object to the object store
- `POST /get`: Get an object from the object store

A maximum value of 100 MB is fixed for the server. Within the `src/server/set.rs` file, you can adjust by changing the `MAX_FILE_SIZE` constant.

### Client requests

To put a file in the object store, use the `/set` endpoint with multipart form data and fields for the uploading `address`, `key`, and `file` path:

```sh
curl -X POST -H 'Content-Type: multipart/form-data' \
--form 'address=0x79447b8db3a9d23f7db75ae724ba450b7b8dd7b0' \
--form 'key=1234abcd' \
--form 'file=@test.dat' \
'http://localhost:8081/set'
```

This will log the transaction information:

```sh
{
  "data": "bafy2bzacedn6bzzjgtzcskpeofjrirvxrisv7x2rokku35kiclpmrazqzyeji",
  "gas_used": 4565634,
  "hash": "F652F998705F835FC784FC00B9583CBDCF21D27EC74C004404CB0B7EE057E54C",
  "height": "880695",
  "status": "committed"
}
```

You can list all objects by simply hitting the `/list`` endpoint:

```sh
curl -X POST -H 'Content-Type: application/json' \
--data-raw '{"key":"hello/test"}' \
'http://localhost:8081/list'
```

This will log something like:

```sh
[
  {
    "key": "hello",
    "value": {
      "cid": "bafybeiffndsajwhk3lwjewwdxqntmjm4b5wxaaanokonsggenkbw6slwk4",
      "metadata": {},
      "resolved": true,
      "size": 6
    }
  }
]
```

## Development

Local development isn't _quite_ enabled yet, so you'll have to use the public Filecoin Calibration testnet and Basin subnet setup.

## Contributing

PRs accepted.

Small note: If editing the README, please conform to
the [standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

MIT OR Apache-2.0, © 2024 Textile Contributors

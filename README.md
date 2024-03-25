# Memcachr: A memcached server implementation in Rust

*Note: This is mostly intended as a learning project and is not recommended for
a production use case.*

### Currently implemented commands

- `get <key>`
- `set <key> <flags> <expiration> <bytecount> [noreply]\r\n<data block>\r\n`

### Running and testing

In order to run an automated integration test, just clone this repo and run
`tox` in the root directory (you can install `tox` using `pip install tox`). In
order to manually test this, follow the instructions.

#### Prerequisites:

- rust, cargo
- python, pymemcache (`pip install pymemcache`)

#### Steps:

1. Ensure that you have rust and cargo installed and execute `cargo run`. Ensure
   that the process starts successfully.

2. In a new tab, open a python shell and run the following commands.

```
from pymemcache.client.base import Client
client = Client(('127.0.0.1', 11212))
client.set('somekey', 'somevalue')
assert 'somevalue' == client.get('somekey')
assert None == client.get('otherkey')
```

### Next steps

- add script to benchmark performance and memory utilization against memcached.
- add more extensive unit and integration testing.
- implement the remaining commands in the
  [memcached protocol](https://github.com/memcached/memcached/blob/master/doc/protocol.txt).

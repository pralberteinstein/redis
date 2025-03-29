# Redis Clone in Rust

A simple Redis-like server implemented in Rust with TCP protocol support. This implementation provides an in-memory key-value store with basic Redis commands.

## Features

- TCP server listening for connections
- Simple text-based protocol for commands
- In-memory hash map for storing key-value pairs
- Basic Redis commands: GET, SET, DEL, EXISTS, PING, KEYS, EXPIRE, TTL, FLUSHALL
- Key expiration (TTL) support
- Simple pattern matching for KEYS command

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version recommended)

### Installation

Clone this repository:

```bash
git clone https://github.com/yourusername/redis-rust.git
cd redis-rust
```

### Running the Server

Start the Redis server with the convenience script:

```bash
./start_server.sh
```

The script accepts the following options:
- `--port=PORT` - Set the Redis server port (default: 6379)
- `--log=LEVEL` - Set logging level (default: info)

Or run the server directly:

```bash
cargo run
```

By default, the server will listen on `127.0.0.1:6379`. You can change the port by setting the `REDIS_PORT` environment variable:

```bash
REDIS_PORT=6380 cargo run
```

You can also configure logging level by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

### Using the Test Client

This project includes a simple test client:

```bash
cargo run --bin client
```

### Trying the Demo Script

For a quick demonstration of the server's capabilities, run:

```bash
./redis_demo.sh
```

The demo script will run various Redis commands and display their outputs. You need to have the server running separately before running the demo.

## Supported Commands

- `SET key value` - Set a key-value pair
- `SET key value EX seconds` - Set a key with an expiration time
- `GET key` - Get the value for a key
- `DEL key` - Delete a key
- `EXISTS key` - Check if a key exists (returns 1 if exists, 0 if not)
- `EXPIRE key seconds` - Set a key's time to live in seconds
- `TTL key` - Get the remaining time to live of a key
- `KEYS pattern` - Find all keys matching the pattern (e.g., KEYS *)
- `FLUSHALL` - Remove all keys from the database
- `PING` - Test server connection
- `HELP` - Display available commands

## Example Usage

```
> SET name John
OK
> GET name
John
> SET session123 active EX 30
OK
> TTL session123
30
> EXISTS name
1
> KEYS *
name
session123
> DEL name
1
> GET name
(nil)
> FLUSHALL
OK
```

## Running Tests

To run the integration tests:

```bash
cargo test
```

The tests verify basic functionality such as:
- Setting and getting values
- Key expiration
- Pattern matching with KEYS
- Database clearing with FLUSHALL

## Implementation Details

- Uses Tokio for async I/O
- Thread-safe in-memory storage with Mutex
- Simple text-based protocol (not RESP)
- Automatic key expiration with background cleanup task

## Performance Considerations

- The server is designed for learning purposes and might not handle high loads
- For production use, consider using the actual Redis server

## License

This project is licensed under the MIT License - see the LICENSE file for details.

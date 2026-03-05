# amps-rust-ffi

Safe Rust bindings for the [AMPS](https://crankuptheamps.com) (Advanced Message Processing System) C++ client library.

> **⚠️ WARNING: This library is entirely vibe-coded and is NOT safe for production use.** It was built as an experiment with AI-assisted development. There are likely memory safety issues, undefined behavior at the FFI boundary, and untested edge cases. Use at your own risk — you have been warned.

## Features

- **Publish / Subscribe** — send and receive messages on topics
- **Filtered Subscriptions** — server-side content filtering (e.g. `/price > 100`)
- **SOW Queries** — query cached State-of-the-World data
- **SOW and Subscribe** — get current state then receive live updates
- **Delta Publish** — send only changed fields
- **Heartbeat** — connection health monitoring
- **Typed Errors** — every AMPS exception maps to an `AmpsError` variant

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
amps-rust-ffi = "0.1.0"
```

Or use `cargo add`:

```bash
cargo add amps-rust-ffi
```

## Prerequisites

- **Rust 1.70+** with Cargo
- **C++ compiler** (Clang 10+, GCC 9+, or MSVC 2019+)
- **CMake 3.16+**

> **Note:** The AMPS C++ client libraries are bundled with the crate, so you don't need to download them separately for building. However, to run the library, you will need an AMPS server to connect to.

## Getting Started

### 1. Create a new project

```bash
cargo new my-amps-app
cd my-amps-app
```

### 2. Add the dependency

```bash
cargo add amps-rust-ffi
```

### 3. Write some code

```rust
use amps_rust_ffi::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new("my-app")?;
    client.connect("tcp://localhost:9007/amps/json")?;
    client.logon(None, 5000)?;

    // Subscribe to a topic
    client.subscribe("orders", None, |msg| {
        println!("Received: {}", msg.data());
    })?;

    // Publish a message
    client.publish("orders", r#"{"symbol": "AAPL", "qty": 100}"#)?;

    client.disconnect()?;
    Ok(())
}
```

### 4. Run with an AMPS server

```bash
# Start an AMPS server (example using Docker)
docker run -p 9007:9007 -p 8085:8085 crankuptheamps/amps:latest

# Run your app
cargo run
```

## Examples

The [`amps-example`](https://crates.io/crates/amps-example) crate provides runnable examples:

```bash
cargo install amps-example

# Run the chat example (requires AMPS server)
amps-examples --help
```

Or run directly with cargo:

```bash
cargo install amps-example
amps-examples  # Shows available examples
```

A ready-to-run chat CLI is included. Two users can chat over a shared AMPS topic:

```bash
# Terminal 1
amps-example --bin chat -- alice

# Terminal 2  
amps-example --bin chat -- bob
```

## Usage

### Publishing

```rust
// Basic publish
client.publish("orders", r#"{"id": "1", "price": 150.0}"#)?;

// Publish with expiration (seconds)
client.publish_with_expiration("orders", r#"{"id": "2", "price": 200.0}"#, 300)?;

// Delta publish — only the changed fields
client.delta_publish("orders", r#"{"id": "1", "price": 155.0}"#)?;
```

### Subscribing

```rust
use std::sync::{Arc, Mutex};

let received = Arc::new(Mutex::new(Vec::new()));
let received_clone = received.clone();

// Subscribe with a server-side filter
client.subscribe("orders", Some("/price > 100"), move |msg| {
    println!("[{}] {}", msg.topic(), msg.data());
    received_clone.lock().unwrap().push(msg.data().to_string());
})?;

// Unsubscribe when done
client.unsubscribe_all()?;
```

### SOW Queries

```rust
// Query cached state with a filter
client.sow("orders", Some("/symbol = 'AAPL'"), |msg| {
    if msg.has_data() {
        println!("SOW record: {}", msg.data());
    }
})?;

// SOW and subscribe — get current state, then receive live updates
client.sow_and_subscribe("orders", None, |msg| {
    if msg.has_data() {
        println!("[{}] {}", msg.command(), msg.data());
    }
})?;
```

### Error Handling

All operations return `AmpsResult<T>`. Errors can be matched by variant:

```rust
use amps_rust_ffi::{Client, AmpsError};

let mut client = Client::new("my-app")?;
match client.connect("tcp://bad-host:9007/amps/json") {
    Ok(_) => println!("Connected"),
    Err(AmpsError::ConnectionRefused { message }) => eprintln!("Refused: {message}"),
    Err(AmpsError::TimedOut { message }) => eprintln!("Timeout: {message}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

### Heartbeat

```rust
client.set_heartbeat(30, 60)?; // send every 30s, read timeout 60s
```

### Thread Safety

`Client` implements `Send` but not `Sync`. You can move a client to another thread, but sharing across threads requires a `Mutex`:

```rust
let handle = std::thread::spawn(move || {
    client.publish("topic", r#"{"from": "background"}"#).unwrap();
    client.disconnect().unwrap();
});
handle.join().unwrap();
```

## Development (Building from Source)

If you want to build this crate from source instead of using the published version:

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/amps-rust-ffi
cd amps-rust-ffi
```

### 2. Set up the AMPS C++ client (for development)

```bash
# Extract so that amps-client/include/amps/ampsplusplus.hpp exists
tar -xzf amps-c++-client-5.3.5.1-*.tar.gz
mv amps-c++-client-5.3.5.1-* amps-client
```

### 3. Build

```bash
# Build the C++ wrapper
mkdir -p c-wrapper/build && cd c-wrapper/build
cmake ..
make
cd ../..

# Build the Rust library
cargo build
```

### 4. Run tests

Tests require a running AMPS server:

```bash
docker-compose -f tests/docker/docker-compose.yml up -d
cargo test
```

## Project Structure

```
amps-rust-ffi/
├── amps-client/            # AMPS C++ client (bundled for crates.io)
├── c-wrapper/
│   ├── include/amps_ffi.h  # C-compatible FFI header
│   ├── src/amps_ffi.cpp    # C++ wrapper implementation
│   └── CMakeLists.txt
├── src/
│   ├── lib.rs              # Library root & re-exports
│   ├── ffi/                # Auto-generated bindgen bindings
│   ├── client.rs           # Safe Client wrapper
│   ├── error.rs            # AmpsError enum
│   ├── message.rs          # Message type
│   └── subscription.rs     # Subscription handling
├── example/                # Example programs & chat CLI
└── tests/
    └── docker/             # AMPS server for integration tests
```

## Publishing to Crates.io

```bash
# Dry-run to check packaging
cargo publish --dry-run

# Publish
cargo publish
```

> **Note:** The AMPS C++ client library is a proprietary dependency that is bundled as pre-built static libraries for convenience. The crate includes libraries for macOS (arm64) and can be extended for other platforms.

## License

MIT

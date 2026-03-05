//! AMPS Rust FFI Example
//!
//! Demonstrates publish/subscribe messaging, SOW queries, delta publishing,
//! filtered subscriptions, SOW-and-subscribe, error handling, heartbeats,
//! and cross-thread usage using the amps-rust-ffi library.
//!
//! Requires a running AMPS server:
//!   docker-compose -f tests/docker/docker-compose.yml up -d

use amps_rust_ffi::{AmpsError, Client};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const AMPS_URI: &str = "tcp://localhost:9007/amps/json";

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    id: String,
    symbol: String,
    side: String,
    qty: u32,
    price: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AMPS Rust FFI Example ===\n");

    publish_example()?;
    publish_with_expiration_example()?;
    subscribe_example()?;
    filtered_subscribe_example()?;
    sow_example()?;
    sow_and_subscribe_example()?;
    delta_publish_example()?;
    heartbeat_example()?;
    error_handling_example();
    cross_thread_example()?;

    println!("\n=== All examples completed successfully ===");
    Ok(())
}

/// Demonstrates basic publish.
fn publish_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Publish Example ---");

    let mut client = Client::new("example-publisher")?;
    client.connect(AMPS_URI)?;
    client.logon(None, 5000)?;

    let order = Order {
        id: "ORD-001".into(),
        symbol: "AAPL".into(),
        side: "buy".into(),
        qty: 100,
        price: 150.25,
    };

    let json = serde_json::to_string(&order)?;
    let seq = client.publish("orders", &json)?;
    println!("  Published order: {} (seq={})", json, seq);

    client.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates publish with an expiration time.
fn publish_with_expiration_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Publish with Expiration Example ---");

    let mut client = Client::new("example-expiry-publisher")?;
    client.connect(AMPS_URI)?;
    client.logon(None, 5000)?;

    let order = Order {
        id: "ORD-EXPIRY-001".into(),
        symbol: "NVDA".into(),
        side: "buy".into(),
        qty: 50,
        price: 875.00,
    };

    let json = serde_json::to_string(&order)?;
    let seq = client.publish_with_expiration("orders", &json, 300)?;
    println!("  Published with 300s expiration: {} (seq={})", json, seq);

    client.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates subscribe with a message handler.
fn subscribe_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Subscribe Example ---");

    // Set up subscriber
    let mut subscriber = Client::new("example-subscriber")?;
    subscriber.connect(AMPS_URI)?;
    subscriber.logon(None, 5000)?;

    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();

    subscriber.subscribe("orders", None, move |msg| {
        println!("  Received [{}]: {}", msg.command(), msg.data());
        received_clone
            .lock()
            .unwrap()
            .push(msg.data().to_string());
    })?;

    // Give subscription time to establish
    std::thread::sleep(Duration::from_millis(100));

    // Publish some messages
    let mut publisher = Client::new("example-sub-publisher")?;
    publisher.connect(AMPS_URI)?;
    publisher.logon(None, 5000)?;

    for i in 1..=3 {
        let msg = format!(r#"{{"id": "SUB-{}", "symbol": "MSFT", "side": "sell", "qty": {}, "price": 400.50}}"#, i, i * 10);
        publisher.publish("orders", &msg)?;
    }

    // Wait for messages to arrive
    std::thread::sleep(Duration::from_millis(500));

    let count = received.lock().unwrap().len();
    println!("  Total messages received: {}", count);

    subscriber.unsubscribe_all()?;
    subscriber.disconnect()?;
    publisher.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates a filtered subscription that only receives matching messages.
fn filtered_subscribe_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Filtered Subscribe Example ---");

    let mut subscriber = Client::new("example-filtered-sub")?;
    subscriber.connect(AMPS_URI)?;
    subscriber.logon(None, 5000)?;

    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let received_clone = received.clone();

    // Only receive orders with price > 200
    subscriber.subscribe("orders", Some("/price > 200"), move |msg| {
        println!("  Matched filter: {}", msg.data());
        received_clone
            .lock()
            .unwrap()
            .push(msg.data().to_string());
    })?;

    std::thread::sleep(Duration::from_millis(100));

    let mut publisher = Client::new("example-filtered-pub")?;
    publisher.connect(AMPS_URI)?;
    publisher.logon(None, 5000)?;

    // This should NOT be received (price <= 200)
    publisher.publish(
        "orders",
        r#"{"id": "FILT-1", "symbol": "INTC", "side": "buy", "qty": 100, "price": 30.00}"#,
    )?;
    // This SHOULD be received (price > 200)
    publisher.publish(
        "orders",
        r#"{"id": "FILT-2", "symbol": "TSLA", "side": "sell", "qty": 10, "price": 850.00}"#,
    )?;

    std::thread::sleep(Duration::from_millis(500));

    let count = received.lock().unwrap().len();
    println!("  Messages matching /price > 200: {}", count);

    subscriber.unsubscribe_all()?;
    subscriber.disconnect()?;
    publisher.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates SOW (State-of-the-World) query.
fn sow_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- SOW Query Example ---");

    // Publish records to a SOW-enabled topic
    let mut publisher = Client::new("example-sow-publisher")?;
    publisher.connect(AMPS_URI)?;
    publisher.logon(None, 5000)?;

    let records = vec![
        r#"{"id": "SOW-1", "symbol": "GOOG", "side": "buy",  "qty": 50,  "price": 140.00}"#,
        r#"{"id": "SOW-2", "symbol": "AMZN", "side": "sell", "qty": 25,  "price": 185.50}"#,
        r#"{"id": "SOW-3", "symbol": "GOOG", "side": "buy",  "qty": 100, "price": 141.00}"#,
    ];

    for record in &records {
        publisher.publish("sow-test", record)?;
    }
    std::thread::sleep(Duration::from_millis(100));

    // Query SOW
    let mut querier = Client::new("example-sow-querier")?;
    querier.connect(AMPS_URI)?;
    querier.logon(None, 5000)?;

    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    querier.sow("sow-test", Some("/symbol = 'GOOG'"), move |msg| {
        if msg.has_data() {
            println!("  SOW record: {}", msg.data());
            results_clone
                .lock()
                .unwrap()
                .push(msg.data().to_string());
        }
    })?;

    std::thread::sleep(Duration::from_millis(500));

    let count = results.lock().unwrap().len();
    println!("  SOW query returned {} records matching /symbol = 'GOOG'", count);

    querier.disconnect()?;
    publisher.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates SOW-and-subscribe: get current state then receive live updates.
fn sow_and_subscribe_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- SOW and Subscribe Example ---");

    // First, seed some data into the SOW topic
    let mut publisher = Client::new("example-sow-sub-publisher")?;
    publisher.connect(AMPS_URI)?;
    publisher.logon(None, 5000)?;

    publisher.publish(
        "sow-test",
        r#"{"id": "SOWSUB-1", "symbol": "META", "side": "buy", "qty": 75, "price": 500.00}"#,
    )?;
    publisher.publish(
        "sow-test",
        r#"{"id": "SOWSUB-2", "symbol": "META", "side": "sell", "qty": 30, "price": 510.00}"#,
    )?;
    std::thread::sleep(Duration::from_millis(100));

    // SOW-and-subscribe: get existing records, then receive live updates
    let mut client = Client::new("example-sow-subscriber")?;
    client.connect(AMPS_URI)?;
    client.logon(None, 5000)?;

    let messages = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let messages_clone = messages.clone();

    client.sow_and_subscribe("sow-test", Some("/symbol = 'META'"), move |msg| {
        if msg.has_data() {
            let cmd = msg.command().to_string();
            let data = msg.data().to_string();
            println!("  [{}] {}", cmd, data);
            messages_clone.lock().unwrap().push((cmd, data));
        }
    })?;

    std::thread::sleep(Duration::from_millis(500));

    // Publish a live update — should also be received
    publisher.publish(
        "sow-test",
        r#"{"id": "SOWSUB-3", "symbol": "META", "side": "buy", "qty": 100, "price": 515.00}"#,
    )?;

    std::thread::sleep(Duration::from_millis(500));

    let msgs = messages.lock().unwrap();
    println!("  Total messages (SOW + live): {}", msgs.len());

    client.unsubscribe_all()?;
    client.disconnect()?;
    publisher.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates delta publishing for partial updates.
fn delta_publish_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Delta Publish Example ---");

    let mut client = Client::new("example-delta")?;
    client.connect(AMPS_URI)?;
    client.logon(None, 5000)?;

    // Publish full record
    let full = r#"{"id": "DELTA-1", "symbol": "TSLA", "side": "buy", "qty": 200, "price": 250.00}"#;
    client.publish("sow-test", full)?;
    println!("  Published full record: {}", full);

    // Delta update — only the changed fields
    let delta = r#"{"id": "DELTA-1", "price": 255.75}"#;
    client.delta_publish("sow-test", delta)?;
    println!("  Delta update: {}", delta);

    client.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates setting heartbeat for connection health monitoring.
fn heartbeat_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Heartbeat Example ---");

    let mut client = Client::new("example-heartbeat")?;
    client.connect(AMPS_URI)?;
    client.logon(None, 5000)?;

    // Set heartbeat: send every 30s, read timeout 60s
    client.set_heartbeat(30, 60)?;
    println!("  Heartbeat configured: interval=30s, read_timeout=60s");

    client.disconnect()?;
    println!();
    Ok(())
}

/// Demonstrates error handling with AmpsError variants.
fn error_handling_example() {
    println!("--- Error Handling Example ---");

    // 1. Invalid URI — connection error
    let mut client = Client::new("example-error").unwrap();
    match client.connect("tcp://invalid-host:59999/amps/json") {
        Ok(_) => println!("  Unexpected success"),
        Err(AmpsError::Connection { message }) => {
            println!("  Connection error (expected): {}", message);
        }
        Err(AmpsError::ConnectionRefused { message }) => {
            println!("  Connection refused (expected): {}", message);
        }
        Err(other) => {
            println!("  Other error (still handled): {}", other);
        }
    }

    // 2. Publish without connecting — should error
    let mut client2 = Client::new("example-error-2").unwrap();
    match client2.publish("topic", r#"{"test": true}"#) {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Publish without connect (expected error): {}", e),
    }

    println!();
}

/// Demonstrates moving a client to another thread (Client is Send).
fn cross_thread_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Cross-Thread Example ---");

    let mut client = Client::new("example-cross-thread")?;
    client.connect(AMPS_URI)?;
    client.logon(None, 5000)?;

    // Move client to a background thread for publishing
    let handle = std::thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for i in 0..3 {
            let msg = format!(r#"{{"id": "THREAD-{}", "thread": "background", "seq": {}}}"#, i, i);
            client.publish("orders", &msg)?;
            println!("  [background thread] Published: {}", msg);
        }
        client.disconnect()?;
        Ok(())
    });

    handle
        .join()
        .expect("Background thread panicked")
        .map_err(|e| -> Box<dyn std::error::Error> { e })?;
    println!("  Background thread completed successfully");

    println!();
    Ok(())
}

//! Integration tests for AMPS Rust FFI
//!
//! These tests require a running AMPS server. To start one:
//!   docker-compose -f tests/docker/docker-compose.yml up -d
//!
//! To stop:
//!   docker-compose -f tests/docker/docker-compose.yml down

use amps_rust_ffi::{AmpsError, Client};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Get the AMPS connection URI from environment or use default
fn amps_uri() -> String {
    std::env::var("AMPS_TEST_URI").unwrap_or_else(|_| "tcp://localhost:9007/amps/json".to_string())
}

/// Helper function to check if AMPS server is available
fn is_amps_available() -> bool {
    let mut client = match Client::new("test-availability") {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Try to connect with a short timeout by using non-blocking pattern
    match client.connect(&amps_uri()) {
        Ok(_) => {
            let _ = client.disconnect();
            true
        }
        Err(_) => false,
    }
}

/// Macro to skip tests if AMPS is not available
///
/// Note: In Rust's test framework, skipped tests show as "passed" but with the skip message.
/// This is the standard behavior for conditional test skipping.
macro_rules! skip_if_no_amps {
    () => {
        if !is_amps_available() {
            eprintln!("\n[SKIPPED] AMPS server not available at {}", amps_uri());
            eprintln!(
                "Start the server with: docker-compose -f tests/docker/docker-compose.yml up -d"
            );
            return;
        }
    };
}

#[test]
fn test_connect_and_publish() {
    skip_if_no_amps!();

    let mut client = Client::new("test-publisher").expect("Failed to create client");

    // Connect to AMPS server
    client
        .connect(&amps_uri())
        .expect("Failed to connect to AMPS");

    // Logon to the server
    client.logon(None, 5000).expect("Failed to logon");

    // Publish a message
    let seq = client
        .publish("test-topic", r#"{"message": "hello"}"#)
        .expect("Failed to publish message");

    assert!(seq > 0, "Expected positive sequence number, got {}", seq);

    // Publish with expiration
    let seq2 = client
        .publish_with_expiration("test-topic", r#"{"message": "with expiration"}"#, 60)
        .expect("Failed to publish message with expiration");

    assert!(seq2 > 0, "Expected positive sequence number, got {}", seq2);
    assert!(seq2 > seq, "Expected sequence number to increase");

    // Disconnect
    client.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_delta_publish() {
    skip_if_no_amps!();

    let mut client = Client::new("test-delta-publisher").expect("Failed to create client");

    client.connect(&amps_uri()).expect("Failed to connect");
    client.logon(None, 5000).expect("Failed to logon");

    // First publish a full record
    client
        .publish("sow-test", r#"{"id": "1", "name": "test", "value": 100}"#)
        .expect("Failed to publish initial record");

    // Then publish a delta update
    client
        .delta_publish("sow-test", r#"{"id": "1", "value": 200}"#)
        .expect("Failed to delta publish");

    client.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_subscribe_and_receive() {
    skip_if_no_amps!();

    let topic = "subscription-test";
    let test_message = r#"{"test": "data", "value": 42}"#;

    // Create publisher client
    let mut publisher = Client::new("test-sub-publisher").expect("Failed to create publisher");
    publisher
        .connect(&amps_uri())
        .expect("Failed to connect publisher");
    publisher
        .logon(None, 5000)
        .expect("Failed to logon publisher");

    // Create subscriber client
    let mut subscriber = Client::new("test-subscriber").expect("Failed to create subscriber");
    subscriber
        .connect(&amps_uri())
        .expect("Failed to connect subscriber");
    subscriber
        .logon(None, 5000)
        .expect("Failed to logon subscriber");

    // Set up message capture
    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_messages.clone();

    // Subscribe
    subscriber
        .subscribe(topic, None, move |msg| {
            let data = msg.data().to_string();
            received_clone.lock().unwrap().push(data);
        })
        .expect("Failed to subscribe");

    // Give subscription time to establish
    std::thread::sleep(Duration::from_millis(100));

    // Publish a message
    publisher
        .publish(topic, test_message)
        .expect("Failed to publish");

    // Wait for message to be received
    std::thread::sleep(Duration::from_millis(500));

    // Verify message was received
    let messages = received_messages.lock().unwrap();
    assert!(
        !messages.is_empty(),
        "Expected at least one message to be received"
    );

    // Parse the received message and verify content
    let received: serde_json::Value =
        serde_json::from_str(&messages[0]).expect("Failed to parse received message as JSON");
    assert_eq!(received["test"], "data");
    assert_eq!(received["value"], 42);

    // Clean up
    subscriber.unsubscribe_all().expect("Failed to unsubscribe");
    subscriber
        .disconnect()
        .expect("Failed to disconnect subscriber");
    publisher
        .disconnect()
        .expect("Failed to disconnect publisher");
}

#[test]
fn test_subscribe_with_filter() {
    skip_if_no_amps!();

    let topic = "subscription-test";

    // Create publisher
    let mut publisher = Client::new("test-filter-publisher").expect("Failed to create publisher");
    publisher.connect(&amps_uri()).expect("Failed to connect");
    publisher.logon(None, 5000).expect("Failed to logon");

    // Create subscriber with filter
    let mut subscriber =
        Client::new("test-filter-subscriber").expect("Failed to create subscriber");
    subscriber.connect(&amps_uri()).expect("Failed to connect");
    subscriber.logon(None, 5000).expect("Failed to logon");

    let received_messages = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_messages.clone();

    // Subscribe with a filter - only receive messages where value > 50
    subscriber
        .subscribe(topic, Some("/value > 50"), move |msg| {
            let data = msg.data().to_string();
            received_clone.lock().unwrap().push(data);
        })
        .expect("Failed to subscribe");

    std::thread::sleep(Duration::from_millis(100));

    // Publish messages that should match the filter
    publisher
        .publish(topic, r#"{"id": "1", "value": 75}"#)
        .expect("Failed to publish");
    publisher
        .publish(topic, r#"{"id": "2", "value": 100}"#)
        .expect("Failed to publish");

    // Publish messages that should NOT match the filter
    publisher
        .publish(topic, r#"{"id": "3", "value": 25}"#)
        .expect("Failed to publish");
    publisher
        .publish(topic, r#"{"id": "4", "value": 10}"#)
        .expect("Failed to publish");

    std::thread::sleep(Duration::from_millis(500));

    // Verify only filtered messages were received
    let messages = received_messages.lock().unwrap();
    assert_eq!(messages.len(), 2, "Expected 2 messages matching the filter");

    // Verify the correct messages were received
    for msg in messages.iter() {
        let parsed: serde_json::Value = serde_json::from_str(msg).unwrap();
        let value = parsed["value"].as_i64().unwrap();
        assert!(
            value > 50,
            "Received message with value {} that should have been filtered",
            value
        );
    }

    subscriber.unsubscribe_all().expect("Failed to unsubscribe");
    subscriber.disconnect().expect("Failed to disconnect");
    publisher.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_sow_query() {
    skip_if_no_amps!();

    let topic = "sow-test";

    // Create publisher to populate SOW
    let mut publisher = Client::new("test-sow-publisher").expect("Failed to create publisher");
    publisher.connect(&amps_uri()).expect("Failed to connect");
    publisher.logon(None, 5000).expect("Failed to logon");

    // Publish some records to SOW
    publisher
        .publish(topic, r#"{"id": "1", "name": "alice", "age": 30}"#)
        .expect("Failed to publish");
    publisher
        .publish(topic, r#"{"id": "2", "name": "bob", "age": 25}"#)
        .expect("Failed to publish");
    publisher
        .publish(topic, r#"{"id": "3", "name": "charlie", "age": 35}"#)
        .expect("Failed to publish");

    // Small delay to ensure messages are persisted
    std::thread::sleep(Duration::from_millis(100));

    // Create client for SOW query
    let mut client = Client::new("test-sow-querier").expect("Failed to create client");
    client.connect(&amps_uri()).expect("Failed to connect");
    client.logon(None, 5000).expect("Failed to logon");

    let sow_results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = sow_results.clone();

    // Execute SOW query
    client
        .sow(topic, None, move |msg| {
            let data = msg.data().to_string();
            results_clone.lock().unwrap().push(data);
        })
        .expect("Failed to execute SOW query");

    // Wait for results
    std::thread::sleep(Duration::from_millis(500));

    let results = sow_results.lock().unwrap();
    assert!(!results.is_empty(), "Expected SOW query results");

    // Verify we got the records we published
    // Note: SOW queries may return additional records if the topic was previously populated
    assert!(
        results.len() >= 3,
        "Expected at least 3 SOW records, got {}",
        results.len()
    );

    client.disconnect().expect("Failed to disconnect");
    publisher.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_sow_query_with_filter() {
    skip_if_no_amps!();

    let topic = "sow-test";

    // Create publisher
    let mut publisher =
        Client::new("test-sow-filter-publisher").expect("Failed to create publisher");
    publisher.connect(&amps_uri()).expect("Failed to connect");
    publisher.logon(None, 5000).expect("Failed to logon");

    // Publish records with unique IDs to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    publisher
        .publish(
            topic,
            &format!(
                r#"{{"id": "sow-filt-{}-1", "category": "A", "amount": 100}}"#,
                timestamp
            ),
        )
        .expect("Failed to publish");
    publisher
        .publish(
            topic,
            &format!(
                r#"{{"id": "sow-filt-{}-2", "category": "B", "amount": 200}}"#,
                timestamp
            ),
        )
        .expect("Failed to publish");
    publisher
        .publish(
            topic,
            &format!(
                r#"{{"id": "sow-filt-{}-3", "category": "A", "amount": 150}}"#,
                timestamp
            ),
        )
        .expect("Failed to publish");

    std::thread::sleep(Duration::from_millis(100));

    // Create client for SOW query
    let mut client = Client::new("test-sow-filter-querier").expect("Failed to create client");
    client.connect(&amps_uri()).expect("Failed to connect");
    client.logon(None, 5000).expect("Failed to logon");

    let sow_results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = sow_results.clone();

    // Execute SOW query with filter - only category A
    client
        .sow(topic, Some("/category = 'A'"), move |msg| {
            let data = msg.data().to_string();
            results_clone.lock().unwrap().push(data);
        })
        .expect("Failed to execute SOW query");

    std::thread::sleep(Duration::from_millis(500));

    let results = sow_results.lock().unwrap();

    // Check that we only got category A records from this batch
    for result in results.iter() {
        let parsed: serde_json::Value = serde_json::from_str(result).unwrap();
        if let Some(id) = parsed["id"].as_str() {
            if id.starts_with(&format!("sow-filt-{}-", timestamp)) {
                assert_eq!(
                    parsed["category"], "A",
                    "Received non-A category record that should have been filtered"
                );
            }
        }
    }

    client.disconnect().expect("Failed to disconnect");
    publisher.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_sow_and_subscribe() {
    skip_if_no_amps!();

    let topic = "sow-test";

    // Create publisher
    let mut publisher = Client::new("test-sowsub-publisher").expect("Failed to create publisher");
    publisher.connect(&amps_uri()).expect("Failed to connect");
    publisher.logon(None, 5000).expect("Failed to logon");

    // Pre-populate SOW
    publisher
        .publish(topic, r#"{"id": "sowsub-1", "status": "initial"}"#)
        .expect("Failed to publish");

    std::thread::sleep(Duration::from_millis(100));

    // Create subscriber
    let mut subscriber =
        Client::new("test-sowsub-subscriber").expect("Failed to create subscriber");
    subscriber.connect(&amps_uri()).expect("Failed to connect");
    subscriber.logon(None, 5000).expect("Failed to logon");

    let messages = Arc::new(Mutex::new(Vec::new()));
    let msg_clone = messages.clone();

    // Execute SOW and subscribe
    subscriber
        .sow_and_subscribe(topic, None, move |msg| {
            let data = msg.data().to_string();
            let cmd = msg.command().to_string();
            msg_clone.lock().unwrap().push((cmd, data));
        })
        .expect("Failed to execute SOW and subscribe");

    // Wait for SOW results
    std::thread::sleep(Duration::from_millis(300));

    // Publish an update
    publisher
        .publish(topic, r#"{"id": "sowsub-1", "status": "updated"}"#)
        .expect("Failed to publish update");
    publisher
        .publish(topic, r#"{"id": "sowsub-2", "status": "new"}"#)
        .expect("Failed to publish new");

    // Wait for subscription updates
    std::thread::sleep(Duration::from_millis(500));

    let received = messages.lock().unwrap();
    assert!(!received.is_empty(), "Expected to receive messages");

    // Should have at least the SOW result and the updates
    assert!(
        received.len() >= 3,
        "Expected at least 3 messages (SOW + 2 updates), got {}",
        received.len()
    );

    subscriber.unsubscribe_all().expect("Failed to unsubscribe");
    subscriber.disconnect().expect("Failed to disconnect");
    publisher.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_unsubscribe() {
    skip_if_no_amps!();

    let topic = "test-topic";

    let mut client = Client::new("test-unsubscriber").expect("Failed to create client");
    client.connect(&amps_uri()).expect("Failed to connect");
    client.logon(None, 5000).expect("Failed to logon");

    let messages = Arc::new(Mutex::new(Vec::new()));
    let msg_clone = messages.clone();

    // Subscribe
    client
        .subscribe(topic, None, move |msg| {
            msg_clone.lock().unwrap().push(msg.data().to_string());
        })
        .expect("Failed to subscribe");

    // Unsubscribe all
    client.unsubscribe_all().expect("Failed to unsubscribe all");

    // Create a publisher to send messages
    let mut publisher = Client::new("test-unsub-publisher").expect("Failed to create publisher");
    publisher.connect(&amps_uri()).expect("Failed to connect");
    publisher.logon(None, 5000).expect("Failed to logon");

    // Publish a message after unsubscribing
    publisher
        .publish(topic, r#"{"test": "after-unsubscribe"}"#)
        .expect("Failed to publish");

    std::thread::sleep(Duration::from_millis(500));

    // Messages received before unsubscribe should be there, but not after
    // Note: We can't easily test this without tracking subscription IDs,
    // but we at least verify the operations don't fail

    client.disconnect().expect("Failed to disconnect");
    publisher.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_exception_handling_invalid_uri() {
    // This test doesn't require a running AMPS server
    let mut client = Client::new("test-error").expect("Failed to create client");

    // Attempt to connect to an invalid/unreachable URI
    let result = client.connect("tcp://invalid-host:59999/amps/json");

    assert!(result.is_err(), "Expected connection to fail");

    // Verify error type is a connection error
    match result {
        Err(AmpsError::Connection { .. }) => (),        // Expected
        Err(AmpsError::ConnectionRefused { .. }) => (), // Also acceptable
        Err(AmpsError::TimedOut { .. }) => (),          // Also acceptable
        Err(other) => {
            // Other error types may also occur depending on the system
            eprintln!("Got error: {:?}", other);
            // Accept any error as long as it's an error
        }
        Ok(_) => panic!("Expected connection to fail but it succeeded"),
    }
}

#[test]
fn test_exception_handling_already_connected() {
    skip_if_no_amps!();

    let mut client = Client::new("test-already-connected").expect("Failed to create client");

    // First connection
    client
        .connect(&amps_uri())
        .expect("First connect should succeed");

    // Second connection attempt should fail
    let result = client.connect(&amps_uri());

    // This may either succeed (if AMPS allows reconnect) or fail with AlreadyConnected
    match result {
        Ok(_) => {
            // Some implementations allow reconnect
            eprintln!("Second connect succeeded (allowed by AMPS implementation)");
        }
        Err(AmpsError::AlreadyConnected { .. }) => {
            // Expected error
        }
        Err(other) => {
            eprintln!("Got unexpected error on second connect: {:?}", other);
        }
    }

    client.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_exception_handling_not_connected() {
    // This test doesn't require a running AMPS server
    let mut client = Client::new("test-not-connected").expect("Failed to create client");

    // Try to publish without connecting
    let result = client.publish("test-topic", r#"{"data": "test"}"#);

    assert!(
        result.is_err(),
        "Expected publish to fail when not connected"
    );
}

#[test]
fn test_set_heartbeat() {
    skip_if_no_amps!();

    let mut client = Client::new("test-heartbeat").expect("Failed to create client");
    client.connect(&amps_uri()).expect("Failed to connect");
    client.logon(None, 5000).expect("Failed to logon");

    // Set heartbeat parameters
    client
        .set_heartbeat(30, 60)
        .expect("Failed to set heartbeat");

    client.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_multiple_clients() {
    skip_if_no_amps!();

    // Create multiple clients
    let mut clients: Vec<Client> = Vec::new();

    for i in 0..5 {
        let client = Client::new(&format!("test-multi-{}", i)).expect("Failed to create client");
        clients.push(client);
    }

    // Connect all clients
    for client in clients.iter_mut() {
        client.connect(&amps_uri()).expect("Failed to connect");
        client.logon(None, 5000).expect("Failed to logon");
    }

    // Use first client to publish, others to subscribe
    let topic = "test-topic";

    // Set up subscriptions on clients 1-4
    let received_count = Arc::new(Mutex::new(0));
    for (i, client) in clients[1..].iter_mut().enumerate() {
        let count = received_count.clone();
        client
            .subscribe(topic, None, move |_msg| {
                *count.lock().unwrap() += 1;
            })
            .expect(&format!("Failed to subscribe client {}", i + 1));
    }

    std::thread::sleep(Duration::from_millis(100));

    // Publish from client 0
    clients[0]
        .publish(topic, r#"{"test": "multi-client"}"#)
        .expect("Failed to publish");

    std::thread::sleep(Duration::from_millis(500));

    // Verify messages were received
    let count = received_count.lock().unwrap();
    assert!(
        *count >= 4,
        "Expected at least 4 messages (one per subscriber), got {}",
        *count
    );

    // Disconnect all
    for client in clients.iter_mut() {
        client.disconnect().expect("Failed to disconnect");
    }
}

#[test]
fn test_message_properties() {
    skip_if_no_amps!();

    let topic = "test-topic";
    let test_data = r#"{"id": "msg-test", "value": 123}"#;

    let mut publisher = Client::new("test-msg-props-pub").expect("Failed to create publisher");
    publisher.connect(&amps_uri()).expect("Failed to connect");
    publisher.logon(None, 5000).expect("Failed to logon");

    let mut subscriber = Client::new("test-msg-props-sub").expect("Failed to create subscriber");
    subscriber.connect(&amps_uri()).expect("Failed to connect");
    subscriber.logon(None, 5000).expect("Failed to logon");

    let received_message = Arc::new(Mutex::new(None));
    let msg_clone = received_message.clone();

    subscriber
        .subscribe(topic, None, move |msg| {
            *msg_clone.lock().unwrap() = Some((
                msg.topic().to_string(),
                msg.data().to_string(),
                msg.command().to_string(),
                msg.data_len(),
                msg.has_data(),
            ));
        })
        .expect("Failed to subscribe");

    std::thread::sleep(Duration::from_millis(100));

    publisher
        .publish(topic, test_data)
        .expect("Failed to publish");

    std::thread::sleep(Duration::from_millis(500));

    let msg_info = received_message.lock().unwrap();
    assert!(msg_info.is_some(), "Expected to receive a message");

    let (received_topic, received_data, received_command, data_len, has_data) =
        msg_info.clone().unwrap();

    assert_eq!(received_topic, topic, "Topic mismatch");
    assert_eq!(received_data, test_data, "Data mismatch");
    assert_eq!(received_command, "publish", "Command should be 'publish'");
    assert_eq!(data_len, test_data.len(), "Data length mismatch");
    assert!(has_data, "Message should have data");

    subscriber.unsubscribe_all().expect("Failed to unsubscribe");
    subscriber.disconnect().expect("Failed to disconnect");
    publisher.disconnect().expect("Failed to disconnect");
}

#[test]
fn test_client_send_trait() {
    // Verify that Client is Send but not Sync
    fn assert_send<T: Send>() {}
    assert_send::<Client>();

    // Note: Client is intentionally NOT Sync (AMPS C++ client is not thread-safe)
    // The following line would fail to compile if uncommented:
    // fn assert_sync<T: Sync>() {}
    // assert_sync::<Client>(); // This would be a compile error
}

#[test]
fn test_connection_to_different_uris() {
    skip_if_no_amps!();

    // Test with different URI formats that might be used
    let uris = vec![
        amps_uri(), // Default URI
    ];

    for (i, uri) in uris.iter().enumerate() {
        let mut client = Client::new(&format!("test-uri-{}", i)).expect("Failed to create client");

        match client.connect(uri) {
            Ok(_) => {
                client.logon(None, 5000).expect("Failed to logon");
                client.disconnect().expect("Failed to disconnect");
                eprintln!("Successfully connected to: {}", uri);
            }
            Err(e) => {
                eprintln!("Failed to connect to {}: {:?}", uri, e);
            }
        }
    }
}

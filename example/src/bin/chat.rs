//! AMPS Chat CLI
//!
//! A simple two-user chat application over an AMPS topic.
//!
//! Usage:
//!   cargo run --bin chat -- <name> [topic] [uri]
//!
//! Examples:
//!   # Terminal 1
//!   cargo run --bin chat -- alice
//!
//!   # Terminal 2
//!   cargo run --bin chat -- bob
//!
//! Messages are published as JSON to a shared topic. Each client subscribes
//! with a server-side filter to exclude its own messages.
//!
//! Requires a running AMPS server:
//!   docker-compose -f tests/docker/docker-compose.yml up -d

use amps_rust_ffi::Client;
use std::io::{self, BufRead, Write};

const DEFAULT_URI: &str = "tcp://localhost:9007/amps/json";
const DEFAULT_TOPIC: &str = "chat";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: chat <name> [topic] [uri]");
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  name    Your display name");
        eprintln!("  topic   Chat topic (default: \"{DEFAULT_URI}\")");
        eprintln!("  uri     AMPS server URI (default: \"{DEFAULT_TOPIC}\")");
        std::process::exit(1);
    }

    let name = &args[1];
    let topic = args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_TOPIC);
    let uri = args.get(3).map(|s| s.as_str()).unwrap_or(DEFAULT_URI);

    // Connect and logon
    let mut client = Client::new(&format!("chat-{name}"))?;
    client.connect(uri)?;
    client.logon(None, 5000)?;

    // Subscribe, filtering out our own messages server-side
    let filter = format!("/sender != '{name}'");
    client.subscribe(topic, Some(&filter), move |msg| {
        if !msg.has_data() {
            return;
        }
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(msg.data()) {
            let sender = parsed["sender"].as_str().unwrap_or("???");
            let text = parsed["text"].as_str().unwrap_or("");
            // Clear current prompt line, print the message, re-show prompt
            print!("\r\x1b[K{sender}: {text}\n> ");
            let _ = io::stdout().flush();
        }
    })?;

    println!("Connected to {uri} on topic \"{topic}\" as {name}");
    println!("Type a message and press Enter. Ctrl-C to quit.\n");

    let stdin = io::stdin();

    print!("> ");
    io::stdout().flush()?;

    for line in stdin.lock().lines() {
        let line = line?;
        let text = line.trim();
        if text.is_empty() {
            print!("> ");
            io::stdout().flush()?;
            continue;
        }

        let msg = serde_json::json!({
            "sender": name,
            "text": text,
        });

        client.publish(topic, &msg.to_string())?;

        print!("> ");
        io::stdout().flush()?;
    }

    // stdin EOF (Ctrl-D) or pipe closed
    println!("\nDisconnecting...");
    client.unsubscribe_all()?;
    client.disconnect()?;

    Ok(())
}

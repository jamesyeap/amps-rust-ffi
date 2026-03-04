//! AMPS Rust FFI - Safe Rust bindings for the AMPS C++ client library.
//!
//! This crate provides Rust bindings for the AMPS (Advanced Message Processing System)
//! C++ client library, enabling publish/subscribe messaging, SOW queries, and more
//! from Rust applications.
//!
//! # Architecture
//!
//! The crate is organized into several layers:
//!
//! - **FFI Layer** ([`ffi`]): Low-level auto-generated bindings to the C wrapper
//! - **Error Types** ([`error`]): Error handling with [`AmpsError`] and [`AmpsResult`]
//! - **Safe API**: High-level, idiomatic Rust types:
//!   - [`Client`] - Main client for connecting to AMPS servers
//!   - [`Message`] - Messages received from subscriptions
//!   - [`subscription`] - Subscription handling traits and types
//!
//! # Quick Start
//!
//! ```no_run
//! use amps_rust_ffi::Client;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client
//!     let mut client = Client::new("my-app")?;
//!     
//!     // Connect to the AMPS server
//!     client.connect("tcp://localhost:9007/amps/json")?;
//!     client.logon(None, 5000)?;
//!     
//!     // Subscribe to a topic
//!     client.subscribe("orders", None, |msg| {
//!         println!("Received: {}", msg.data());
//!     })?;
//!     
//!     // Publish a message
//!     client.publish("orders", r#"{"symbol": "AAPL", "qty": 100}"#)?;
//!     
//!     // Disconnect when done
//!     client.disconnect()?;
//!     Ok(())
//! }
//! ```
//!
//! # Thread Safety
//!
//! The AMPS C++ client is **not** thread-safe. The [`Client`] type implements [`Send`]
//! but not [`Sync`]. If you need to share a client between threads, wrap it in a
//! [`Mutex`](std::sync::Mutex) or use a dedicated thread for AMPS operations.
//!
//! [`Message`] types are both [`Send`] and [`Sync`] as they are immutable views
//! into AMPS-owned memory.
//!
//! # Error Handling
//!
//! All operations that can fail return [`AmpsResult<T>`](AmpsResult), which is
//! a type alias for `Result<T, AmpsError>`. The [`AmpsError`] enum covers all
//! possible error conditions from the AMPS client library.

pub mod client;
pub mod error;
pub mod ffi;
pub mod message;
pub mod subscription;

// Re-export main types for convenience
pub use client::Client;
pub use error::{AmpsError, AmpsResult};
pub use message::Message;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_setup_compiles() {
        // Verify that core dependencies are available
        let _: log::Level = log::Level::Info;
        let _: libc::c_int = 0;
    }

    #[test]
    fn header_parseable_by_bindgen() {
        // Verify that bindgen can parse the C wrapper header
        let bindings = bindgen::Builder::default()
            .header("c-wrapper/include/amps_ffi.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindgen should parse amps_ffi.h successfully");

        let tokens = bindings.to_string();

        // Verify key types are generated
        assert!(
            tokens.contains("amps_ffi_error_t"),
            "missing amps_ffi_error_t enum"
        );
        assert!(
            tokens.contains("amps_ffi_error_info_t"),
            "missing amps_ffi_error_info_t struct"
        );
        assert!(
            tokens.contains("amps_ffi_client_create"),
            "missing amps_ffi_client_create fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_destroy"),
            "missing amps_ffi_client_destroy fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_connect"),
            "missing amps_ffi_client_connect fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_disconnect"),
            "missing amps_ffi_client_disconnect fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_logon"),
            "missing amps_ffi_client_logon fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_publish"),
            "missing amps_ffi_client_publish fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_delta_publish"),
            "missing amps_ffi_client_delta_publish fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_subscribe"),
            "missing amps_ffi_client_subscribe fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_unsubscribe"),
            "missing amps_ffi_client_unsubscribe fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_sow"),
            "missing amps_ffi_client_sow fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_sow_and_subscribe"),
            "missing amps_ffi_client_sow_and_subscribe fn"
        );
        assert!(
            tokens.contains("amps_ffi_message_get_data"),
            "missing amps_ffi_message_get_data fn"
        );
        assert!(
            tokens.contains("amps_ffi_message_get_topic"),
            "missing amps_ffi_message_get_topic fn"
        );
        assert!(
            tokens.contains("amps_ffi_error_string"),
            "missing amps_ffi_error_string fn"
        );
        assert!(
            tokens.contains("amps_ffi_version"),
            "missing amps_ffi_version fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_set_heartbeat"),
            "missing amps_ffi_client_set_heartbeat fn"
        );
        assert!(
            tokens.contains("amps_ffi_client_set_disconnect_handler"),
            "missing amps_ffi_client_set_disconnect_handler fn"
        );
        assert!(
            tokens.contains("AMPS_FFI_OK"),
            "missing AMPS_FFI_OK constant"
        );
        assert!(
            tokens.contains("AMPS_FFI_ERROR_CONNECTION"),
            "missing AMPS_FFI_ERROR_CONNECTION constant"
        );
    }

    #[test]
    fn test_re_exports() {
        // Test that main types are re-exported from the crate root
        let _: fn(&str) -> error::AmpsResult<client::Client> = Client::new;
        let _: error::AmpsResult<i32> = Err(error::AmpsError::NullPointer);
    }
}

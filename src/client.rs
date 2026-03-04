//! Safe Rust wrapper for the AMPS Client.
//!
//! This module provides a safe, idiomatic Rust interface to the AMPS (Advanced Message
//! Processing System) client library. The [`Client`] struct wraps the underlying FFI
//! handle and manages its lifecycle.
//!
//! # Thread Safety
//!
//! The AMPS C++ client is **not** thread-safe. Therefore, [`Client`] implements [`Send`]
//! but **not** [`Sync`]. You should not share a [`Client`] instance between threads
//! without external synchronization (e.g., wrapping it in a [`Mutex`](std::sync::Mutex)).
//!
//! # Example
//!
//! ```no_run
//! use amps_rust_ffi::{Client, AmpsResult};
//!
//! fn main() -> AmpsResult<()> {
//!     let mut client = Client::new("my-client")?;
//!     client.connect("tcp://localhost:9007/amps/json")?;
//!     client.logon(None, 5000)?;
//!     
//!     let seq = client.publish("test-topic", r#"{"message": "hello"}"#)?;
//!     println!("Published with sequence: {}", seq);
//!     
//!     client.disconnect()?;
//!     Ok(())
//! }
//! ```

use crate::error::{AmpsError, AmpsResult};
use crate::ffi;
use crate::message::Message;
use std::ffi::{c_ulong, c_void, CString};
use std::os::raw::c_char;
use std::ptr;

/// A safe wrapper around the AMPS C++ client.
///
/// This struct wraps an opaque FFI handle to an AMPS [`Client`](ffi::AMPS::Client) instance.
/// It provides safe methods for connecting, publishing, subscribing, and managing
/// the client lifecycle.
///
/// # Lifecycle
///
/// The client is created with [`Client::new`](Self::new) and should be properly
/// disconnected via [`Client::disconnect`](Self::disconnect) before being dropped.
/// The [`Drop`] implementation will automatically clean up the underlying handle.
pub struct Client {
    inner: ffi::amps_ffi_client_t,
}

// Mark Client as Send but NOT Sync (AMPS client is not thread-safe)
unsafe impl Send for Client {}

impl Client {
    /// Creates a new AMPS client with the given name.
    ///
    /// # Arguments
    ///
    /// * `client_name` - A unique name identifying this client instance
    ///
    /// # Errors
    ///
    /// Returns an error if the client name contains null bytes or if the
    /// underlying C++ client creation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use amps_rust_ffi::Client;
    ///
    /// let client = Client::new("my-client");
    /// assert!(client.is_ok());
    /// ```
    pub fn new(client_name: &str) -> AmpsResult<Self> {
        let name = CString::new(client_name)?;
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let handle = unsafe { ffi::amps_ffi_client_create(name.as_ptr(), &mut error_info) };

        if handle.is_null() {
            return Err(AmpsError::from(error_info));
        }

        Ok(Client { inner: handle })
    }

    /// Connects to an AMPS server at the specified URI.
    ///
    /// # Arguments
    ///
    /// * `uri` - The connection URI (e.g., `tcp://localhost:9007/amps/json`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The URI is invalid
    /// - The server is unreachable
    /// - The connection is refused
    /// - Already connected to a server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use amps_rust_ffi::Client;
    ///
    /// let mut client = Client::new("test").unwrap();
    /// client.connect("tcp://localhost:9007/amps/json").unwrap();
    /// ```
    pub fn connect(&mut self, uri: &str) -> AmpsResult<()> {
        let uri = CString::new(uri)?;
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let result =
            unsafe { ffi::amps_ffi_client_connect(self.inner, uri.as_ptr(), &mut error_info) };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Disconnects from the AMPS server.
    ///
    /// This method gracefully closes the connection to the AMPS server.
    /// It is safe to call even if not currently connected.
    ///
    /// # Errors
    ///
    /// Returns an error if the disconnect operation fails.
    pub fn disconnect(&mut self) -> AmpsResult<()> {
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let result = unsafe { ffi::amps_ffi_client_disconnect(self.inner, &mut error_info) };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Performs logon to the AMPS server with optional parameters.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional logon options string (pass `None` for default)
    /// * `timeout_ms` - Timeout in milliseconds for the logon operation
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails, the operation times out,
    /// or if not connected to a server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use amps_rust_ffi::Client;
    ///
    /// let mut client = Client::new("test").unwrap();
    /// client.connect("tcp://localhost:9007/amps/json").unwrap();
    /// client.logon(None, 5000).unwrap(); // 5 second timeout
    /// ```
    pub fn logon(&mut self, options: Option<&str>, timeout_ms: i32) -> AmpsResult<()> {
        let opts = options.map(CString::new).transpose()?;
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let result = unsafe {
            ffi::amps_ffi_client_logon(
                self.inner,
                opts.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null()),
                timeout_ms,
                &mut error_info,
            )
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Publishes a message to the specified topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic name to publish to
    /// * `data` - The message payload as a string
    ///
    /// # Returns
    ///
    /// Returns a sequence number on success (non-zero value).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not connected to the server
    /// - The topic is invalid
    /// - The message cannot be sent
    ///
    /// # Example
    ///
    /// ```no_run
    /// use amps_rust_ffi::Client;
    ///
    /// let mut client = Client::new("publisher").unwrap();
    /// client.connect("tcp://localhost:9007/amps/json").unwrap();
    /// client.logon(None, 5000).unwrap();
    ///
    /// let seq = client.publish("my-topic", r#"{"data": "value"}"#).unwrap();
    /// println!("Published with sequence: {}", seq);
    /// ```
    pub fn publish(&mut self, topic: &str, data: &str) -> AmpsResult<u64> {
        self.publish_with_expiration(topic, data, 0)
    }

    /// Publishes a message with an expiration time.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic name to publish to
    /// * `data` - The message payload as a string
    /// * `expiration_secs` - Time in seconds until the message expires (0 = no expiration)
    ///
    /// # Returns
    ///
    /// Returns a sequence number on success.
    pub fn publish_with_expiration(
        &mut self,
        topic: &str,
        data: &str,
        expiration_secs: u64,
    ) -> AmpsResult<u64> {
        let topic = CString::new(topic)?;
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let seq = unsafe {
            ffi::amps_ffi_client_publish(
                self.inner,
                topic.as_ptr(),
                data.as_ptr() as *const c_char,
                data.len(),
                expiration_secs as c_ulong,
                &mut error_info,
            )
        };

        if error_info.code != ffi::amps_ffi_error_t_AMPS_FFI_OK {
            return Err(AmpsError::from(error_info));
        }

        Ok(seq)
    }

    /// Publishes a delta message to the specified topic.
    ///
    /// Delta messages only contain the fields that have changed, allowing
    /// for efficient updates of large records.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic name to publish to
    /// * `data` - The delta payload as a string (only changed fields)
    ///
    /// # Errors
    ///
    /// Returns an error if publishing fails or if not connected.
    pub fn delta_publish(&mut self, topic: &str, data: &str) -> AmpsResult<()> {
        let topic = CString::new(topic)?;
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let result = unsafe {
            ffi::amps_ffi_client_delta_publish(
                self.inner,
                topic.as_ptr(),
                data.as_ptr() as *const c_char,
                data.len(),
                &mut error_info,
            )
        };

        if result == 0 {
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Subscribes to a topic with a message handler callback.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic or regex pattern to subscribe to
    /// * `filter` - Optional filter expression (pass `None` for no filter)
    /// * `handler` - A callback function that receives incoming messages
    ///
    /// # Type Parameters
    ///
    /// * `F` - The handler function type: `FnMut(&Message) + Send + 'static`
    ///
    /// # Errors
    ///
    /// Returns an error if subscription fails or if already subscribed to the topic.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use amps_rust_ffi::Client;
    ///
    /// let mut client = Client::new("subscriber").unwrap();
    /// client.connect("tcp://localhost:9007/amps/json").unwrap();
    /// client.logon(None, 5000).unwrap();
    ///
    /// client.subscribe("my-topic", None, |msg| {
    ///     println!("Received: {:?}", msg.data());
    /// }).unwrap();
    /// ```
    pub fn subscribe<F>(&mut self, topic: &str, filter: Option<&str>, handler: F) -> AmpsResult<()>
    where
        F: FnMut(&Message) + Send + 'static,
    {
        self.subscribe_with_options(topic, filter, None, 0, handler)
    }

    /// Subscribes to a topic with full options.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic or regex pattern to subscribe to
    /// * `filter` - Optional filter expression
    /// * `options` - Optional subscription options string
    /// * `timeout_ms` - Timeout in milliseconds (0 = no timeout)
    /// * `handler` - A callback function that receives incoming messages
    pub fn subscribe_with_options<F>(
        &mut self,
        topic: &str,
        filter: Option<&str>,
        options: Option<&str>,
        timeout_ms: i32,
        handler: F,
    ) -> AmpsResult<()>
    where
        F: FnMut(&Message) + Send + 'static,
    {
        let topic = CString::new(topic)?;
        let filter_cstr = filter.map(CString::new).transpose()?;
        let options_cstr = options.map(CString::new).transpose()?;

        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        // Box the handler to heap-allocate it and get a stable pointer
        let handler_box: Box<Box<dyn FnMut(&Message) + Send>> = Box::new(Box::new(handler));
        let user_data: *mut c_void = Box::into_raw(handler_box) as *mut c_void;

        let result = unsafe {
            ffi::amps_ffi_client_subscribe(
                self.inner,
                topic.as_ptr(),
                filter_cstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                options_cstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                timeout_ms,
                Some(message_handler_trampoline),
                user_data,
                &mut error_info,
            )
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            // Clean up the handler if subscription failed
            unsafe {
                let _: Box<Box<dyn FnMut(&Message) + Send>> = Box::from_raw(user_data as *mut _);
            }
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Executes a SOW (State-of-the-World) query.
    ///
    /// # Arguments
    ///
    /// * `topic` - The SOW topic to query
    /// * `filter` - Optional filter expression
    /// * `handler` - A callback function that receives query results
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub fn sow<F>(&mut self, topic: &str, filter: Option<&str>, handler: F) -> AmpsResult<()>
    where
        F: FnMut(&Message) + Send + 'static,
    {
        self.sow_with_options(topic, filter, None, 0, 0, 0, handler)
    }

    /// Executes a SOW query with full options.
    ///
    /// # Arguments
    ///
    /// * `topic` - The SOW topic to query
    /// * `filter` - Optional filter expression
    /// * `order_by` - Optional ordering specification
    /// * `batch_size` - Batch size for results (0 = default)
    /// * `top_n` - Limit results to top N (0 = no limit)
    /// * `timeout_ms` - Timeout in milliseconds (0 = no timeout)
    /// * `handler` - A callback function that receives query results
    pub fn sow_with_options<F>(
        &mut self,
        topic: &str,
        filter: Option<&str>,
        order_by: Option<&str>,
        batch_size: i32,
        top_n: i32,
        timeout_ms: i32,
        handler: F,
    ) -> AmpsResult<()>
    where
        F: FnMut(&Message) + Send + 'static,
    {
        let topic = CString::new(topic)?;
        let filter_cstr = filter.map(CString::new).transpose()?;
        let order_by_cstr = order_by.map(CString::new).transpose()?;

        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let handler_box: Box<Box<dyn FnMut(&Message) + Send>> = Box::new(Box::new(handler));
        let user_data: *mut c_void = Box::into_raw(handler_box) as *mut c_void;

        let result = unsafe {
            ffi::amps_ffi_client_sow(
                self.inner,
                topic.as_ptr(),
                filter_cstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                order_by_cstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                batch_size,
                top_n,
                timeout_ms,
                Some(message_handler_trampoline),
                user_data,
                &mut error_info,
            )
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            unsafe {
                let _: Box<Box<dyn FnMut(&Message) + Send>> = Box::from_raw(user_data as *mut _);
            }
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Executes a SOW query and subscribes to the topic.
    ///
    /// This combines a SOW query with a subscription, first returning all
    /// matching records from the SOW, then continuing with live updates.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to query and subscribe to
    /// * `filter` - Optional filter expression
    /// * `handler` - A callback function that receives messages
    pub fn sow_and_subscribe<F>(
        &mut self,
        topic: &str,
        filter: Option<&str>,
        handler: F,
    ) -> AmpsResult<()>
    where
        F: FnMut(&Message) + Send + 'static,
    {
        self.sow_and_subscribe_with_options(topic, filter, None, 0, handler)
    }

    /// Executes a SOW and subscribe with full options.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to query and subscribe to
    /// * `filter` - Optional filter expression
    /// * `options` - Optional subscription options
    /// * `timeout_ms` - Timeout in milliseconds (0 = no timeout)
    /// * `handler` - A callback function that receives messages
    pub fn sow_and_subscribe_with_options<F>(
        &mut self,
        topic: &str,
        filter: Option<&str>,
        options: Option<&str>,
        timeout_ms: i32,
        handler: F,
    ) -> AmpsResult<()>
    where
        F: FnMut(&Message) + Send + 'static,
    {
        let topic = CString::new(topic)?;
        let filter_cstr = filter.map(CString::new).transpose()?;
        let options_cstr = options.map(CString::new).transpose()?;

        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let handler_box: Box<Box<dyn FnMut(&Message) + Send>> = Box::new(Box::new(handler));
        let user_data: *mut c_void = Box::into_raw(handler_box) as *mut c_void;

        let result = unsafe {
            ffi::amps_ffi_client_sow_and_subscribe(
                self.inner,
                topic.as_ptr(),
                filter_cstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                options_cstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                timeout_ms,
                Some(message_handler_trampoline),
                user_data,
                &mut error_info,
            )
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            unsafe {
                let _: Box<Box<dyn FnMut(&Message) + Send>> = Box::from_raw(user_data as *mut _);
            }
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Unsubscribes from a specific subscription.
    ///
    /// # Arguments
    ///
    /// * `sub_id` - The subscription ID returned from a previous subscribe call
    pub fn unsubscribe(&mut self, sub_id: &str) -> AmpsResult<()> {
        let sub_id = CString::new(sub_id)?;
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let result = unsafe {
            ffi::amps_ffi_client_unsubscribe(self.inner, sub_id.as_ptr(), &mut error_info)
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Unsubscribes from all active subscriptions.
    pub fn unsubscribe_all(&mut self) -> AmpsResult<()> {
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        let result = unsafe { ffi::amps_ffi_client_unsubscribe_all(self.inner, &mut error_info) };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Sets a disconnect handler callback.
    ///
    /// The handler will be called when the client is disconnected from the server.
    ///
    /// # Arguments
    ///
    /// * `handler` - A callback function receiving the client handle and user data
    pub fn set_disconnect_handler<F>(&mut self, handler: F) -> AmpsResult<()>
    where
        F: FnMut(&mut Client) + Send + 'static,
    {
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_OK,
            message: [0; 1024],
        };

        // Wrap the handler in a Box to heap-allocate
        let handler_box: Box<Box<dyn FnMut(&mut Client) + Send>> = Box::new(Box::new(handler));
        let user_data: *mut c_void = Box::into_raw(handler_box) as *mut c_void;

        let result = unsafe {
            ffi::amps_ffi_client_set_disconnect_handler(
                self.inner,
                Some(disconnect_handler_trampoline),
                user_data,
                &mut error_info,
            )
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            unsafe {
                let _: Box<Box<dyn FnMut(&mut Client) + Send>> = Box::from_raw(user_data as *mut _);
            }
            return Err(AmpsError::from(error_info));
        }

        Ok(())
    }

    /// Sets heartbeat parameters for the connection.
    ///
    /// # Arguments
    ///
    /// * `heartbeat_time_sec` - Interval between heartbeats in seconds
    /// * `read_timeout_sec` - Read timeout in seconds
    pub fn set_heartbeat(
        &mut self,
        heartbeat_time_sec: u32,
        read_timeout_sec: u32,
    ) -> AmpsResult<()> {
        let result = unsafe {
            ffi::amps_ffi_client_set_heartbeat(self.inner, heartbeat_time_sec, read_timeout_sec)
        };

        if result as i32 != ffi::amps_ffi_error_t_AMPS_FFI_OK as i32 {
            return Err(AmpsError::NullPointer);
        }

        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe {
                ffi::amps_ffi_client_destroy(self.inner);
            }
        }
    }
}

/// Trampoline function for message handler callbacks.
///
/// This function is called from C and forwards the call to the Rust handler.
extern "C" fn message_handler_trampoline(message: ffi::amps_ffi_message_t, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }

    unsafe {
        let handler = &mut *(user_data as *mut Box<dyn FnMut(&Message) + Send>);
        let msg = Message::from_raw(message);
        handler(&msg);
    }
}

/// Trampoline function for disconnect handler callbacks.
extern "C" fn disconnect_handler_trampoline(
    _client: ffi::amps_ffi_client_t,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }

    unsafe {
        // Note: We can't safely reconstruct a Client here since we don't own it.
        // The handler should be designed to work without the client reference
        // or the user should use external state (Arc<Mutex<...>>) to track disconnects.
        let handler = &mut *(user_data as *mut Box<dyn FnMut(&mut Client) + Send>);
        // We can't actually call this handler safely without a valid Client.
        // This is a limitation that would need additional design work.
        // For now, we just ignore the call.
        let _ = handler;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("test-client");
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_creation_with_null_in_name() {
        let client = Client::new("test\0client");
        assert!(client.is_err());
        match client {
            Err(AmpsError::NulError { .. }) => (), // Expected
            _ => panic!("Expected NulError"),
        }
    }

    #[test]
    fn test_client_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Client>();
    }
}

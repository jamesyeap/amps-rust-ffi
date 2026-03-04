//! Message type for AMPS operations.
//!
//! This module provides the [`Message`] struct, which wraps an AMPS message
//! handle and provides safe accessor methods for message properties.

use crate::ffi;
use std::ffi::CStr;

/// A message received from AMPS.
///
/// This struct wraps an opaque FFI handle to an AMPS [`Message`](ffi::AMPS::Message)
/// instance. It provides safe methods for accessing message properties such as
/// the topic, data payload, command type, and various metadata.
///
/// # Lifetime
///
/// Messages are typically created from within callback handlers and should not
/// be stored beyond the callback's execution unless cloned. The underlying message
/// data is owned by the AMPS client library.
///
/// # Example
///
/// ```no_run
/// use amps_rust_ffi::Client;
///
/// let mut client = Client::new("test").unwrap();
/// client.subscribe("my-topic", None, |msg| {
///     println!("Topic: {}", msg.topic());
///     println!("Data: {}", msg.data());
///     println!("Command: {}", msg.command());
/// }).unwrap();
/// ```
pub struct Message {
    inner: ffi::amps_ffi_message_t,
}

impl Message {
    /// Creates a new Message wrapper from a raw FFI handle.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it relies on the caller to ensure that
    /// the handle is valid and points to a valid AMPS Message object.
    ///
    /// # Arguments
    ///
    /// * `handle` - The raw FFI message handle
    pub(crate) unsafe fn from_raw(handle: ffi::amps_ffi_message_t) -> Self {
        Message { inner: handle }
    }

    /// Returns the message data/payload.
    ///
    /// This is the actual content of the message, such as JSON, XML, or other
    /// message format data.
    ///
    /// # Returns
    ///
    /// A string slice containing the message data, or an empty string if unavailable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use amps_rust_ffi::Client;
    ///
    /// let mut client = Client::new("test").unwrap();
    /// client.subscribe("orders", None, |msg| {
    ///     let data = msg.data();
    ///     println!("Received: {}", data);
    /// }).unwrap();
    /// ```
    pub fn data(&self) -> &str {
        unsafe {
            let mut len: usize = 0;
            let ptr = ffi::amps_ffi_message_get_data(self.inner, &mut len);
            if ptr.is_null() || len == 0 {
                return "";
            }
            // The data is not null-terminated, so we use from_raw_parts
            let bytes = std::slice::from_raw_parts(ptr as *const u8, len);
            std::str::from_utf8(bytes).unwrap_or("")
        }
    }

    /// Returns the data as raw bytes.
    ///
    /// This is useful when working with binary message formats.
    pub fn data_bytes(&self) -> &[u8] {
        unsafe {
            let mut len: usize = 0;
            let ptr = ffi::amps_ffi_message_get_data(self.inner, &mut len);
            if ptr.is_null() || len == 0 {
                return &[];
            }
            std::slice::from_raw_parts(ptr as *const u8, len)
        }
    }

    /// Returns the topic this message was published to or received from.
    ///
    /// # Returns
    ///
    /// A string slice containing the topic name, or an empty string if unavailable.
    pub fn topic(&self) -> &str {
        unsafe {
            let ptr = ffi::amps_ffi_message_get_topic(self.inner);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Returns the AMPS command type for this message.
    ///
    /// Common commands include:
    /// - `publish` - A published message
    /// - `sow` - A SOW query result
    /// - `delta_publish` - A delta publish message
    /// - `group_begin` - Beginning of a message group
    /// - `group_end` - End of a message group
    ///
    /// # Returns
    ///
    /// A string slice containing the command, or an empty string if unavailable.
    pub fn command(&self) -> &str {
        unsafe {
            let ptr = ffi::amps_ffi_message_get_command(self.inner);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Returns the SOW key for this message.
    ///
    /// The SOW key uniquely identifies a record in a State-of-the-World topic.
    /// This is only meaningful for messages from SOW topics.
    ///
    /// # Returns
    ///
    /// A string slice containing the SOW key, or an empty string if unavailable.
    pub fn sow_key(&self) -> &str {
        unsafe {
            let ptr = ffi::amps_ffi_message_get_sow_key(self.inner);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Returns the bookmark for this message.
    ///
    /// Bookmarks are used for replaying messages from the transaction log
    /// and represent the position of the message in the log.
    ///
    /// # Returns
    ///
    /// A string slice containing the bookmark, or an empty string if unavailable.
    pub fn bookmark(&self) -> &str {
        unsafe {
            let ptr = ffi::amps_ffi_message_get_bookmark(self.inner);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Returns the subscription ID this message was received on.
    ///
    /// # Returns
    ///
    /// A string slice containing the subscription ID, or an empty string if unavailable.
    pub fn sub_id(&self) -> &str {
        unsafe {
            let ptr = ffi::amps_ffi_message_get_sub_id(self.inner);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Returns the command ID for this message.
    ///
    /// The command ID correlates responses with their original requests.
    ///
    /// # Returns
    ///
    /// A string slice containing the command ID, or an empty string if unavailable.
    pub fn command_id(&self) -> &str {
        unsafe {
            let ptr = ffi::amps_ffi_message_get_command_id(self.inner);
            if ptr.is_null() {
                return "";
            }
            CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Checks if this message is a subscription group begin marker.
    ///
    /// When batching is enabled, messages are grouped and delimited by
    /// `group_begin` and `group_end` markers.
    pub fn is_group_begin(&self) -> bool {
        self.command() == "group_begin"
    }

    /// Checks if this message is a subscription group end marker.
    pub fn is_group_end(&self) -> bool {
        self.command() == "group_end"
    }

    /// Checks if this message is a SOW delete notification.
    ///
    /// When a record is deleted from a SOW topic, subscribers receive
    /// a delete notification message.
    pub fn is_sow_delete(&self) -> bool {
        self.command() == "sow_delete"
    }

    /// Returns the data length in bytes.
    pub fn data_len(&self) -> usize {
        unsafe {
            let mut len: usize = 0;
            ffi::amps_ffi_message_get_data(self.inner, &mut len);
            len
        }
    }

    /// Checks if the message has any data payload.
    pub fn has_data(&self) -> bool {
        self.data_len() > 0
    }
}

// Messages can be sent between threads as they are just views into AMPS-owned memory
unsafe impl Send for Message {}
unsafe impl Sync for Message {}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("topic", &self.topic())
            .field("command", &self.command())
            .field("sow_key", &self.sow_key())
            .field("sub_id", &self.sub_id())
            .field("data_len", &self.data_len())
            .field("data", &self.data())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Message>();
        assert_sync::<Message>();
    }
}

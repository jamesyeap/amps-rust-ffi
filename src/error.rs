//! Error types for the AMPS Rust FFI library.
//!
//! This module defines the error types used throughout the safe Rust API,
//! providing idiomatic error handling via the [`AmpsError`] enum and
//! [`AmpsResult`] type alias.

use crate::ffi;
use std::ffi::NulError;

/// Error type for AMPS operations.
///
/// This enum represents all possible errors that can occur when interacting
/// with the AMPS server. Each variant corresponds to a specific error condition
/// and includes an error message with details.
#[derive(Debug, Clone, PartialEq)]
pub enum AmpsError {
    /// Connection error - general connection failure
    Connection { message: String },

    /// Already connected - attempting to connect when already connected
    AlreadyConnected { message: String },

    /// Authentication failed - invalid credentials or not authorized
    Authentication { message: String },

    /// Connection refused - server rejected the connection
    ConnectionRefused { message: String },

    /// Disconnected - connection was lost
    Disconnected { message: String },

    /// Client name in use - the client name is already registered
    NameInUse { message: String },

    /// Not entitled - user lacks permission for the operation
    NotEntitled { message: String },

    /// Bad filter expression - invalid filter syntax
    BadFilter { message: String },

    /// Bad regex topic - invalid regex pattern for topic
    BadRegexTopic { message: String },

    /// Bad SOW key - invalid State-of-the-World key
    BadSowKey { message: String },

    /// Invalid topic - the topic does not exist or is invalid
    InvalidTopic { message: String },

    /// Publish error - error while publishing a message
    Publish { message: String },

    /// Subscription already exists - attempting to create a duplicate subscription
    SubscriptionExists { message: String },

    /// Publish store gap - gap detected in publish store
    PublishStoreGap { message: String },

    /// Operation timed out - the operation did not complete in time
    TimedOut { message: String },

    /// Unknown error - an unexpected error occurred
    Unknown { message: String },

    /// Null pointer error - a null pointer was passed where not allowed
    NullPointer,

    /// Invalid argument - an argument was invalid
    InvalidArgument { message: String },

    /// CString conversion error - string contains null bytes
    NulError { message: String },
}

impl std::fmt::Display for AmpsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmpsError::Connection { message } => write!(f, "Connection error: {message}"),
            AmpsError::AlreadyConnected { message } => write!(f, "Already connected: {message}"),
            AmpsError::Authentication { message } => write!(f, "Authentication failed: {message}"),
            AmpsError::ConnectionRefused { message } => write!(f, "Connection refused: {message}"),
            AmpsError::Disconnected { message } => write!(f, "Disconnected: {message}"),
            AmpsError::NameInUse { message } => write!(f, "Client name in use: {message}"),
            AmpsError::NotEntitled { message } => write!(f, "Not entitled: {message}"),
            AmpsError::BadFilter { message } => write!(f, "Bad filter expression: {message}"),
            AmpsError::BadRegexTopic { message } => write!(f, "Bad regex topic: {message}"),
            AmpsError::BadSowKey { message } => write!(f, "Bad SOW key: {message}"),
            AmpsError::InvalidTopic { message } => write!(f, "Invalid topic: {message}"),
            AmpsError::Publish { message } => write!(f, "Publish error: {message}"),
            AmpsError::SubscriptionExists { message } => {
                write!(f, "Subscription already exists: {message}")
            }
            AmpsError::PublishStoreGap { message } => write!(f, "Publish store gap: {message}"),
            AmpsError::TimedOut { message } => write!(f, "Operation timed out: {message}"),
            AmpsError::Unknown { message } => write!(f, "Unknown error: {message}"),
            AmpsError::NullPointer => write!(f, "Null pointer error"),
            AmpsError::InvalidArgument { message } => write!(f, "Invalid argument: {message}"),
            AmpsError::NulError { message } => write!(f, "String contains null bytes: {message}"),
        }
    }
}

impl std::error::Error for AmpsError {}

impl From<ffi::amps_ffi_error_info_t> for AmpsError {
    /// Converts an FFI error info structure to an `AmpsError`.
    ///
    /// This conversion extracts the error code and message from the C-FFI
    /// error structure and maps it to the appropriate Rust error variant.
    fn from(error_info: ffi::amps_ffi_error_info_t) -> Self {
        // Convert the message from a C string to a Rust String
        // The message buffer may not be null-terminated if truncated,
        // so we need to handle that carefully.
        // Note: C char is i8 on some platforms (macOS), so we need to convert to u8.
        let message = unsafe {
            let len = error_info
                .message
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(error_info.message.len());
            let bytes = &error_info.message[..len];
            // Convert i8 slice to u8 slice for from_utf8_lossy
            let u8_bytes = std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len());
            String::from_utf8_lossy(u8_bytes).to_string()
        };

        match error_info.code {
            ffi::amps_ffi_error_t_AMPS_FFI_OK => {
                // This shouldn't happen - no error to convert
                AmpsError::Unknown {
                    message: "No error (OK code)".to_string(),
                }
            }
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_CONNECTION => AmpsError::Connection {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_ALREADY_CONNECTED => {
                AmpsError::AlreadyConnected {
                    message: message.clone(),
                }
            }
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_AUTHENTICATION => AmpsError::Authentication {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_CONNECTION_REFUSED => {
                AmpsError::ConnectionRefused {
                    message: message.clone(),
                }
            }
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_DISCONNECTED => AmpsError::Disconnected {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_NAME_IN_USE => AmpsError::NameInUse {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_NOT_ENTITLED => AmpsError::NotEntitled {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_BAD_FILTER => AmpsError::BadFilter {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_BAD_REGEX_TOPIC => AmpsError::BadRegexTopic {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_BAD_SOW_KEY => AmpsError::BadSowKey {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_INVALID_TOPIC => AmpsError::InvalidTopic {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_PUBLISH => AmpsError::Publish {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_SUBSCRIPTION_EXISTS => {
                AmpsError::SubscriptionExists {
                    message: message.clone(),
                }
            }
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_PUBLISH_STORE_GAP => {
                AmpsError::PublishStoreGap {
                    message: message.clone(),
                }
            }
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_TIMEOUT => AmpsError::TimedOut {
                message: message.clone(),
            },
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_NULL_POINTER => AmpsError::NullPointer,
            ffi::amps_ffi_error_t_AMPS_FFI_ERROR_INVALID_ARGUMENT => AmpsError::InvalidArgument {
                message: message.clone(),
            },
            _ => AmpsError::Unknown {
                message: format!("Unknown error code {}: {}", error_info.code, message),
            },
        }
    }
}

impl From<NulError> for AmpsError {
    /// Converts a `NulError` (from `CString::new`) to an `AmpsError`.
    fn from(error: NulError) -> Self {
        AmpsError::NulError {
            message: format!("String contains null byte at position {}", error.nul_position()),
        }
    }
}

/// Result type alias for AMPS operations.
///
/// This is a convenience type alias for `Result<T, AmpsError>`, used
/// throughout the AMPS Rust API.
pub type AmpsResult<T> = Result<T, AmpsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_connection() {
        let err = AmpsError::Connection {
            message: "test message".to_string(),
        };
        assert_eq!(err.to_string(), "Connection error: test message");
    }

    #[test]
    fn test_error_display_null_pointer() {
        let err = AmpsError::NullPointer;
        assert_eq!(err.to_string(), "Null pointer error");
    }

    #[test]
    fn test_from_ffi_error_connection() {
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_ERROR_CONNECTION,
            message: [0; 1024],
        };
        // Copy message into the fixed-size array (converting u8 to i8)
        let msg = b"Connection refused";
        for (i, &b) in msg.iter().enumerate() {
            error_info.message[i] = b as i8;
        }

        let err = AmpsError::from(error_info);
        assert!(matches!(err, AmpsError::Connection { .. }));
        if let AmpsError::Connection { message } = err {
            assert_eq!(message, "Connection refused");
        }
    }

    #[test]
    fn test_from_ffi_error_timeout() {
        let mut error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_ERROR_TIMEOUT,
            message: [0; 1024],
        };
        let msg = b"Operation timed out after 5000ms";
        for (i, &b) in msg.iter().enumerate() {
            error_info.message[i] = b as i8;
        }

        let err = AmpsError::from(error_info);
        assert!(matches!(err, AmpsError::TimedOut { .. }));
    }

    #[test]
    fn test_from_ffi_error_null_pointer() {
        let error_info = ffi::amps_ffi_error_info_t {
            code: ffi::amps_ffi_error_t_AMPS_FFI_ERROR_NULL_POINTER,
            message: [0; 1024],
        };

        let err = AmpsError::from(error_info);
        assert_eq!(err, AmpsError::NullPointer);
    }

    #[test]
    fn test_from_ffi_error_unknown_code() {
        let error_info = ffi::amps_ffi_error_info_t {
            code: 999, // Invalid code
            message: [0; 1024],
        };

        let err = AmpsError::from(error_info);
        assert!(matches!(err, AmpsError::Unknown { .. }));
    }

    #[test]
    fn test_from_nul_error() {
        let result = std::ffi::CString::new("hello\0world");
        assert!(result.is_err());

        let err = AmpsError::from(result.unwrap_err());
        assert!(matches!(err, AmpsError::NulError { .. }));
    }

    #[test]
    fn test_amps_result_type() {
        // Test that AmpsResult works as expected
        let ok_result: AmpsResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: AmpsResult<i32> = Err(AmpsError::NullPointer);
        assert!(err_result.is_err());
    }

    #[test]
    fn test_all_error_variants_display() {
        // Ensure all variants implement Display correctly
        let errors = vec![
            AmpsError::Connection { message: "test".into() },
            AmpsError::AlreadyConnected { message: "test".into() },
            AmpsError::Authentication { message: "test".into() },
            AmpsError::ConnectionRefused { message: "test".into() },
            AmpsError::Disconnected { message: "test".into() },
            AmpsError::NameInUse { message: "test".into() },
            AmpsError::NotEntitled { message: "test".into() },
            AmpsError::BadFilter { message: "test".into() },
            AmpsError::BadRegexTopic { message: "test".into() },
            AmpsError::BadSowKey { message: "test".into() },
            AmpsError::InvalidTopic { message: "test".into() },
            AmpsError::Publish { message: "test".into() },
            AmpsError::SubscriptionExists { message: "test".into() },
            AmpsError::PublishStoreGap { message: "test".into() },
            AmpsError::TimedOut { message: "test".into() },
            AmpsError::Unknown { message: "test".into() },
            AmpsError::NullPointer,
            AmpsError::InvalidArgument { message: "test".into() },
            AmpsError::NulError { message: "test".into() },
        ];

        for err in errors {
            let s = err.to_string();
            assert!(!s.is_empty(), "Error display should not be empty");
        }
    }
}

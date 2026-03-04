//! AMPS Rust FFI - Safe Rust bindings for the AMPS C++ client library.
//!
//! This crate provides Rust bindings for the AMPS (Advanced Message Processing System)
//! C++ client library, enabling publish/subscribe messaging, SOW queries, and more
//! from Rust applications.
//!
//! # FFI Module
//!
//! The [`ffi`](crate::ffi) module contains the low-level FFI bindings auto-generated
//! by bindgen from the C wrapper header file. These bindings are `unsafe` and require
//! careful handling of pointers and memory.
//!
//! A safe, idiomatic Rust API is planned for future development.

pub mod ffi;

#[cfg(test)]
mod tests {
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
        assert!(tokens.contains("amps_ffi_error_t"), "missing amps_ffi_error_t enum");
        assert!(tokens.contains("amps_ffi_error_info_t"), "missing amps_ffi_error_info_t struct");
        assert!(tokens.contains("amps_ffi_client_create"), "missing amps_ffi_client_create fn");
        assert!(tokens.contains("amps_ffi_client_destroy"), "missing amps_ffi_client_destroy fn");
        assert!(tokens.contains("amps_ffi_client_connect"), "missing amps_ffi_client_connect fn");
        assert!(tokens.contains("amps_ffi_client_disconnect"), "missing amps_ffi_client_disconnect fn");
        assert!(tokens.contains("amps_ffi_client_logon"), "missing amps_ffi_client_logon fn");
        assert!(tokens.contains("amps_ffi_client_publish"), "missing amps_ffi_client_publish fn");
        assert!(tokens.contains("amps_ffi_client_delta_publish"), "missing amps_ffi_client_delta_publish fn");
        assert!(tokens.contains("amps_ffi_client_subscribe"), "missing amps_ffi_client_subscribe fn");
        assert!(tokens.contains("amps_ffi_client_unsubscribe"), "missing amps_ffi_client_unsubscribe fn");
        assert!(tokens.contains("amps_ffi_client_sow"), "missing amps_ffi_client_sow fn");
        assert!(tokens.contains("amps_ffi_client_sow_and_subscribe"), "missing amps_ffi_client_sow_and_subscribe fn");
        assert!(tokens.contains("amps_ffi_message_get_data"), "missing amps_ffi_message_get_data fn");
        assert!(tokens.contains("amps_ffi_message_get_topic"), "missing amps_ffi_message_get_topic fn");
        assert!(tokens.contains("amps_ffi_error_string"), "missing amps_ffi_error_string fn");
        assert!(tokens.contains("amps_ffi_version"), "missing amps_ffi_version fn");
        assert!(tokens.contains("amps_ffi_client_set_heartbeat"), "missing amps_ffi_client_set_heartbeat fn");
        assert!(tokens.contains("amps_ffi_client_set_disconnect_handler"), "missing amps_ffi_client_set_disconnect_handler fn");
        assert!(tokens.contains("AMPS_FFI_OK"), "missing AMPS_FFI_OK constant");
        assert!(tokens.contains("AMPS_FFI_ERROR_CONNECTION"), "missing AMPS_FFI_ERROR_CONNECTION constant");
    }
}

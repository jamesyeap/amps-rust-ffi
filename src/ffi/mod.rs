//! FFI bindings for the AMPS C++ client library.
//!
//! This module contains the auto-generated FFI bindings created by bindgen
//! from the C wrapper header file (`c-wrapper/include/amps_ffi.h`).
//!
//! # Safety
//!
//! These bindings are unsafe by nature as they interact with C code. Users of this
//! module must ensure that:
//!
//! - Pointers passed to FFI functions are valid and properly aligned
//! - Strings are properly null-terminated when required
//! - The thread-safety requirements of the underlying AMPS library are respected
//!
//! For a safe Rust API, use the [`Client`](crate::Client) type instead.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

// Include the auto-generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

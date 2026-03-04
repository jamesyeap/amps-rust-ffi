use std::env;
use std::path::PathBuf;

fn main() {
    // Get the project root directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Path to AMPS client library
    let amps_client_lib_dir = manifest_dir.join("amps-client/lib");
    println!(
        "cargo:rustc-link-search=native={}",
        amps_client_lib_dir.display()
    );

    // Path to C++ wrapper library
    let wrapper_lib_dir = manifest_dir.join("c-wrapper/build");
    println!(
        "cargo:rustc-link-search=native={}",
        wrapper_lib_dir.display()
    );

    // Link to AMPS client library (must come before wrapper since wrapper depends on it)
    println!("cargo:rustc-link-lib=static=amps");

    // Link to C++ wrapper library
    println!("cargo:rustc-link-lib=static=amps_ffi");

    // Get target for platform-specific linking
    let target = env::var("TARGET").unwrap();

    // Link system libraries required by AMPS
    println!("cargo:rustc-link-lib=pthread");

    // Link system zlib - AMPS uses dynamic loading but we need to link it for the symbols
    println!("cargo:rustc-link-lib=z");

    // Link dynamic loader for amps_zlib.c (dlopen/dlsym)
    if target.contains("linux") || target.contains("apple") {
        println!("cargo:rustc-link-lib=dl");
    }

    // Link C++ standard library based on target
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("windows") {
        println!("cargo:rustc-link-lib=msvcrt");
    }

    // Tell cargo to invalidate the built crate whenever the header changes
    println!("cargo:rerun-if-changed=c-wrapper/include/amps_ffi.h");

    // Generate bindings using bindgen
    let bindings = bindgen::Builder::default()
        .header("c-wrapper/include/amps_ffi.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

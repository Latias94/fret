use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=FRET_WINDOWS_STACK_RESERVE_BYTES");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    // Windows defaults to a small main-thread stack reserve (often 1 MiB), which can overflow
    // in deep recursive layout (e.g. taffy).
    //
    // This sets a larger reserve for demo binaries (MSVC /STACK or GNU ld --stack).
    let reserve_bytes = env::var("FRET_WINDOWS_STACK_RESERVE_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(8 * 1024 * 1024);

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "msvc" {
        println!("cargo:rustc-link-arg=/STACK:{reserve_bytes}");
    } else {
        println!("cargo:rustc-link-arg=-Wl,--stack,{reserve_bytes}");
    }
}

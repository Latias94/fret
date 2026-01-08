fn main() {
    // Subsecond uses `GetProcAddress(..., "main")` on Windows to compute an ASLR anchor.
    // Export `main` explicitly when the `hotpatch` feature is enabled so `subsecond::aslr_reference()`
    // can return a non-zero address.
    //
    // Note: `dx serve --hotpatch` also configures the toolchain for hotpatching, but keeping this
    // local to `fret-demo` makes the smoke demos less fragile and easier to run via plain Cargo.
    let hotpatch_enabled = std::env::var_os("CARGO_FEATURE_HOTPATCH").is_some();
    if !hotpatch_enabled {
        return;
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        println!("cargo:rustc-link-arg=/EXPORT:main");
    }
}

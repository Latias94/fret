fn main() {
    #[cfg(windows)]
    {
        const DEFAULT_STACK_RESERVE_BYTES: usize = 8 * 1024 * 1024;
        let reserve_bytes = std::env::var("FRET_WINDOWS_STACK_RESERVE_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_STACK_RESERVE_BYTES);

        println!("cargo:rustc-link-arg=/STACK:{reserve_bytes}");
    }
}

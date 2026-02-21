#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    fret_examples::external_video_imports_mf_demo::run()
}

#[cfg(not(all(not(target_arch = "wasm32"), target_os = "windows")))]
fn main() -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "`external_video_imports_mf_demo` is Windows-only (Media Foundation)."
    ))
}

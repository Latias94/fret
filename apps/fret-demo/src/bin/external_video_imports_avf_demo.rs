#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn main() -> anyhow::Result<()> {
    fret_examples::external_video_imports_avf_demo::run()
}

#[cfg(not(all(not(target_arch = "wasm32"), target_os = "macos")))]
fn main() -> anyhow::Result<()> {
    eprintln!("external_video_imports_avf_demo is only supported on macOS");
    Ok(())
}

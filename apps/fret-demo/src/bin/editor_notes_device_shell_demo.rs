#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::editor_notes_device_shell_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

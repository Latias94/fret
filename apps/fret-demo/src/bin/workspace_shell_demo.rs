#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::workspace_shell_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

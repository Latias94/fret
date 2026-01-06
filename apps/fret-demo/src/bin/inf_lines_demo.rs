#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::inf_lines_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

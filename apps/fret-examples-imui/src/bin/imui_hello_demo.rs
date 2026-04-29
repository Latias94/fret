#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples_imui::imui_hello_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

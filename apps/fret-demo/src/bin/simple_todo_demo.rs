#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::simple_todo_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

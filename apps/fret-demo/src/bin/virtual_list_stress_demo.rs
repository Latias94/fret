#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::virtual_list_stress_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

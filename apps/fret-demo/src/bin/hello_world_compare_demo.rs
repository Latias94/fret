#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let _ = fret_alloc::allocator_name();
    fret_examples::hello_world_compare_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

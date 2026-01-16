#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::echarts_multi_grid_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

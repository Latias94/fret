#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    fret_examples::extras_marquee_perf_demo::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}

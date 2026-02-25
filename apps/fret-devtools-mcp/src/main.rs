#[cfg(target_arch = "wasm32")]
fn main() {
    // This binary is not supported on wasm targets.
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    native::run().await
}

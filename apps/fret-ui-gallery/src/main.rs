fn main() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        fret_ui_gallery::run()
    }

    #[cfg(target_arch = "wasm32")]
    {
        Ok(())
    }
}

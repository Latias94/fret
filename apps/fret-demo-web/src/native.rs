#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoError;

impl std::fmt::Display for DemoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fret-demo-web is only available on wasm32")
    }
}

impl std::error::Error for DemoError {}

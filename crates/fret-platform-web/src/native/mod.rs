#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformError;

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fret-platform-web is only available on wasm32")
    }
}

impl std::error::Error for PlatformError {}

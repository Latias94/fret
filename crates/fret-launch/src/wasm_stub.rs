#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunnerError;

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fret-launch desktop backend is not available on wasm32")
    }
}

impl std::error::Error for RunnerError {}

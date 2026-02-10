#[derive(Debug, thiserror::Error)]
pub enum KeymapError {
    #[error("failed to read keymap file")]
    ReadFailed { source: std::io::Error },
    #[error("failed to parse keymap json")]
    ParseFailed { source: serde_json::Error },
    #[error("unsupported keymap_version {0}")]
    UnsupportedVersion(u32),
    #[error("unknown platform value at binding[{index}]: {value}")]
    UnknownPlatform { index: usize, value: String },
    #[error("unknown key token at binding[{index}]: {token}")]
    UnknownKeyToken { index: usize, token: String },
    #[error("unknown modifier at binding[{index}]: {value}")]
    UnknownModifier { index: usize, value: String },
    #[error("empty keys sequence at binding[{index}]")]
    EmptyKeys { index: usize },
    #[error("failed to parse when at binding[{index}]: {error}")]
    WhenParseFailed { index: usize, error: String },
    #[error("invalid when expression at binding[{index}]: {error}")]
    WhenValidationFailed { index: usize, error: String },
}

#[derive(Debug, thiserror::Error)]
pub enum MenuBarError {
    #[error("failed to parse menubar json")]
    ParseFailed { source: serde_json::Error },
    #[error("unsupported menu_bar_version {0}")]
    UnsupportedVersion(u32),
    #[error("invalid when expression at {path}: {error}")]
    WhenParseFailed { path: String, error: String },
    #[error("invalid when expression at {path}: {error}")]
    WhenValidationFailed { path: String, error: String },
    #[error("menubar patch failed at ops[{index}]: {error}")]
    PatchFailed { index: usize, error: String },
}

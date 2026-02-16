use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Stable command identifier.
///
/// Command ids are used to register commands, route input, and bind key chords across the
/// workspace. They are expected to be stable and human-readable (e.g. `"app.quit"`,
/// `"workspace.toggle_sidebar"`).
pub struct CommandId(pub Arc<str>);

impl CommandId {
    /// Creates a new command id from an owned or shared string.
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    /// Returns the underlying id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for CommandId {
    fn from(value: &'static str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

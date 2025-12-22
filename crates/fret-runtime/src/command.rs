use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub Arc<str>);

impl CommandId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for CommandId {
    fn from(value: &'static str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

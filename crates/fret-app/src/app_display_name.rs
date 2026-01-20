use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDisplayName(pub Arc<str>);

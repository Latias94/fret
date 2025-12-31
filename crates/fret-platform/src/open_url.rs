//! Open-url platform contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenUrlErrorKind {
    Unsupported,
    BackendError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenUrlError {
    pub kind: OpenUrlErrorKind,
}

pub trait OpenUrl {
    fn open_url(&mut self, url: &str) -> Result<(), OpenUrlError>;
}

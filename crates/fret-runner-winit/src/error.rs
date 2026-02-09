#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerError {
    message: String,
}

impl RunnerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RunnerError {}

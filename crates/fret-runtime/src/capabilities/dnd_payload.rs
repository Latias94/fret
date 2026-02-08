use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExternalDragPayloadKind {
    None,
    FileToken,
    #[default]
    Text,
}

impl ExternalDragPayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FileToken => "file_token",
            Self::Text => "text",
        }
    }
}

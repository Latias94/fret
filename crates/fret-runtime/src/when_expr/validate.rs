use crate::capabilities::{CapabilityValueKind, capability_key_kind};

use super::ast::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WhenValueKind {
    Bool,
    Str,
}

impl From<CapabilityValueKind> for WhenValueKind {
    fn from(value: CapabilityValueKind) -> Self {
        match value {
            CapabilityValueKind::Bool => Self::Bool,
            CapabilityValueKind::Str => Self::Str,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WhenExprValidationError {
    #[error("unknown identifier: {name}")]
    UnknownIdentifier { name: String },
    #[error("identifier must be boolean in this context: {name}")]
    IdentifierNotBool { name: String },
    #[error("string literal is not a boolean expression")]
    StrUsedAsBool,
    #[error("type mismatch in comparison: left={left:?} right={right:?}")]
    ComparisonTypeMismatch {
        left: WhenValueKind,
        right: WhenValueKind,
    },
}

pub(super) fn ident_kind(name: &str) -> Result<WhenValueKind, WhenExprValidationError> {
    match name {
        "ui.has_modal" | "focus.is_text_input" => return Ok(WhenValueKind::Bool),
        "edit.can_undo" | "edit.can_redo" => return Ok(WhenValueKind::Bool),
        "platform" => return Ok(WhenValueKind::Str),
        _ => {}
    }

    let key = name.strip_prefix("cap.").unwrap_or(name);
    match capability_key_kind(key) {
        Some(kind) => Ok(kind.into()),
        None => Err(WhenExprValidationError::UnknownIdentifier {
            name: name.to_string(),
        }),
    }
}

pub(super) fn value_kind(v: &Value) -> Result<WhenValueKind, WhenExprValidationError> {
    match v {
        Value::Bool(_) => Ok(WhenValueKind::Bool),
        Value::Str(_) => Ok(WhenValueKind::Str),
        Value::Ident(id) => ident_kind(id),
    }
}

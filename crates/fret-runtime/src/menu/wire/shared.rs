use crate::WhenExpr;

use serde::Deserialize;

use super::super::MenuBarError;

pub(super) fn parse_when(path: &str, when: &str) -> Result<WhenExpr, MenuBarError> {
    let expr = WhenExpr::parse(when).map_err(|error| MenuBarError::WhenParseFailed {
        path: path.to_string(),
        error,
    })?;
    expr.validate()
        .map_err(|error| MenuBarError::WhenValidationFailed {
            path: path.to_string(),
            error: error.to_string(),
        })?;
    Ok(expr)
}

#[derive(Debug, Deserialize)]
pub(super) struct MenuBarVersionOnly {
    pub menu_bar_version: u32,
}

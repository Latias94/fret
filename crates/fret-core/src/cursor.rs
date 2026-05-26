use serde::{Deserialize, Serialize};

/// Portable cursor icon set for UI → host requests.
///
/// This is intentionally a small, cross-platform subset (desktop-first). Platforms may map these
/// to their closest native cursors, or treat them as no-ops if unsupported.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    EResize,
    WResize,
    ColResize,
    RowResize,
    NwseResize,
    NeswResize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_icon_directional_resize_serializes_stably() {
        assert_eq!(
            serde_json::to_string(&CursorIcon::EResize).expect("serialize e-resize cursor"),
            "\"e_resize\""
        );
        assert_eq!(
            serde_json::to_string(&CursorIcon::WResize).expect("serialize w-resize cursor"),
            "\"w_resize\""
        );
    }
}

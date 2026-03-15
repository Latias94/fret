//! Shared compact readout styling for non-edit editor text.
//!
//! This keeps trailing value/outcome labels on one subdued baseline without forcing a shared
//! container geometry across controls and proof surfaces.

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Color, Px, TextAlign, TextStyle};
use fret_ui::Theme;
use fret_ui::element::{LayoutStyle, TextProps};
use fret_ui_kit::typography;

use super::style::EditorStyle;

/// Resolve the compact readout text size from a base control text size.
///
/// The editor baseline keeps readouts one step quieter than primary editable text, but clamps at
/// a conservative floor so dense presets do not become illegible.
pub fn compact_readout_text_px(base_text_px: Px) -> Px {
    Px((base_text_px.0 - 1.0).max(11.0))
}

fn compact_readout_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

/// Shared compact non-edit readout text styling.
#[derive(Debug, Clone, Copy)]
pub struct EditorCompactReadoutStyle {
    pub text_px: Px,
    pub line_height: Px,
    pub color: Color,
}

impl EditorCompactReadoutStyle {
    pub fn resolve(theme: &Theme, line_height: Px) -> Self {
        let base_text_px = EditorStyle::resolve(theme).frame_chrome_small().text_px;
        Self {
            text_px: compact_readout_text_px(base_text_px),
            line_height,
            color: compact_readout_fg(theme),
        }
    }

    pub fn text_props(
        self,
        text: Arc<str>,
        layout: LayoutStyle,
        align: TextAlign,
        overflow: TextOverflow,
    ) -> TextProps {
        TextProps {
            layout,
            text,
            style: Some(typography::as_control_text(TextStyle {
                size: self.text_px,
                line_height: Some(self.line_height),
                ..Default::default()
            })),
            color: Some(self.color),
            wrap: TextWrap::None,
            overflow,
            align,
            ink_overflow: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compact_readout_text_px;
    use fret_core::Px;

    #[test]
    fn compact_readout_text_px_keeps_floor_for_small_base_sizes() {
        assert_eq!(compact_readout_text_px(Px(10.0)), Px(11.0));
        assert_eq!(compact_readout_text_px(Px(11.0)), Px(11.0));
    }

    #[test]
    fn compact_readout_text_px_trims_one_step_from_primary_text() {
        assert_eq!(compact_readout_text_px(Px(12.0)), Px(11.0));
        assert_eq!(compact_readout_text_px(Px(14.0)), Px(13.0));
    }
}

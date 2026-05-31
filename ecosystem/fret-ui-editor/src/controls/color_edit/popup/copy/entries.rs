use std::sync::Arc;

use fret_core::Color;

use super::super::super::model::format_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) enum ColorEditCopyFormat {
    FloatTuple,
    IntTuple,
    HexRgb,
    HexRgba,
}

impl ColorEditCopyFormat {
    pub(super) fn test_suffix(self) -> &'static str {
        match self {
            Self::FloatTuple => "float-tuple",
            Self::IntTuple => "int-tuple",
            Self::HexRgb => "hex-rgb",
            Self::HexRgba => "hex-rgba",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::controls::color_edit) struct ColorEditCopyEntry {
    pub(in crate::controls::color_edit) format: ColorEditCopyFormat,
    pub(in crate::controls::color_edit) text: Arc<str>,
}

pub(in crate::controls::color_edit) fn color_copy_entries(
    color: Color,
    show_alpha: bool,
) -> Vec<ColorEditCopyEntry> {
    let (r, g, b, a) = color_copy_u8_channels(color, show_alpha);
    let mut entries = vec![
        ColorEditCopyEntry {
            format: ColorEditCopyFormat::FloatTuple,
            text: Arc::from(format!(
                "({:.3}f, {:.3}f, {:.3}f, {:.3}f)",
                finite_or_zero(color.r),
                finite_or_zero(color.g),
                finite_or_zero(color.b),
                if show_alpha {
                    finite_or_zero(color.a)
                } else {
                    1.0
                }
            )),
        },
        ColorEditCopyEntry {
            format: ColorEditCopyFormat::IntTuple,
            text: Arc::from(format!("({r},{g},{b},{a})")),
        },
        ColorEditCopyEntry {
            format: ColorEditCopyFormat::HexRgb,
            text: format_hex(color, false),
        },
    ];

    if show_alpha {
        entries.push(ColorEditCopyEntry {
            format: ColorEditCopyFormat::HexRgba,
            text: format_hex(color, true),
        });
    }

    entries
}

fn color_copy_u8_channels(color: Color, show_alpha: bool) -> (u8, u8, u8, u8) {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(color);
    let r = ((rgb >> 16) & 0xff) as u8;
    let g = ((rgb >> 8) & 0xff) as u8;
    let b = (rgb & 0xff) as u8;
    let a = if show_alpha {
        (color.a.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };
    (r, g, b, a)
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

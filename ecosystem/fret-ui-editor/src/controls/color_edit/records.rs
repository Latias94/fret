use std::sync::Arc;

use fret_core::Color;
use fret_ui::action::{ActionCx, UiActionHost};

use super::drag_drop::palette_slot_drop_from_payload;

const COLOR_PRESETS: [(&str, u32); 12] = [
    ("Slate", 0x0f_17_2a),
    ("Red", 0xef_44_44),
    ("Orange", 0xf9_73_16),
    ("Amber", 0xf5_9e_0b),
    ("Yellow", 0xea_d3_08),
    ("Green", 0x22_c5_5e),
    ("Emerald", 0x10_b9_81),
    ("Cyan", 0x06_b6_d4),
    ("Blue", 0x3b_82_f6),
    ("Violet", 0x8b_5c_f6),
    ("Fuchsia", 0xd9_46_ef),
    ("White", 0xff_ff_ff),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorEditPaletteEntry {
    pub name: Arc<str>,
    pub rgb: u32,
}

impl ColorEditPaletteEntry {
    pub fn new(name: impl Into<Arc<str>>, rgb: u32) -> Self {
        Self {
            name: name.into(),
            rgb,
        }
    }

    pub fn with_rgb(mut self, rgb: u32) -> Self {
        self.rgb = rgb;
        self
    }
}

pub fn default_color_edit_palette() -> Arc<[ColorEditPaletteEntry]> {
    COLOR_PRESETS
        .iter()
        .map(|(name, rgb)| ColorEditPaletteEntry::new(*name, *rgb))
        .collect::<Vec<_>>()
        .into()
}

/// Color payload component shape used by `ColorEdit` drag/drop.
///
/// This mirrors Dear ImGui's standard `_COL3F` and `_COL4F` payload split while keeping the Fret
/// payload typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditDragDropComponents {
    /// RGB payload; dropping preserves the target alpha.
    Rgb,
    /// RGBA payload; dropping applies alpha only when the target exposes alpha editing.
    Rgba,
}

/// Typed color payload published and accepted by editor `ColorEdit` swatches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorEditDragDropPayload {
    color: Color,
    components: ColorEditDragDropComponents,
}

impl ColorEditDragDropPayload {
    pub fn from_color(color: Color, include_alpha: bool) -> Self {
        Self {
            color,
            components: if include_alpha {
                ColorEditDragDropComponents::Rgba
            } else {
                ColorEditDragDropComponents::Rgb
            },
        }
    }

    pub fn color(self) -> Color {
        self.color
    }

    pub fn components(self) -> ColorEditDragDropComponents {
        self.components
    }
}

/// App-owned palette slot mutation request emitted when a color payload is dropped onto a
/// `ColorEdit` popup palette entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorEditPaletteSlotDrop {
    index: usize,
    previous: ColorEditPaletteEntry,
    payload: ColorEditDragDropPayload,
    next: ColorEditPaletteEntry,
}

impl ColorEditPaletteSlotDrop {
    pub fn new(
        index: usize,
        previous: ColorEditPaletteEntry,
        payload: ColorEditDragDropPayload,
    ) -> Self {
        Self {
            index,
            next: palette_slot_drop_from_payload(previous.clone(), payload),
            previous,
            payload,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn previous(&self) -> &ColorEditPaletteEntry {
        &self.previous
    }

    pub fn payload(&self) -> ColorEditDragDropPayload {
        self.payload
    }

    pub fn next(&self) -> &ColorEditPaletteEntry {
        &self.next
    }
}

pub type OnColorEditPaletteSlotDrop =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, ColorEditPaletteSlotDrop) + 'static>;

/// App-owned eyedropper activation request emitted from an editor `ColorEdit` popup.
///
/// Fret does not currently expose a portable platform screen-sampling contract. This request keeps
/// the editor control useful for apps that already own an eyedropper implementation while avoiding
/// an implicit runtime or renderer readback dependency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorEditEyedropperRequest {
    current: Color,
    show_alpha: bool,
}

impl ColorEditEyedropperRequest {
    pub fn new(current: Color, show_alpha: bool) -> Self {
        Self {
            current,
            show_alpha,
        }
    }

    pub fn current(self) -> Color {
        self.current
    }

    pub fn show_alpha(self) -> bool {
        self.show_alpha
    }

    pub fn apply_sample(self, sampled: Color) -> Color {
        if self.show_alpha {
            sampled
        } else {
            let mut next = sampled;
            next.a = self.current.a;
            next
        }
    }
}

/// App-owned eyedropper activation hook for editor `ColorEdit`.
///
/// Return `Some(sampled_color)` for synchronous sampling and the control will update its color
/// model, draft text, and validation state. Return `None` for asynchronous app/platform flows.
pub type OnColorEditEyedropper = Arc<
    dyn Fn(&mut dyn UiActionHost, ActionCx, ColorEditEyedropperRequest) -> Option<Color> + 'static,
>;

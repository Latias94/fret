use std::sync::Arc;

use fret_core::Color;
use fret_ui::action::{ActionCx, UiActionHost};

use super::palette::ColorEditPaletteEntry;
use crate::controls::color_edit::drag_drop::palette_slot_drop_from_payload;

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

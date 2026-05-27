//! Shared visual chrome helpers for immediate controls.

use fret_core::Px;

// Dear ImGui's default style is compact and mostly square.
// Keep Fret IMUI slightly roomier than upstream to preserve a usable hit target, but use the
// same overall density direction instead of the old shadcn-form defaults.
pub(super) const CONTROL_RADIUS: Px = Px(2.0);
pub(super) const PANEL_RADIUS: Px = Px(4.0);
pub(super) const BUTTON_MIN_HEIGHT: Px = Px(24.0);
pub(super) const SMALL_BUTTON_MIN_HEIGHT: Px = Px(20.0);
pub(super) const FIELD_MIN_HEIGHT: Px = Px(24.0);
pub(super) const ARROW_BUTTON_SIZE: Px = Px(20.0);
pub(super) const RADIO_INDICATOR_SIZE: Px = Px(14.0);
pub(super) const RADIO_DOT_SIZE: Px = Px(6.0);
pub(super) const STACK_GAP: Px = Px(4.0);
pub(super) const ROW_GAP: Px = Px(8.0);
pub(super) const SLIDER_TRACK_HEIGHT: Px = Px(4.0);

mod chrome;
mod layout;
mod text;

pub(super) use chrome::{ImUiControlPalette, button_chrome, field_chrome};
pub(super) use layout::{centered_row_props, fill_row_props, fill_stack_props};
pub(super) use text::{caption_text, control_text, fill_text, pill};

#[cfg(test)]
mod tests;

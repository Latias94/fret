//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).

mod list_style;
mod sizing;
mod style;

pub mod button;
pub mod checkbox;
pub mod combobox;
pub mod command;
pub mod command_palette;
pub mod dialog;
pub mod dropdown_menu;
pub mod frame;
pub mod icon_button;
pub mod list_view;
pub mod progress;
pub mod resizable_panel_group;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod slider;
pub mod sonner;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod toolbar;
pub mod tooltip;
pub mod window_overlays;

pub use combobox::Combobox;
pub use sizing::{Sizable, Size};
pub use style::{ColorRef, MetricRef, Radius, Space, StyleRefinement};

pub use resizable_panel_group::ResizablePanelGroup;
pub use window_overlays::WindowOverlays;

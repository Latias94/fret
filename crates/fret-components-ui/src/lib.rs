//! General-purpose UI components built on top of `fret-ui`.
//!
//! This crate is intentionally domain-agnostic (no engine/editor-specific concepts).
//! Styling is token-driven and supports namespaced extension tokens (see ADR 0050).

mod list_style;
pub mod recipes;
mod sizing;
mod style;
mod styled;

pub mod app_menu_bar;
pub mod button;
pub mod checkbox;
pub mod combobox;
pub mod command;
pub mod command_palette;
pub mod command_palette_overlay;
pub mod context_menu;
pub mod dialog;
pub mod dialog_overlay;
pub mod dropdown_menu;
pub mod frame;
pub mod icon_button;
pub mod list_view;
pub mod popover;
pub mod progress;
pub mod resizable_panel_group;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod slider;
pub mod sonner;
pub mod switch;
pub mod tabs;
pub mod text_area_field;
pub mod text_field;
pub mod toast;
pub mod toolbar;
pub mod tooltip;
pub mod window_overlays;

pub use app_menu_bar::AppMenuBar;
pub use combobox::Combobox;
pub use command_palette_overlay::{CommandPaletteOverlay, CommandPaletteStyle};
pub use context_menu::{ContextMenu, ContextMenuStyle};
pub use dialog_overlay::{DialogAction, DialogOverlay, DialogRequest, DialogService, DialogStyle};
pub use sizing::{Sizable, Size};
pub use style::{ColorRef, MetricRef, Radius, Space, StyleRefinement};
pub use styled::{RefineStyle, Stylable, Styled, StyledExt};
pub use text_area_field::TextAreaField;
pub use toast::{ToastAction, ToastKind, ToastOverlay, ToastRequest, ToastService, ToastStyle};
pub use tooltip::{TooltipArea, TooltipOverlay, TooltipRequest, TooltipService, TooltipStyle};

pub use fret_ui::{
    ContextMenuRequest, ContextMenuService, MenuBarContextMenu, MenuBarContextMenuEntry,
};

pub use resizable_panel_group::ResizablePanelGroup;
pub use window_overlays::WindowOverlays;

pub use popover::{Popover, PopoverItem, PopoverRequest, PopoverService, PopoverStyle};

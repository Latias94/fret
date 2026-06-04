mod activation;
mod context_menu;
mod element;
mod visual;

use std::sync::Arc;

pub(super) use element::color_swatch;
use fret_core::{Color, Px};
use fret_runtime::Model;

use super::drag_drop::ColorDragDropStore;
use super::{
    ColorEditAlphaPreview, ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditPopupOptions,
    ColorEditTooltipOptions,
};

pub(super) struct ColorEditSwatchArgs {
    pub(super) model: Model<Color>,
    pub(super) open: Model<bool>,
    pub(super) tooltip_open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) current: Color,
    pub(super) current_hex: Arc<str>,
    pub(super) show_alpha: bool,
    pub(super) alpha_preview: ColorEditAlphaPreview,
    pub(super) enabled: bool,
    pub(super) swatch_enabled: bool,
    pub(super) swatch_focusable: bool,
    pub(super) popup_has_visible_content: bool,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) tooltip_options: ColorEditTooltipOptions,
    pub(super) copy_options: ColorEditCopyOptions,
    pub(super) copy_enabled: bool,
    pub(super) drag_drop_enabled: bool,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) test_id: Option<Arc<str>>,
}

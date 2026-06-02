mod slot;

use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use self::slot::preset_swatch;
use super::super::drag_drop::ColorDragDropStore;
use super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry,
    OnColorEditPaletteSlotDrop,
};

pub(super) fn preset_swatches<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    palette: Arc<[ColorEditPaletteEntry]>,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    swatch_row(
        cx,
        current,
        model,
        draft,
        error,
        open,
        show_alpha,
        enabled,
        alpha_preview,
        palette,
        drag_drop_store,
        drag_drop_options,
        drag_threshold,
        on_palette_slot_drop,
        "preset",
        test_id,
    )
}

pub(super) fn history_swatches<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    history: Arc<[ColorEditPaletteEntry]>,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    swatch_row(
        cx,
        current,
        model,
        draft,
        error,
        open,
        show_alpha,
        enabled,
        alpha_preview,
        history,
        drag_drop_store,
        drag_drop_options,
        drag_threshold,
        None,
        "history",
        test_id,
    )
}

fn swatch_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    entries: Arc<[ColorEditPaletteEntry]>,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    test_segment: &'static str,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let current_rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: true,
        },
        move |cx| {
            entries
                .iter()
                .enumerate()
                .map(|(idx, entry)| {
                    preset_swatch(
                        cx,
                        idx,
                        entry.clone(),
                        current_rgb == entry.rgb,
                        current.a,
                        model.clone(),
                        draft.clone(),
                        error.clone(),
                        open.clone(),
                        show_alpha,
                        enabled,
                        alpha_preview,
                        drag_drop_store.clone(),
                        drag_drop_options,
                        drag_threshold,
                        on_palette_slot_drop.clone(),
                        derived_test_id(test_id.as_ref(), format!("{test_segment}.{idx}").as_str()),
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

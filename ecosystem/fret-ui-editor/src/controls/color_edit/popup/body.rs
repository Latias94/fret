use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::super::drag_drop::ColorDragDropStore;
use super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry, ColorEditPopupOptions,
    ColorEditPopupRuntimeOptions, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
};

mod layout;
mod sections;

use layout::{color_popup_content, color_popup_width};
use sections::{ColorPopupBodySectionsArgs, color_popup_body_sections};

pub(super) struct ColorPopupBodyArgs {
    pub(super) model: Model<Color>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) open: Model<bool>,
    pub(super) rgb_draft: Model<String>,
    pub(super) hsv_draft: Model<String>,
    pub(super) numeric_error: Model<Option<Arc<str>>>,
    pub(super) show_alpha: bool,
    pub(super) enabled: bool,
    pub(super) alpha_preview: ColorEditAlphaPreview,
    pub(super) palette: Arc<[ColorEditPaletteEntry]>,
    pub(super) history: Arc<[ColorEditPaletteEntry]>,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    pub(super) on_eyedropper: Option<OnColorEditEyedropper>,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) popup_runtime_options: Model<ColorEditPopupRuntimeOptions>,
    pub(super) popup_padding: Px,
    pub(super) popup_test_id: Option<Arc<str>>,
    pub(super) eyedropper_test_id: Option<Arc<str>>,
}

pub(super) fn color_popup_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorPopupBodyArgs,
) -> AnyElement {
    let ColorPopupBodyArgs {
        model,
        reference,
        draft,
        error,
        open,
        rgb_draft,
        hsv_draft,
        numeric_error,
        show_alpha,
        enabled,
        alpha_preview,
        palette,
        history,
        drag_drop_store,
        drag_drop_options,
        drag_threshold,
        on_palette_slot_drop,
        on_eyedropper,
        popup_options,
        popup_runtime_options,
        popup_padding,
        popup_test_id,
        eyedropper_test_id,
    } = args;

    let popup_chrome = {
        let theme = Theme::global(&*cx.app);
        resolve_editor_popup_surface_chrome(theme, true)
    };
    let current = cx
        .get_model_copied(&model, Invalidation::Paint)
        .unwrap_or(Color::TRANSPARENT);
    let reference_color = cx
        .get_model_copied(&reference, Invalidation::Paint)
        .unwrap_or(None);
    let runtime_options = cx
        .get_model_copied(&popup_runtime_options, Invalidation::Paint)
        .unwrap_or_else(|| popup_options.runtime_defaults());
    let effective_popup_options = popup_options.with_runtime_options(runtime_options);
    let picker_for_width = effective_popup_options.picker;
    let sections = color_popup_body_sections(
        cx,
        ColorPopupBodySectionsArgs {
            current,
            reference_color,
            model: model.clone(),
            draft: draft.clone(),
            error: error.clone(),
            open: open.clone(),
            rgb_draft: rgb_draft.clone(),
            hsv_draft: hsv_draft.clone(),
            numeric_error: numeric_error.clone(),
            show_alpha,
            enabled,
            alpha_preview,
            palette: palette.clone(),
            history: history.clone(),
            drag_drop_store: drag_drop_store.clone(),
            drag_drop_options,
            drag_threshold,
            on_palette_slot_drop: on_palette_slot_drop.clone(),
            on_eyedropper,
            popup_options,
            popup_runtime_options: popup_runtime_options.clone(),
            runtime_options,
            effective_popup_options,
            popup_test_id: popup_test_id.clone(),
            eyedropper_test_id,
        },
    );
    let popup_width = color_popup_width(picker_for_width, sections.has_side_preview);
    let content = color_popup_content(cx, sections.content);
    let popup = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(popup_width),
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(popup_padding).into(),
            background: Some(popup_chrome.bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(popup_chrome.border),
            corner_radii: Corners::all(popup_chrome.radius),
            shadow: popup_chrome.shadow,
            ..Default::default()
        },
        move |_cx| vec![content],
    );

    if let Some(test_id) = popup_test_id.as_ref() {
        popup.test_id(test_id.clone())
    } else {
        popup
    }
}

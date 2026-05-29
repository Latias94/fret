//! Minimal color edit control (swatch + hex input + picker popup).
//!
//! v1 scope:
//! - hex input for `#RRGGBB` (and optionally `#RRGGBBAA`)
//! - swatch button that opens HSV picker controls plus app-owned palette/history swatches
//! - RGB-only edits preserve alpha; `show_alpha` only controls explicit alpha editing
//! - per-control alpha preview policy mirroring Dear ImGui's ColorButton preview modes

use std::panic::Location;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::input_group::derived_test_id;
use crate::primitives::readout::editor_inline_error_text_props;
use crate::primitives::{EditorDensity, EditorTokenKeys};

mod drag_drop;
mod input;
mod model;
mod options;
mod popup;
mod records;
mod state;
mod swatch;

#[cfg(test)]
mod tests;

use self::drag_drop::{
    apply_color_drop_payload, color_drag_drop_store_for, prune_color_drag_drop_store,
    resolve_color_drag_threshold, take_delivered_color_drop,
};
use self::input::{ColorEditInputArgs, color_hex_input};
use self::model::format_hex;
pub(in crate::controls::color_edit) use self::options::ColorEditPopupRuntimeOptions;
pub use self::options::{
    ColorEditAlphaPreview, ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditOptions,
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupSidePreview, ColorEditTooltipOptions,
};
use self::popup::{
    request_color_copy_menu_overlay, request_color_tooltip_overlay, request_popup_overlay,
};
pub use self::records::{
    ColorEditDragDropComponents, ColorEditDragDropPayload, ColorEditEyedropperRequest,
    ColorEditPaletteEntry, ColorEditPaletteSlotDrop, OnColorEditEyedropper,
    OnColorEditPaletteSlotDrop, default_color_edit_palette,
};
use self::state::{
    copy_menu_open_model, draft_model, error_model, popup_open_model, popup_runtime_options_model,
    reference_model, sync_popup_runtime_options, tooltip_open_model,
};
use self::swatch::{ColorEditSwatchArgs, color_swatch};

const CHECKERBOARD_LIGHT_RGB: u32 = 0xd8_de_e8;
const CHECKERBOARD_DARK_RGB: u32 = 0x8b_95_a5;
const ALPHA_BAR_STEPS: usize = 8;
const HUE_BAR_STEPS: usize = 12;
const SV_PICKER_STEPS: usize = 8;

#[derive(Clone)]
pub struct ColorEdit {
    model: Model<Color>,
    options: ColorEditOptions,
}

impl ColorEdit {
    pub fn new(model: Model<Color>) -> Self {
        Self {
            model,
            options: ColorEditOptions::default(),
        }
    }

    pub fn options(mut self, options: ColorEditOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.color_edit", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.color_edit", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let open = popup_open_model(cx);
        let tooltip_open = tooltip_open_model(cx);
        let copy_menu_open = copy_menu_open_model(cx);
        let reference = reference_model(cx);
        let draft = draft_model(cx);
        let error = error_model(cx);

        let (density, popup_padding) = {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);
            let popup_padding = theme
                .metric_by_key(EditorTokenKeys::COLOR_POPUP_PADDING)
                .unwrap_or(Px(8.0));
            (density, popup_padding)
        };

        let current = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or(Color::TRANSPARENT);
        let current_hex = format_hex(current, self.options.show_alpha);
        let drag_drop_store = color_drag_drop_store_for(cx);
        prune_color_drag_drop_store(cx, &drag_drop_store);
        let drag_drop_options = self.options.drag_drop;
        let drag_threshold = resolve_color_drag_threshold(cx);
        let input_test_id = self
            .options
            .input_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "input"));
        let swatch_test_id = self
            .options
            .swatch_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "swatch"));
        let popup_test_id = self
            .options
            .popup_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "popup"));
        let tooltip_test_id = self
            .options
            .tooltip_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "tooltip"));
        let copy_menu_test_id = self
            .options
            .copy_menu_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "copy-menu"));
        let eyedropper_test_id = self
            .options
            .eyedropper_test_id
            .clone()
            .or_else(|| derived_test_id(self.options.test_id.as_ref(), "eyedropper"));
        let popup_options = self.options.popup;
        let tooltip_options = self.options.tooltip;
        let copy_options = self.options.copy;
        let on_eyedropper = self.options.on_eyedropper.clone();
        let popup_runtime_options =
            popup_runtime_options_model(cx, popup_options.runtime_defaults());
        sync_popup_runtime_options(cx, &popup_runtime_options, popup_options.runtime_defaults());
        let popup_options_for_frame = popup_options.with_runtime_options(
            cx.get_model_copied(&popup_runtime_options, Invalidation::Paint)
                .unwrap_or_else(|| popup_options.runtime_defaults()),
        );
        let palette = self.options.palette.clone();
        let history = self.options.history.clone();
        let popup_has_visible_content = popup_options_for_frame.has_visible_content_with_swatches(
            self.options.show_alpha,
            !palette.is_empty(),
            !history.is_empty(),
        );
        let drag_drop_enabled = self.options.enabled && drag_drop_options.enabled;
        let tooltip_enabled = self.options.enabled && tooltip_options.enabled;
        let copy_enabled = self.options.enabled && copy_options.enabled;
        let eyedropper_enabled = self.options.enabled && on_eyedropper.is_some();
        let popup_has_visible_content = popup_has_visible_content || on_eyedropper.is_some();
        let swatch_enabled = self.options.enabled
            && (popup_has_visible_content
                || drag_drop_enabled
                || tooltip_enabled
                || copy_enabled
                || eyedropper_enabled);
        let swatch_focusable = self.options.focusable
            && (popup_has_visible_content
                || drag_drop_enabled
                || copy_enabled
                || eyedropper_enabled);

        let input = color_hex_input(
            cx,
            ColorEditInputArgs {
                model: self.model.clone(),
                draft: draft.clone(),
                error: error.clone(),
                current_hex: current_hex.clone(),
                show_alpha: self.options.show_alpha,
                enabled: self.options.enabled,
                focusable: self.options.focusable,
                test_id: input_test_id.clone(),
                row_height: density.row_height,
            },
        );

        let swatch = color_swatch(
            cx,
            ColorEditSwatchArgs {
                model: self.model.clone(),
                open: open.clone(),
                tooltip_open: tooltip_open.clone(),
                copy_menu_open: copy_menu_open.clone(),
                reference: reference.clone(),
                drag_drop_store: drag_drop_store.clone(),
                current,
                current_hex: current_hex.clone(),
                show_alpha: self.options.show_alpha,
                alpha_preview: self.options.alpha_preview,
                enabled: self.options.enabled,
                swatch_enabled,
                swatch_focusable,
                popup_has_visible_content,
                popup_options,
                tooltip_options,
                copy_options,
                copy_enabled,
                drag_drop_enabled,
                drag_drop_options,
                drag_threshold,
                test_id: swatch_test_id.clone(),
            },
        );

        if drag_drop_enabled
            && let Some(payload) = take_delivered_color_drop(cx, &drag_drop_store, swatch.id)
        {
            let current_for_drop = cx
                .get_model_copied(&self.model, Invalidation::Paint)
                .unwrap_or(current);
            let next = apply_color_drop_payload(payload, current_for_drop, self.options.show_alpha);
            let formatted = format_hex(next, self.options.show_alpha);
            let _ = cx
                .app
                .models_mut()
                .update(&self.model, |color| *color = next);
            let _ = cx
                .app
                .models_mut()
                .update(&draft, |s| *s = formatted.as_ref().to_string());
            let _ = cx.app.models_mut().update(&error, |e| *e = None);
        }

        request_popup_overlay(
            cx,
            swatch.id,
            self.model.clone(),
            reference.clone(),
            draft.clone(),
            error.clone(),
            open.clone(),
            self.options.show_alpha,
            self.options.enabled,
            self.options.alpha_preview,
            palette,
            history,
            drag_drop_store.clone(),
            drag_drop_options,
            drag_threshold,
            self.options.on_palette_slot_drop.clone(),
            on_eyedropper,
            popup_options,
            popup_runtime_options,
            popup_padding,
            popup_test_id,
            eyedropper_test_id,
        );
        request_color_tooltip_overlay(
            cx,
            swatch.id,
            tooltip_open,
            current,
            self.options.show_alpha,
            self.options.alpha_preview,
            tooltip_options,
            tooltip_test_id,
        );
        request_color_copy_menu_overlay(
            cx,
            swatch.id,
            copy_menu_open,
            current,
            self.options.show_alpha,
            copy_options,
            copy_menu_test_id,
        );

        let error_msg = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None);
        let error_el = error_msg.map(|msg| {
            cx.text_props(editor_inline_error_text_props(
                msg,
                Theme::global(&*cx.app).color_token("destructive"),
                density.row_height,
            ))
        });

        let mut root_layout = self.options.layout;
        if root_layout.size.min_height.is_none() {
            root_layout.size.min_height = Some(Length::Px(density.row_height));
        }

        let mut el = cx.flex(
            FlexProps {
                layout: root_layout,
                direction: Axis::Vertical,
                gap: SpacingLength::Px(Px(4.0)),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |cx| {
                let row = cx.flex(
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
                        gap: SpacingLength::Px(Px(8.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| vec![swatch, input],
                );

                let mut out = vec![row];
                if let Some(err) = error_el {
                    out.push(err);
                }
                out
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
    }
}

use std::panic::Location;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::{EditorDensity, EditorTokenKeys};

mod test_ids;

use self::test_ids::color_edit_element_test_ids;

use super::drag_drop::{
    ColorEditDeliveredDropArgs, apply_delivered_color_drop, color_drag_drop_store_for,
    prune_color_drag_drop_store, resolve_color_drag_threshold,
};
use super::input::{ColorEditInputArgs, color_hex_input};
use super::layout::{ColorEditRootLayoutArgs, color_edit_root_layout};
use super::model::format_hex;
use super::options::ColorEditOptions;
use super::popup::{
    request_color_copy_menu_overlay, request_color_tooltip_overlay, request_popup_overlay,
};
use super::state::{
    copy_menu_open_model, draft_model, error_model, popup_open_model, popup_runtime_options_model,
    reference_model, sync_popup_runtime_options, tooltip_open_model,
};
use super::swatch::{ColorEditSwatchArgs, color_swatch};

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
        let test_ids = color_edit_element_test_ids(&self.options);
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
                test_id: test_ids.input.clone(),
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
                test_id: test_ids.swatch.clone(),
            },
        );

        apply_delivered_color_drop(
            cx,
            ColorEditDeliveredDropArgs {
                store: drag_drop_store.clone(),
                target_id: swatch.id,
                model: self.model.clone(),
                draft: draft.clone(),
                error: error.clone(),
                current,
                show_alpha: self.options.show_alpha,
                enabled: drag_drop_enabled,
            },
        );

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
            test_ids.popup,
            test_ids.eyedropper,
        );
        request_color_tooltip_overlay(
            cx,
            swatch.id,
            tooltip_open,
            current,
            self.options.show_alpha,
            self.options.alpha_preview,
            tooltip_options,
            test_ids.tooltip,
        );
        request_color_copy_menu_overlay(
            cx,
            swatch.id,
            copy_menu_open,
            current,
            self.options.show_alpha,
            copy_options,
            test_ids.copy_menu,
        );

        color_edit_root_layout(
            cx,
            ColorEditRootLayoutArgs {
                swatch,
                input,
                error,
                layout: self.options.layout,
                test_id: self.options.test_id.clone(),
                row_height: density.row_height,
            },
        )
    }
}

//! Minimal color edit control (swatch + hex input + picker popup).
//!
//! v1 scope:
//! - hex input for `#RRGGBB` (and optionally `#RRGGBBAA`)
//! - swatch button that opens HSV picker controls plus app-owned palette/history swatches
//! - RGB-only edits preserve alpha; `show_alpha` only controls explicit alpha editing
//! - per-control alpha preview policy mirroring Dear ImGui's ColorButton preview modes

use std::panic::Location;
use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, KeyCode, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, PressablePointerDownResult};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::input_group::derived_test_id;
use crate::primitives::readout::editor_inline_error_text_props;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};
use crate::primitives::{EditorDensity, EditorTokenKeys};

mod drag_drop;
mod input;
mod model;
mod options;
mod popup;
mod records;
mod state;

#[cfg(test)]
mod tests;

use self::drag_drop::{
    apply_color_drop_payload, color_drag_drop_store_for, install_color_drag_source,
    prune_color_drag_drop_store, resolve_color_drag_threshold, take_delivered_color_drop,
    update_color_drop_target,
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
    color_preview_stack, request_color_copy_menu_overlay, request_color_tooltip_overlay,
    request_popup_overlay,
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

        let (density, frame_chrome, swatch_size, popup_padding, ring) = {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);
            let frame_chrome = EditorStyle::resolve(theme).frame_chrome_small();
            let swatch_size = theme
                .metric_by_key(EditorTokenKeys::COLOR_SWATCH_SIZE)
                .unwrap_or(density.icon_size);
            let popup_padding = theme
                .metric_by_key(EditorTokenKeys::COLOR_POPUP_PADDING)
                .unwrap_or(Px(8.0));
            let ring = theme
                .color_by_key("ring")
                .unwrap_or_else(|| theme.color_token("primary"));
            (density, frame_chrome, swatch_size, popup_padding, ring)
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

        let swatch = {
            let open_for_activate = open.clone();
            let open_for_paint = open.clone();
            let tooltip_open_for_paint = tooltip_open.clone();
            let copy_menu_open_for_activate = copy_menu_open.clone();
            let copy_menu_open_for_pointer = copy_menu_open.clone();
            let copy_menu_open_for_paint = copy_menu_open.clone();
            let open_for_pointer = open.clone();
            let tooltip_open_for_pointer = tooltip_open.clone();
            let reference_for_activate = reference.clone();
            let model_for_activate = self.model.clone();
            let enabled_for_paint = self.options.enabled;
            let drag_drop_store_for_swatch = drag_drop_store.clone();
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
                    if !popup_has_visible_content {
                        return;
                    }
                    let prev = host
                        .models_mut()
                        .get_copied(&open_for_activate)
                        .unwrap_or(false);
                    let opening = !prev;
                    if opening && popup_options.side_preview.shows_original() {
                        let current = host
                            .models_mut()
                            .get_copied(&model_for_activate)
                            .unwrap_or(Color::TRANSPARENT);
                        let _ = host
                            .models_mut()
                            .update(&reference_for_activate, |reference| {
                                *reference = Some(current)
                            });
                    }
                    let _ = host
                        .models_mut()
                        .update(&open_for_activate, |v| *v = opening);
                    let _ = host
                        .models_mut()
                        .update(&copy_menu_open_for_activate, |v| *v = false);
                    host.request_redraw(action_cx.window);
                });

            let mut swatch = cx.pressable(
                PressableProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(density.hit_thickness),
                            height: Length::Px(density.hit_thickness),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    enabled: swatch_enabled,
                    focusable: swatch_enabled && swatch_focusable,
                    a11y: PressableA11y {
                        role: Some(fret_core::SemanticsRole::Button),
                        label: Some(Arc::from("Color swatch")),
                        ..Default::default()
                    },
                    focus_ring: Some(fret_ui::element::RingStyle {
                        placement: fret_ui::element::RingPlacement::Outset,
                        width: Px(2.0),
                        offset: Px(2.0),
                        color: ring,
                        offset_color: None,
                        corner_radii: Corners::all(frame_chrome.radius),
                    }),
                    ..Default::default()
                },
                move |cx, st| {
                    cx.pressable_add_on_activate(on_activate.clone());
                    if copy_options.enabled {
                        cx.pressable_add_on_pointer_down(Arc::new({
                            let copy_menu_open_for_pointer = copy_menu_open_for_pointer.clone();
                            let open_for_pointer = open_for_pointer.clone();
                            let tooltip_open_for_pointer = tooltip_open_for_pointer.clone();
                            move |host, action_cx, down| {
                                let is_context_menu = down.button == MouseButton::Right
                                    || (cfg!(target_os = "macos")
                                        && down.button == MouseButton::Left
                                        && down.modifiers.ctrl);
                                if !is_context_menu {
                                    return PressablePointerDownResult::Continue;
                                }

                                let _ = host
                                    .models_mut()
                                    .update(&open_for_pointer, |value| *value = false);
                                let _ = host
                                    .models_mut()
                                    .update(&tooltip_open_for_pointer, |value| *value = false);
                                let _ = host
                                    .models_mut()
                                    .update(&copy_menu_open_for_pointer, |value| *value = true);
                                host.request_focus(action_cx.target);
                                host.request_redraw(action_cx.window);
                                PressablePointerDownResult::SkipDefaultAndStopPropagation
                            }
                        }));
                    }
                    let swatch_id = cx.root_id();
                    install_color_drag_source(
                        cx,
                        swatch_id,
                        drag_drop_store_for_swatch.clone(),
                        ColorEditDragDropPayload::from_color(current, self.options.show_alpha),
                        drag_drop_options,
                        drag_threshold,
                    );
                    let drop_over = update_color_drop_target(
                        cx,
                        &drag_drop_store_for_swatch,
                        swatch_id,
                        st.hovered_raw,
                        drag_drop_enabled,
                    );

                    let is_open = cx
                        .get_model_copied(&open_for_paint, Invalidation::Paint)
                        .unwrap_or(false);
                    let copy_menu_is_open = cx
                        .get_model_copied(&copy_menu_open_for_paint, Invalidation::Paint)
                        .unwrap_or(false);
                    let tooltip_visible = tooltip_options.enabled
                        && enabled_for_paint
                        && !is_open
                        && !copy_menu_is_open
                        && st.hovered_raw;
                    let tooltip_open_now = cx
                        .get_model_copied(&tooltip_open_for_paint, Invalidation::Paint)
                        .unwrap_or(false);
                    if tooltip_open_now != tooltip_visible {
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&tooltip_open_for_paint, |value| *value = tooltip_visible);
                    }
                    let visuals = {
                        let theme = Theme::global(&*cx.app);
                        EditorWidgetVisuals::new(theme).frame_visuals(
                            frame_chrome,
                            EditorFrameState {
                                enabled: enabled_for_paint,
                                hovered: st.hovered || st.hovered_raw,
                                pressed: st.pressed || drop_over,
                                focused: st.focused,
                                open: (is_open && popup_has_visible_content) || copy_menu_is_open,
                                semantic: EditorFrameSemanticState::default(),
                            },
                        )
                    };

                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(swatch_size),
                                    height: Length::Px(swatch_size),
                                    ..Default::default()
                                },
                                overflow: Overflow::Clip,
                                ..Default::default()
                            },
                            border: Edges::all(frame_chrome.border_width),
                            border_color: Some(visuals.border),
                            corner_radii: Corners::all(frame_chrome.radius),
                            padding: Edges::all(frame_chrome.border_width).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![color_preview_stack(
                                cx,
                                current,
                                frame_chrome.radius,
                                self.options.alpha_preview,
                            )]
                        },
                    )]
                },
            );

            if let Some(test_id) = swatch_test_id.as_ref() {
                swatch = swatch.test_id(test_id.clone());
            }
            swatch = swatch.a11y_value(current_hex.clone());
            if copy_enabled {
                let open_for_key = open.clone();
                let tooltip_open_for_key = tooltip_open.clone();
                let copy_menu_open_for_key = copy_menu_open.clone();
                cx.key_on_key_down_for(
                    swatch.id,
                    Arc::new(move |host, action_cx, down| {
                        if down.repeat {
                            return false;
                        }

                        let no_extra_modifiers = !down.modifiers.ctrl
                            && !down.modifiers.alt
                            && !down.modifiers.meta
                            && !down.modifiers.alt_gr;
                        let is_shift_f10 =
                            down.key == KeyCode::F10 && down.modifiers.shift && no_extra_modifiers;
                        let is_context_menu_key = down.key == KeyCode::ContextMenu
                            && !down.modifiers.shift
                            && no_extra_modifiers;
                        if !is_shift_f10 && !is_context_menu_key {
                            return false;
                        }

                        let _ = host
                            .models_mut()
                            .update(&open_for_key, |value| *value = false);
                        let _ = host
                            .models_mut()
                            .update(&tooltip_open_for_key, |value| *value = false);
                        let _ = host
                            .models_mut()
                            .update(&copy_menu_open_for_key, |value| *value = true);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );
            }
            swatch
        };

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

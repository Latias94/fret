//! Minimal color edit control (swatch + hex input + picker popup).
//!
//! v1 scope:
//! - hex input for `#RRGGBB` (and optionally `#RRGGBBAA`)
//! - swatch button that opens HSV picker controls plus a small preset palette
//! - RGB-only edits preserve alpha; `show_alpha` only controls explicit alpha editing
//! - per-control alpha preview policy mirroring Dear ImGui's ColorButton preview modes

use std::panic::Location;
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Color, Corners, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, SizeStyle, SpacingLength, TextInputProps,
    TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};
use crate::primitives::{EditorDensity, EditorTokenKeys};

mod drag_drop;
mod model;
mod popup;

#[cfg(test)]
mod tests;

use self::drag_drop::{
    apply_color_drop_payload, color_drag_drop_store_for, install_color_drag_source,
    prune_color_drag_drop_store, resolve_color_drag_threshold, take_delivered_color_drop,
    update_color_drop_target,
};
use self::model::{format_hex, parse_hex};
use self::popup::{color_preview_stack, request_popup_overlay};

const COLOR_PRESETS: [(&str, u32); 12] = [
    ("Slate", 0x0f_17_2a),
    ("Red", 0xef_44_44),
    ("Orange", 0xf9_73_16),
    ("Amber", 0xf5_9e_0b),
    ("Yellow", 0xea_d3_08),
    ("Green", 0x22_c5_5e),
    ("Emerald", 0x10_b9_81),
    ("Cyan", 0x06_b6_d4),
    ("Blue", 0x3b_82_f6),
    ("Violet", 0x8b_5c_f6),
    ("Fuchsia", 0xd9_46_ef),
    ("White", 0xff_ff_ff),
];

const CHECKERBOARD_LIGHT_RGB: u32 = 0xd8_de_e8;
const CHECKERBOARD_DARK_RGB: u32 = 0x8b_95_a5;
const ALPHA_BAR_STEPS: usize = 8;
const HUE_BAR_STEPS: usize = 12;
const SV_PICKER_STEPS: usize = 8;

/// Alpha preview policy for `ColorEdit` swatches.
///
/// Dear ImGui exposes this as `AlphaOpaque`, `AlphaNoBg`, and `AlphaPreviewHalf` flags on
/// `ColorButton` / `ColorEdit`. Fret keeps it as explicit per-control editor policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditAlphaPreview {
    /// Show transparent colors over a checkerboard background.
    Checkerboard,
    /// Show the current RGB channels as fully opaque in preview only.
    Opaque,
    /// Show the color with its real alpha without a checkerboard background.
    NoBackground,
    /// Split the preview between opaque RGB and transparent checkerboard-backed RGB.
    Half,
}

impl Default for ColorEditAlphaPreview {
    fn default() -> Self {
        Self::Checkerboard
    }
}

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
    pub color: Color,
    pub components: ColorEditDragDropComponents,
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
}

/// Per-control color drag/drop policy for editor `ColorEdit`.
///
/// Dear ImGui enables color drag/drop by default and uses `NoDragDrop` as the opt-out flag. Fret
/// keeps the same default for local editor payloads while making cross-window routing explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditDragDropOptions {
    pub enabled: bool,
    pub cross_window: bool,
}

impl Default for ColorEditDragDropOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            cross_window: false,
        }
    }
}

/// Picker surface shown inside the `ColorEdit` popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditPopupPicker {
    /// Dear ImGui's default `PickerHueBar` shape: saturation/value area plus a hue bar.
    HsvHueBar,
    /// Hide the picker surface while keeping other popup affordances available.
    Hidden,
}

impl Default for ColorEditPopupPicker {
    fn default() -> Self {
        Self::HsvHueBar
    }
}

/// Numeric edit rows shown inside the `ColorEdit` popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditPopupNumericInputs {
    /// Show both RGB and HSV numeric rows.
    RgbAndHsv,
    /// Show only the RGB numeric row.
    Rgb,
    /// Show only the HSV numeric row.
    Hsv,
    /// Hide numeric edit rows.
    Hidden,
}

impl Default for ColorEditPopupNumericInputs {
    fn default() -> Self {
        Self::RgbAndHsv
    }
}

/// Per-control popup defaults for editor `ColorEdit`.
///
/// Dear ImGui stores color edit defaults in the global context via `SetColorEditOptions()`. Fret
/// keeps that policy explicit and app-owned: each editor control receives the popup defaults it
/// should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditPopupOptions {
    pub picker: ColorEditPopupPicker,
    pub numeric_inputs: ColorEditPopupNumericInputs,
    pub presets: bool,
    pub alpha_bar: bool,
}

impl ColorEditPopupOptions {
    fn has_visible_content(self, show_alpha: bool) -> bool {
        self.picker != ColorEditPopupPicker::Hidden
            || self.numeric_inputs != ColorEditPopupNumericInputs::Hidden
            || self.presets
            || self.shows_alpha_bar(show_alpha)
    }

    fn shows_alpha_bar(self, show_alpha: bool) -> bool {
        show_alpha && self.alpha_bar
    }
}

impl Default for ColorEditPopupOptions {
    fn default() -> Self {
        Self {
            picker: ColorEditPopupPicker::default(),
            numeric_inputs: ColorEditPopupNumericInputs::default(),
            presets: true,
            alpha_bar: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorEditOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub show_alpha: bool,
    pub alpha_preview: ColorEditAlphaPreview,
    pub drag_drop: ColorEditDragDropOptions,
    pub popup: ColorEditPopupOptions,
    /// Explicit identity source for internal state (draft/error/open models, overlay root ids).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple color edits from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub swatch_test_id: Option<Arc<str>>,
    pub input_test_id: Option<Arc<str>>,
    pub popup_test_id: Option<Arc<str>>,
}

impl Default for ColorEditOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            show_alpha: false,
            alpha_preview: ColorEditAlphaPreview::default(),
            drag_drop: ColorEditDragDropOptions::default(),
            popup: ColorEditPopupOptions::default(),
            id_source: None,
            test_id: None,
            swatch_test_id: None,
            input_test_id: None,
            popup_test_id: None,
        }
    }
}

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
        let popup_options = self.options.popup;
        let popup_has_visible_content = popup_options.has_visible_content(self.options.show_alpha);
        let drag_drop_enabled = self.options.enabled && drag_drop_options.enabled;
        let swatch_enabled =
            self.options.enabled && (popup_has_visible_content || drag_drop_enabled);

        let input = {
            let (chrome, text_style) = {
                let theme = Theme::global(&*cx.app);
                let (chrome, text_style) = resolve_editor_text_field_style(
                    theme,
                    Size::default(),
                    &ChromeRefinement::default(),
                );
                (chrome, text_style)
            };

            // Keep the draft synced while not focused so external updates (undo, scripts) show up.
            let mut props = TextInputProps::new(draft.clone());
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    min_height: Some(Length::Px(density.row_height)),
                    ..Default::default()
                },
                ..Default::default()
            };
            props.enabled = self.options.enabled;
            props.focusable = self.options.focusable;
            props.test_id = input_test_id.clone();
            props.chrome = chrome;
            props.text_style = text_style;

            let input = cx.text_input(props);
            let input_id = input.id;
            let is_focused = cx.is_focused_element(input_id);

            if !is_focused {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&draft, |s| *s = current_hex.as_ref().to_string());
                let _ = cx.app.models_mut().update(&error, |e| *e = None);
            }

            let model_for_key = self.model.clone();
            let draft_for_key = draft.clone();
            let error_for_key = error.clone();
            let show_alpha = self.options.show_alpha;
            cx.key_add_on_key_down_capture_for(
                input_id,
                Arc::new(move |host, action_cx: ActionCx, down| match down.key {
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        let text = host
                            .models_mut()
                            .read(&draft_for_key, |s| s.clone())
                            .unwrap_or_default();
                        let current = host
                            .models_mut()
                            .get_copied(&model_for_key)
                            .unwrap_or(Color::TRANSPARENT);
                        if let Some(next) = parse_hex(&text, show_alpha, current) {
                            let _ = host.models_mut().update(&model_for_key, |c| *c = next);
                            let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                        } else {
                            let _ = host
                                .models_mut()
                                .update(&error_for_key, |e| *e = Some(Arc::from("Invalid color")));
                        }
                        host.request_redraw(action_cx.window);
                        true
                    }
                    KeyCode::Escape => {
                        let current = host
                            .models_mut()
                            .get_copied(&model_for_key)
                            .unwrap_or_else(|| Color::from_srgb_hex_rgb(0x00_00_00));
                        let formatted = format_hex(current, show_alpha);
                        let _ = host
                            .models_mut()
                            .update(&draft_for_key, |s| *s = formatted.as_ref().to_string());
                        let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                        host.request_redraw(action_cx.window);
                        true
                    }
                    _ => false,
                }),
            );

            cx.pointer_region(
                PointerRegionProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            min_height: Some(Length::Px(density.row_height)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    enabled: self.options.enabled && self.options.focusable,
                    capture_phase_pointer_moves: false,
                },
                move |cx| {
                    cx.pointer_region_on_pointer_down(Arc::new(move |host, action_cx, _down| {
                        host.request_focus(input_id);
                        host.request_redraw(action_cx.window);
                        false
                    }));
                    vec![input]
                },
            )
        };

        let swatch = {
            let open_for_activate = open.clone();
            let open_for_paint = open.clone();
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
                    let _ = host.models_mut().update(&open_for_activate, |v| *v = !prev);
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
                    focusable: swatch_enabled && self.options.focusable,
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
                    let visuals = {
                        let theme = Theme::global(&*cx.app);
                        EditorWidgetVisuals::new(theme).frame_visuals(
                            frame_chrome,
                            EditorFrameState {
                                enabled: enabled_for_paint,
                                hovered: st.hovered || st.hovered_raw,
                                pressed: st.pressed || drop_over,
                                focused: st.focused,
                                open: is_open && popup_has_visible_content,
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
            draft.clone(),
            error.clone(),
            open.clone(),
            self.options.show_alpha,
            self.options.enabled,
            self.options.alpha_preview,
            popup_options,
            popup_padding,
            popup_test_id,
        );

        let error_msg = cx
            .get_model_cloned(&error, Invalidation::Paint)
            .unwrap_or(None);
        let error_el = error_msg.map(|msg| {
            cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: msg,
                style: Some(typography::as_control_text(TextStyle {
                    size: Px(10.0),
                    line_height: Some(density.row_height),
                    ..Default::default()
                })),
                color: Some(Theme::global(&*cx.app).color_token("destructive")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            })
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

#[track_caller]
fn popup_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}

#[track_caller]
fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    cx.local_model(|| None::<Arc<str>>)
}

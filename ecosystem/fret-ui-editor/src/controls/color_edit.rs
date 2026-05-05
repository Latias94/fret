//! Minimal color edit control (swatch + hex input + picker popup).
//!
//! v1 scope:
//! - hex input for `#RRGGBB` (and optionally `#RRGGBBAA`)
//! - swatch button that opens HSV picker controls plus a small preset palette
//! - RGB-only edits preserve alpha; `show_alpha` only controls explicit alpha editing

use std::panic::Location;
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{
    Axis, Color, Corners, Edges, KeyCode, MouseButton, Px, SemanticsInvalid, TextAlign, TextStyle,
};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, ActivateReason, OnActivate, OnCloseAutoFocus, PressablePointerDownResult,
    PressablePointerUpResult, UiPointerActionHost,
};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, GridProps, GridTrackSizing,
    InsetStyle, LayoutStyle, Length, MainAlign, Overflow, PointerRegionProps, PositionStyle,
    PressableA11y, PressableProps, SizeStyle, SpacingLength, StackProps, TextInputProps, TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, OverlayController, OverlayPresence, OverlayRequest, Size};

use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};
use crate::primitives::{EditorDensity, EditorTokenKeys};

mod model;

#[cfg(test)]
mod tests;

use self::model::{
    ColorNumericInputMode, HsvColor, color_from_rgb_preserving_alpha, color_numeric_input_modes,
    color_numeric_text, format_hex, hsv_from_color, hsv_numeric_text,
    hsv_to_color_preserving_alpha, hsv_with_sv_from_local_position, hue_from_local_x,
    hue_percent_text, parse_color_numeric_input, parse_hex, rgb_numeric_text, sv_picker_a11y_text,
    unit_from_step,
};

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
        let swatch_enabled =
            self.options.enabled && popup_options.has_visible_content(self.options.show_alpha);

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
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
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
                                pressed: st.pressed,
                                focused: st.focused,
                                open: is_open,
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
                        move |cx| vec![color_preview_stack(cx, current, frame_chrome.radius)],
                    )]
                },
            );

            if let Some(test_id) = swatch_test_id.as_ref() {
                swatch = swatch.test_id(test_id.clone());
            }
            swatch = swatch.a11y_value(current_hex.clone());
            swatch
        };

        request_popup_overlay(
            cx,
            swatch.id,
            self.model.clone(),
            draft.clone(),
            error.clone(),
            open.clone(),
            self.options.show_alpha,
            self.options.enabled,
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

fn request_popup_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    popup_options: ColorEditPopupOptions,
    popup_padding: Px,
    popup_test_id: Option<Arc<str>>,
) {
    if !popup_options.has_visible_content(show_alpha) {
        return;
    }

    let rgb_draft = draft_model(cx);
    let hsv_draft = draft_model(cx);
    let numeric_error = error_model(cx);
    let overlay_id = cx
        .named("color_edit.popup", |cx| cx.spacer(Default::default()))
        .id;
    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);

    let close_focus: OnCloseAutoFocus = Arc::new(move |host, _cx, req| {
        req.prevent_default();
        host.request_focus(swatch_id);
    });

    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        Px(4.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)));

    let open_for_content = open.clone();
    let popup = cx.anchored_props(
        fret_ui::element::AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            outer_margin: Edges::all(Px(0.0)),
            anchor_element: Some(swatch_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
            let popup_chrome = {
                let theme = Theme::global(&*cx.app);
                resolve_editor_popup_surface_chrome(theme, true)
            };
            let current = cx
                .get_model_copied(&model, Invalidation::Paint)
                .unwrap_or(Color::TRANSPARENT);
            let current_rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
            let picker = match popup_options.picker {
                ColorEditPopupPicker::HsvHueBar => Some(hsv_picker(
                    cx,
                    current,
                    model.clone(),
                    draft.clone(),
                    error.clone(),
                    show_alpha,
                    enabled,
                    derived_test_id(popup_test_id.as_ref(), "hsv"),
                )),
                ColorEditPopupPicker::Hidden => None,
            };
            let numbers = (popup_options.numeric_inputs != ColorEditPopupNumericInputs::Hidden)
                .then(|| {
                    color_numeric_inputs(
                        cx,
                        current,
                        model.clone(),
                        draft.clone(),
                        rgb_draft.clone(),
                        hsv_draft.clone(),
                        numeric_error.clone(),
                        popup_options.numeric_inputs,
                        show_alpha,
                        enabled,
                        derived_test_id(popup_test_id.as_ref(), "numbers"),
                    )
                });
            let popup_test_id_for_swatches = popup_test_id.clone();
            let model_for_swatches = model.clone();
            let draft_for_swatches = draft.clone();
            let error_for_swatches = error.clone();
            let swatches = popup_options.presets.then(|| {
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
                        COLOR_PRESETS
                            .iter()
                            .enumerate()
                            .map(|(idx, (name, rgb))| {
                                preset_swatch(
                                    cx,
                                    *name,
                                    *rgb,
                                    current_rgb == *rgb,
                                    current.a,
                                    model_for_swatches.clone(),
                                    draft_for_swatches.clone(),
                                    error_for_swatches.clone(),
                                    open_for_content.clone(),
                                    show_alpha,
                                    enabled,
                                    derived_test_id(
                                        popup_test_id_for_swatches.as_ref(),
                                        format!("preset.{idx}").as_str(),
                                    ),
                                )
                            })
                            .collect::<Vec<_>>()
                    },
                )
            });
            let alpha_bar = if popup_options.shows_alpha_bar(show_alpha) {
                Some(alpha_bar(
                    cx,
                    current,
                    model.clone(),
                    draft.clone(),
                    error.clone(),
                    enabled,
                    derived_test_id(popup_test_id.as_ref(), "alpha"),
                ))
            } else {
                None
            };
            let content = cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Vertical,
                    gap: SpacingLength::Px(Px(8.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |_cx| {
                    let mut out = Vec::new();
                    if let Some(picker) = picker {
                        out.push(picker);
                    }
                    if let Some(numbers) = numbers {
                        out.push(numbers);
                    }
                    if let Some(swatches) = swatches {
                        out.push(swatches);
                    }
                    if let Some(alpha_bar) = alpha_bar {
                        out.push(alpha_bar);
                    }
                    out
                },
            );
            let popup = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(216.0)),
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

            let popup = if let Some(test_id) = popup_test_id.as_ref() {
                popup.test_id(test_id.clone())
            } else {
                popup
            };

            vec![popup]
        },
    );

    let mut request = OverlayRequest::dismissible_menu(
        overlay_id,
        swatch_id,
        open,
        presence,
        vec![cx.pointer_region(
            PointerRegionProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                enabled: true,
                capture_phase_pointer_moves: false,
            },
            move |_cx| vec![popup],
        )],
    );
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);

    OverlayController::request(cx, request);
}

fn color_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    radius: Px,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fill_preview_layout(),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        move |cx| {
            vec![cx.stack_props(
                StackProps {
                    layout: fill_preview_layout(),
                },
                move |cx| {
                    let checkerboard = checkerboard_grid(cx);
                    let overlay = cx.container(
                        ContainerProps {
                            layout: fill_absolute_preview_layout(),
                            background: Some(color),
                            corner_radii: Corners::all(radius),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    );
                    vec![checkerboard, overlay]
                },
            )]
        },
    )
}

fn checkerboard_grid<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: 2,
            rows: Some(2),
            template_columns: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0), GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..4)
                .map(|idx| {
                    let row = idx / 2;
                    let col = idx % 2;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(checkerboard_cell_color(row, col)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn fill_preview_layout() -> LayoutStyle {
    LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Fill,
            ..Default::default()
        },
        overflow: Overflow::Clip,
        ..Default::default()
    }
}

fn fill_absolute_preview_layout() -> LayoutStyle {
    let mut layout = fill_preview_layout();
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
    };
    layout
}

fn checkerboard_cell_color(row: usize, col: usize) -> Color {
    let rgb = if (row + col).is_multiple_of(2) {
        CHECKERBOARD_LIGHT_RGB
    } else {
        CHECKERBOARD_DARK_RGB
    };
    Color::from_srgb_hex_rgb(rgb)
}

fn hsv_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let sv_test_id = derived_test_id(test_id.as_ref(), "sv");
    let hue_test_id = derived_test_id(test_id.as_ref(), "hue");
    let sv = sv_picker(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        sv_test_id,
    );
    let hue = hue_bar(
        cx,
        current,
        model,
        draft,
        error,
        show_alpha,
        enabled,
        hue_test_id,
    );

    let mut picker = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| vec![sv, hue],
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker
}

fn color_numeric_inputs<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    hex_draft: Model<String>,
    rgb_draft: Model<String>,
    hsv_draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    numeric_inputs: ColorEditPopupNumericInputs,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = rgb_numeric_text(current, show_alpha);
    let hsv = hsv_numeric_text(current);
    let error_msg = cx
        .get_model_cloned(&error, Invalidation::Paint)
        .unwrap_or(None);
    let (chrome, text_style, error_color, row_height) = {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);
        let (chrome, text_style) =
            resolve_editor_text_field_style(theme, Size::default(), &ChromeRefinement::default());
        (
            chrome,
            typography::as_control_text(TextStyle {
                size: Px(10.0),
                line_height: Some(density.row_height),
                ..text_style
            }),
            theme.color_token("destructive"),
            density.row_height,
        )
    };
    let rgb_test_id = derived_test_id(test_id.as_ref(), ColorNumericInputMode::Rgb.test_suffix());
    let hsv_test_id = derived_test_id(test_id.as_ref(), ColorNumericInputMode::Hsv.test_suffix());

    let mut inputs = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(2.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            for mode in color_numeric_input_modes(numeric_inputs) {
                let (draft, display_text, test_id) = match *mode {
                    ColorNumericInputMode::Rgb => {
                        (rgb_draft.clone(), rgb.clone(), rgb_test_id.clone())
                    }
                    ColorNumericInputMode::Hsv => {
                        (hsv_draft.clone(), hsv.clone(), hsv_test_id.clone())
                    }
                };
                out.push(color_numeric_input_field(
                    cx,
                    *mode,
                    model.clone(),
                    hex_draft.clone(),
                    draft,
                    error.clone(),
                    display_text,
                    show_alpha,
                    enabled,
                    chrome.clone(),
                    text_style.clone(),
                    error_msg.is_some(),
                    test_id,
                ));
            }
            if let Some(msg) = error_msg.clone() {
                out.push(color_numeric_error_line(cx, msg, error_color, row_height));
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        inputs = inputs.test_id(test_id);
    }
    inputs
}

fn color_numeric_input_field<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mode: ColorNumericInputMode,
    model: Model<Color>,
    hex_draft: Model<String>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    display_text: Arc<str>,
    show_alpha: bool,
    enabled: bool,
    chrome: fret_ui::TextInputStyle,
    text_style: TextStyle,
    has_error: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = TextInputProps::new(draft.clone());
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            min_height: Some(Length::Px(row_height_from_style(&text_style))),
            ..Default::default()
        },
        ..Default::default()
    };
    props.enabled = enabled;
    props.focusable = enabled;
    props.test_id = test_id;
    props.placeholder = Some(color_numeric_placeholder(mode, show_alpha));
    props.a11y_label = Some(mode.a11y_label());
    props.a11y_invalid = has_error.then_some(SemanticsInvalid::True);
    props.chrome = chrome;
    props.text_style = text_style;

    let input = cx.text_input(props);
    let input_id = input.id;
    let is_focused = cx.is_focused_element(input_id);

    if !is_focused {
        let _ = cx
            .app
            .models_mut()
            .update(&draft, |s| *s = display_text.as_ref().to_string());
    }

    let model_for_key = model;
    let hex_draft_for_key = hex_draft;
    let draft_for_key = draft;
    let error_for_key = error;
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
                if let Some(next) = parse_color_numeric_input(mode, &text, show_alpha, current) {
                    let _ = host.models_mut().update(&model_for_key, |c| *c = next);
                    let formatted = format_hex(next, show_alpha);
                    let numeric = color_numeric_text(next, show_alpha, mode);
                    let _ = host
                        .models_mut()
                        .update(&hex_draft_for_key, |s| *s = formatted.as_ref().to_string());
                    let _ = host
                        .models_mut()
                        .update(&draft_for_key, |s| *s = numeric.as_ref().to_string());
                    let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                } else {
                    let message = mode.invalid_message();
                    let _ = host
                        .models_mut()
                        .update(&error_for_key, |e| *e = Some(message));
                }
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::Escape => {
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or(Color::TRANSPARENT);
                let numeric = color_numeric_text(current, show_alpha, mode);
                let _ = host
                    .models_mut()
                    .update(&draft_for_key, |s| *s = numeric.as_ref().to_string());
                let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                host.request_redraw(action_cx.window);
                true
            }
            _ => false,
        }),
    );

    input
}

fn color_numeric_error_line<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    color: Color,
    row_height: Px,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(typography::as_control_text(TextStyle {
            size: Px(10.0),
            line_height: Some(row_height),
            ..Default::default()
        })),
        color: Some(color),
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis,
        align: TextAlign::Start,
        ink_overflow: Default::default(),
    })
}

fn row_height_from_style(style: &TextStyle) -> Px {
    style.line_height.unwrap_or(style.size)
}

fn color_numeric_placeholder(mode: ColorNumericInputMode, show_alpha: bool) -> Arc<str> {
    match (mode, show_alpha) {
        (ColorNumericInputMode::Rgb, true) => Arc::from("RGB 255 255 255 | A 100%"),
        (ColorNumericInputMode::Rgb, false) => Arc::from("RGB 255 255 255"),
        (ColorNumericInputMode::Hsv, _) => Arc::from("HSV 0deg | S 100% | V 100%"),
    }
}

fn sv_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hsv = hsv_from_color(current);
    let value = sv_picker_a11y_text(hsv);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut picker = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(96.0)),
                    min_height: Some(Length::Px(Px(96.0))),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Saturation and value")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_sv_picker_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    show_alpha,
                    down.position_local.x.0,
                    down.position_local.y.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_sv_picker_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    show_alpha,
                    mv.position_local.x.0,
                    mv.position_local.y.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: fill_preview_layout(),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    ..Default::default()
                },
                move |cx| vec![sv_picker_preview_stack(cx, hsv)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker.a11y_value(value)
}

fn sv_picker_preview_stack<H: UiHost>(cx: &mut ElementContext<'_, H>, hsv: HsvColor) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                sv_picker_grid(cx, hsv.hue),
                sv_picker_thumb_overlay(cx, hsv.saturation, hsv.value),
            ]
        },
    )
}

fn sv_picker_grid<H: UiHost>(cx: &mut ElementContext<'_, H>, hue: f32) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: SV_PICKER_STEPS as u16,
            rows: Some(SV_PICKER_STEPS as u16),
            template_columns: Some(
                (0..SV_PICKER_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(
                (0..SV_PICKER_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        move |cx| {
            (0..SV_PICKER_STEPS * SV_PICKER_STEPS)
                .map(|idx| {
                    let row = idx / SV_PICKER_STEPS;
                    let col = idx % SV_PICKER_STEPS;
                    let saturation = unit_from_step(col, SV_PICKER_STEPS);
                    let value = 1.0 - unit_from_step(row, SV_PICKER_STEPS);
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation,
                                    value,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn sv_picker_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    saturation: f32,
    value: f32,
) -> AnyElement {
    let left_grow = saturation.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    let top_grow = (1.0 - value.clamp(0.0, 1.0)).max(0.0);
    let bottom_grow = value.clamp(0.0, 1.0);

    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                sv_thumb_vertical_spacer(cx, top_grow),
                cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(9.0)),
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
                                grow: 0.0,
                                shrink: 0.0,
                                basis: Length::Px(Px(9.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(0.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            horizontal_bar_thumb_spacer(cx, left_grow),
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(9.0)),
                                            height: Length::Px(Px(9.0)),
                                            ..Default::default()
                                        },
                                        flex: FlexItemStyle {
                                            grow: 0.0,
                                            shrink: 0.0,
                                            basis: Length::Px(Px(9.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    background: Some(Color::TRANSPARENT),
                                    border: Edges::all(Px(2.0)),
                                    border_color: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
                                    corner_radii: Corners::all(Px(10.0)),
                                    ..Default::default()
                                },
                                |_cx| vec![],
                            ),
                            horizontal_bar_thumb_spacer(cx, right_grow),
                        ]
                    },
                ),
                sv_thumb_vertical_spacer(cx, bottom_grow),
            ]
        },
    )
}

fn sv_thumb_vertical_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn hue_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hsv = hsv_from_color(current);
    let value = hue_percent_text(hsv.hue);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(18.0)),
                    min_height: Some(Length::Px(Px(18.0))),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Hue")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_hue_bar_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    show_alpha,
                    down.position_local.x.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_hue_bar_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    show_alpha,
                    mv.position_local.x.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(4.0)),
                    padding: Edges::all(Px(1.0)).into(),
                    ..Default::default()
                },
                move |cx| vec![hue_bar_preview_stack(cx, hsv.hue)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}

fn hue_bar_preview_stack<H: UiHost>(cx: &mut ElementContext<'_, H>, hue: f32) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                hue_gradient_overlay(cx),
                horizontal_bar_thumb_overlay(cx, hue),
            ]
        },
    )
}

fn hue_gradient_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: HUE_BAR_STEPS as u16,
            rows: Some(1),
            template_columns: Some(
                (0..HUE_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..HUE_BAR_STEPS)
                .map(|idx| {
                    let hue = idx as f32 / HUE_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(hsv_to_color_preserving_alpha(
                                HsvColor {
                                    hue,
                                    saturation: 1.0,
                                    value: 1.0,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn alpha_bar<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    let alpha = current.a.clamp(0.0, 1.0);
    let value = alpha_percent_text(alpha);

    let model_for_down = model.clone();
    let draft_for_down = draft.clone();
    let error_for_down = error.clone();
    let model_for_move = model;
    let draft_for_move = draft;
    let error_for_move = error;

    let mut bar = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(Px(18.0)),
                    min_height: Some(Length::Px(Px(18.0))),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Slider),
                label: Some(Arc::from("Alpha")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }
                apply_alpha_bar_position(
                    host,
                    action_cx,
                    &model_for_down,
                    &draft_for_down,
                    &error_for_down,
                    down.position_local.x.0,
                );
                host.capture_pointer();
                PressablePointerDownResult::Continue
            }));

            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                if !mv.buttons.left {
                    host.release_pointer_capture();
                    return false;
                }
                apply_alpha_bar_position(
                    host,
                    action_cx,
                    &model_for_move,
                    &draft_for_move,
                    &error_for_move,
                    mv.position_local.x.0,
                );
                true
            }));
            cx.pressable_add_on_pointer_up(Arc::new(move |host, _action_cx, _up| {
                host.release_pointer_capture();
                PressablePointerUpResult::Continue
            }));

            let (border, ring) = {
                let theme = Theme::global(&*cx.app);
                let border = theme
                    .color_by_key("border")
                    .unwrap_or_else(|| theme.color_token("border"));
                let ring = theme
                    .color_by_key("ring")
                    .unwrap_or_else(|| theme.color_token("primary"));
                (border, ring)
            };
            let border_color = if st.focused { ring } else { border };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(4.0)),
                    padding: Edges::all(Px(1.0)).into(),
                    ..Default::default()
                },
                move |cx| vec![alpha_bar_preview_stack(cx, rgb, alpha)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        bar = bar.test_id(test_id);
    }
    bar.a11y_value(value)
}

fn alpha_bar_preview_stack<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    rgb: u32,
    alpha: f32,
) -> AnyElement {
    cx.stack_props(
        StackProps {
            layout: fill_preview_layout(),
        },
        move |cx| {
            vec![
                checkerboard_grid(cx),
                alpha_gradient_overlay(cx, rgb),
                horizontal_bar_thumb_overlay(cx, alpha),
            ]
        },
    )
}

fn alpha_gradient_overlay<H: UiHost>(cx: &mut ElementContext<'_, H>, rgb: u32) -> AnyElement {
    cx.grid(
        GridProps {
            layout: fill_preview_layout(),
            cols: ALPHA_BAR_STEPS as u16,
            rows: Some(1),
            template_columns: Some(
                (0..ALPHA_BAR_STEPS)
                    .map(|_| GridTrackSizing::Flex(1.0))
                    .collect(),
            ),
            template_rows: Some(vec![GridTrackSizing::Flex(1.0)]),
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            ..Default::default()
        },
        |cx| {
            (0..ALPHA_BAR_STEPS)
                .map(|idx| {
                    let alpha = (idx + 1) as f32 / ALPHA_BAR_STEPS as f32;
                    cx.container(
                        ContainerProps {
                            layout: fill_preview_layout(),
                            background: Some(color_from_rgb_preserving_alpha(rgb, alpha)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn horizontal_bar_thumb_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    position: f32,
) -> AnyElement {
    let left_grow = position.clamp(0.0, 1.0);
    let right_grow = (1.0 - left_grow).max(0.0);
    cx.flex(
        FlexProps {
            layout: fill_preview_layout(),
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            vec![
                horizontal_bar_thumb_spacer(cx, left_grow),
                cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(3.0)),
                                height: Length::Fill,
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
                                grow: 0.0,
                                shrink: 0.0,
                                basis: Length::Px(Px(3.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        background: Some(Color::from_srgb_hex_rgb(0xff_ff_ff)),
                        border: Edges::all(Px(1.0)),
                        border_color: Some(Color::from_srgb_hex_rgb(0x1f_29_37)),
                        corner_radii: Corners::all(Px(2.0)),
                        ..Default::default()
                    },
                    |_cx| vec![],
                ),
                horizontal_bar_thumb_spacer(cx, right_grow),
            ]
        },
    )
}

fn horizontal_bar_thumb_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Fill,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn apply_alpha_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    x: f32,
) {
    let width = host.bounds().size.width.0;
    let alpha = alpha_from_local_x(x, width);
    let mut next = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    next.a = alpha;
    let formatted = format_hex(next, true);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
}

fn alpha_from_local_x(x: f32, width: f32) -> f32 {
    if !width.is_finite() || width <= f32::EPSILON {
        return 0.0;
    }
    (x / width).clamp(0.0, 1.0)
}

fn alpha_percent_text(alpha: f32) -> Arc<str> {
    Arc::from(format!(
        "{}%",
        (alpha.clamp(0.0, 1.0) * 100.0).round() as u8
    ))
}

fn preset_swatch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    rgb: u32,
    selected: bool,
    current_alpha: f32,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let color = color_from_rgb_preserving_alpha(rgb, current_alpha);
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let current = host.models_mut().get_copied(&model).unwrap_or(color);
            let color = color_from_rgb_preserving_alpha(rgb, current.a);
            let formatted = format_hex(color, show_alpha);
            let _ = host.models_mut().update(&model, |c| *c = color);
            let _ = host
                .models_mut()
                .update(&draft, |s| *s = formatted.as_ref().to_string());
            let _ = host.models_mut().update(&error, |e| *e = None);
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        });

    let (border_color, ring) = {
        let theme = Theme::global(&*cx.app);
        let ring = theme
            .color_by_key("ring")
            .unwrap_or_else(|| theme.color_token("primary"));
        let border_color = if selected {
            ring
        } else {
            theme
                .color_by_key("border")
                .unwrap_or_else(|| theme.color_token("border"))
        };
        (border_color, ring)
    };

    let mut swatch = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(28.0)),
                    height: Length::Px(Px(28.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Button),
                label: Some(Arc::from(format!("{name} color preset"))),
                ..Default::default()
            },
            focus_ring: Some(fret_ui::element::RingStyle {
                placement: fret_ui::element::RingPlacement::Outset,
                width: Px(2.0),
                offset: Px(1.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(Px(5.0)),
            }),
            ..Default::default()
        },
        move |cx, _st| {
            cx.pressable_add_on_activate(on_activate.clone());
            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(if selected { Px(2.0) } else { Px(1.0) }),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    padding: Edges::all(if selected { Px(2.0) } else { Px(1.0) }).into(),
                    ..Default::default()
                },
                move |cx| vec![color_preview_stack(cx, color, Px(5.0))],
            )]
        },
    );

    if let Some(test_id) = test_id {
        swatch = swatch.test_id(test_id);
    }
    swatch.a11y_value(format_hex(color, show_alpha))
}

fn apply_sv_picker_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    x: f32,
    y: f32,
) {
    let bounds = host.bounds();
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let current_hsv = hsv_from_color(current);
    let next_hsv = hsv_with_sv_from_local_position(
        current_hsv,
        x,
        y,
        bounds.size.width.0,
        bounds.size.height.0,
    );
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}

fn apply_hue_bar_position(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    x: f32,
) {
    let width = host.bounds().size.width.0;
    let current = host
        .models_mut()
        .get_copied(model)
        .unwrap_or(Color::TRANSPARENT);
    let mut next_hsv = hsv_from_color(current);
    next_hsv.hue = hue_from_local_x(x, width);
    apply_hsv_color(
        host, action_cx, model, draft, error, show_alpha, current, next_hsv,
    );
}

fn apply_hsv_color(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    current: Color,
    next_hsv: HsvColor,
) {
    let next = hsv_to_color_preserving_alpha(next_hsv, current.a);
    let formatted = format_hex(next, show_alpha);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
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

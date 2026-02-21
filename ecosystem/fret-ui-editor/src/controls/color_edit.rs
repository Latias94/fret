//! Minimal color edit control (swatch + hex input + stub popup).
//!
//! v1 scope:
//! - hex input for `#RRGGBB` (and optionally `#RRGGBBAA`)
//! - swatch button that can open a popup (picker TBD)

use std::panic::Location;
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Color, Corners, Edges, KeyCode, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, OnCloseAutoFocus};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PointerRegionProps, PressableA11y, PressableProps, SizeStyle, TextInputProps, TextProps,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{ChromeRefinement, OverlayController, OverlayPresence, OverlayRequest, Size};

use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::{EditorDensity, EditorTokenKeys};

#[derive(Debug, Clone)]
pub struct ColorEditOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub show_alpha: bool,
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

        let (density, swatch_size, popup_padding, border, ring) = {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);
            let swatch_size = theme
                .metric_by_key(EditorTokenKeys::COLOR_SWATCH_SIZE)
                .unwrap_or(density.icon_size);
            let popup_padding = theme
                .metric_by_key(EditorTokenKeys::COLOR_POPUP_PADDING)
                .unwrap_or(Px(8.0));
            let border = theme
                .color_by_key("border")
                .or_else(|| theme.color_by_key("component.input.border"))
                .unwrap_or_else(|| theme.color_token("foreground"));
            let ring = theme
                .color_by_key("ring")
                .unwrap_or_else(|| theme.color_token("primary"));
            (density, swatch_size, popup_padding, border, ring)
        };

        let current = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or(Color::TRANSPARENT);
        let current_hex = format_hex(current, self.options.show_alpha);

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
                    min_height: Some(density.row_height),
                    ..Default::default()
                },
                ..Default::default()
            };
            props.enabled = self.options.enabled;
            props.focusable = self.options.focusable;
            props.test_id = self.options.input_test_id.clone();
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
                        if let Some(next) = parse_hex(&text, show_alpha) {
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
                        let current =
                            host.models_mut()
                                .get_copied(&model_for_key)
                                .unwrap_or(Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                });
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

            input
        };

        let swatch = {
            let open_for_activate = open.clone();
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
                    enabled: self.options.enabled,
                    focusable: self.options.focusable,
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
                        corner_radii: Corners::all(Px(6.0)),
                    }),
                    ..Default::default()
                },
                move |cx, _st| {
                    cx.pressable_add_on_activate(on_activate.clone());

                    vec![cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(swatch_size),
                                    height: Length::Px(swatch_size),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            background: Some(current),
                            border: Edges::all(Px(1.0)),
                            border_color: Some(border),
                            corner_radii: Corners::all(Px(4.0)),
                            ..Default::default()
                        },
                        |_cx| vec![],
                    )]
                },
            );

            if let Some(test_id) = self.options.swatch_test_id.as_ref() {
                swatch = swatch.test_id(test_id.clone());
            }
            swatch
        };

        request_popup_overlay(
            cx,
            swatch.id,
            open.clone(),
            popup_padding,
            self.options.popup_test_id.clone(),
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
                style: Some(TextStyle {
                    size: Px(10.0),
                    line_height: Some(density.row_height),
                    ..Default::default()
                }),
                color: Some(Theme::global(&*cx.app).color_token("destructive")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: TextAlign::Start,
                ink_overflow: Default::default(),
            })
        });

        let mut root_layout = self.options.layout;
        if root_layout.size.min_height.is_none() {
            root_layout.size.min_height = Some(density.row_height);
        }

        let mut el = cx.flex(
            FlexProps {
                layout: root_layout,
                direction: Axis::Vertical,
                gap: Px(4.0),
                padding: Edges::all(Px(0.0)),
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
                        gap: Px(8.0),
                        padding: Edges::all(Px(0.0)),
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
    open: Model<bool>,
    popup_padding: Px,
    popup_test_id: Option<Arc<str>>,
) {
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
            let popup = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(Px(220.0)),
                            height: Length::Auto,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(popup_padding),
                    background: Some(Theme::global(&*cx.app).color_token("popover")),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(Theme::global(&*cx.app).color_token("border")),
                    corner_radii: Corners::all(Px(8.0)),
                    ..Default::default()
                },
                move |cx| vec![cx.text("Color picker (stub)")],
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

fn format_hex(color: Color, show_alpha: bool) -> Arc<str> {
    let r = (color.r.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (color.g.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (color.b.clamp(0.0, 1.0) * 255.0).round() as u8;
    let a = (color.a.clamp(0.0, 1.0) * 255.0).round() as u8;
    if show_alpha {
        Arc::from(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))
    } else {
        Arc::from(format!("#{r:02X}{g:02X}{b:02X}"))
    }
}

fn parse_hex(text: &str, show_alpha: bool) -> Option<Color> {
    let s = text.trim().trim_start_matches('#');
    let s = s.trim();

    let want_len = if show_alpha { 8 } else { 6 };
    if s.len() != want_len {
        return None;
    }

    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    let a = if show_alpha {
        u8::from_str_radix(&s[6..8], 16).ok()?
    } else {
        255
    };

    Some(Color {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    })
}

fn popup_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let m = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(false);
            cx.with_state(
                || None::<Model<bool>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}

fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    let m = cx.with_state(|| None::<Model<String>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(String::new());
            cx.with_state(
                || None::<Model<String>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}

fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    let m = cx.with_state(|| None::<Model<Option<Arc<str>>>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(
                || None::<Model<Option<Arc<str>>>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}

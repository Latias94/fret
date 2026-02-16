//! Gradient editor spike (v1).
//!
//! Goal: validate that editor primitives (DragValue, ColorEdit, PropertyGrid) are sufficient to
//! build an editor-grade gradient stop editor without adding new runtime contracts.
//!
//! This is intentionally a policy/composition surface:
//! - callers own stop models and mutations (add/remove/reorder),
//! - the editor crate provides consistent layout, chrome, and a compact preview.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::scene::{ColorSpace, GradientStop, LinearGradient, MAX_STOPS, Paint, TileMode};
use fret_core::{Axis, Color, Corners, Edges, MouseButton, Point, PointerId, Px, Rect};
use fret_runtime::Model;
use fret_ui::action::{
    ActionCx, ActivateReason, OnActivate, PressablePointerDownResult, PressablePointerUpResult,
    UiActionHost,
};
use fret_ui::canvas::OnCanvasPaint;
use fret_ui::element::{
    AnyElement, CanvasProps, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle,
    Length, MainAlign, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::ColorRef;

use super::property_row::PropertyRowOptions;
use super::{PropertyGrid, PropertyGroup, PropertyRow};
use crate::controls::{ColorEdit, ColorEditOptions, DragValue, NumericFormatFn, NumericParseFn};
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};
use crate::primitives::{EditorDensity, percent_0_1_format, percent_0_1_parse};

pub type OnGradientStopAction =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, fret_ui::ItemKey) + 'static>;

pub type OnGradientAction = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct GradientEditorOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub preview_height: Px,
    pub show_angle: bool,
    pub enable_preview_drag: bool,
    pub a11y_label: Option<Arc<str>>,
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub preview_test_id: Option<Arc<str>>,
    pub stops_test_id: Option<Arc<str>>,
    pub add_stop_test_id: Option<Arc<str>>,
}

impl Default for GradientEditorOptions {
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
            preview_height: Px(22.0),
            show_angle: true,
            enable_preview_drag: true,
            a11y_label: None,
            id_source: None,
            test_id: None,
            preview_test_id: None,
            stops_test_id: None,
            add_stop_test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct GradientStopBinding {
    pub id: fret_ui::ItemKey,
    pub position: Model<f64>,
    pub color: Model<Color>,
    pub remove: Option<OnGradientStopAction>,
}

#[derive(Clone)]
pub struct GradientEditor {
    pub angle_degrees: Option<Model<f64>>,
    pub stops: Arc<[GradientStopBinding]>,
    pub on_add_stop: Option<OnGradientAction>,
    pub options: GradientEditorOptions,
}

impl GradientEditor {
    pub fn new(stops: Arc<[GradientStopBinding]>) -> Self {
        Self {
            angle_degrees: None,
            stops,
            on_add_stop: None,
            options: GradientEditorOptions::default(),
        }
    }

    pub fn angle_degrees(mut self, angle: Option<Model<f64>>) -> Self {
        self.angle_degrees = angle;
        self
    }

    pub fn on_add_stop(mut self, on_add: Option<OnGradientAction>) -> Self {
        self.on_add_stop = on_add;
        self
    }

    pub fn options(mut self, options: GradientEditorOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.gradient_editor", id_source), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.gradient_editor", callsite), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let GradientEditor {
            angle_degrees,
            stops,
            on_add_stop,
            options,
        } = self;

        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let state_id = cx.named("gradient_editor.preview_state", |cx| cx.root_id());
        let preview_state: Arc<Mutex<GradientPreviewState>> = cx.with_state_for(
            state_id,
            || Arc::new(Mutex::new(GradientPreviewState::default())),
            |s| s.clone(),
        );

        let angle = angle_degrees
            .as_ref()
            .and_then(|m| cx.get_model_copied(m, Invalidation::Paint))
            .unwrap_or(0.0);

        let mut preview_stops: Vec<PreviewStop> = Vec::new();
        let mut stop_rows: Vec<(f64, GradientStopBinding)> = Vec::new();
        for stop in stops.iter() {
            let pos = cx
                .get_model_copied(&stop.position, Invalidation::Paint)
                .unwrap_or(0.0);
            let color = cx
                .get_model_copied(&stop.color, Invalidation::Paint)
                .unwrap_or(Color::TRANSPARENT);
            preview_stops.push(PreviewStop {
                id: stop.id,
                position: (pos as f32).clamp(0.0, 1.0),
                color,
            });
            stop_rows.push((pos, stop.clone()));
        }
        preview_stops.sort_by(|a, b| a.position.total_cmp(&b.position));
        stop_rows.sort_by(|a, b| a.0.total_cmp(&b.0));

        let preview_h = Px(options.preview_height.0.max(1.0));
        let active_stop = preview_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .active_stop;
        let mut preview = gradient_preview_canvas(
            cx,
            options.enabled && options.enable_preview_drag,
            angle,
            preview_stops.clone(),
            preview_h,
            active_stop,
            preview_state.clone(),
            stops
                .iter()
                .map(|s| (s.id, s.position.clone()))
                .collect::<Vec<_>>()
                .into(),
        );
        if let Some(test_id) = options.preview_test_id.as_ref() {
            preview = preview.test_id(test_id.clone());
        }

        let angle_row = (options.show_angle)
            .then_some(angle_degrees.clone())
            .flatten()
            .map(|m| {
                let fmt: NumericFormatFn<f64> = Arc::new(|v| Arc::from(format!("{v:.0}°")));
                let parse: NumericParseFn<f64> = Arc::new(|s| {
                    let s = s.trim().trim_end_matches('°').trim();
                    s.parse::<f64>().ok()
                });
                PropertyRow::new().into_element(
                    cx,
                    |cx| cx.text("Angle"),
                    |cx| DragValue::new(m, fmt, parse).into_element(cx),
                    |_cx| None,
                )
            });

        let add_stop_test_id = options.add_stop_test_id.clone();
        let enabled = options.enabled;
        let can_add_stop = enabled && (stops.len() < MAX_STOPS);

        let mut stops_group = PropertyGroup::new("Stops").into_element(
            cx,
            move |cx| {
                let on_add_stop = on_add_stop.clone()?;
                Some(add_stop_button(
                    cx,
                    density,
                    can_add_stop,
                    on_add_stop,
                    add_stop_test_id.clone(),
                ))
            },
            move |cx| {
                vec![PropertyGrid::new().into_element(cx, move |cx, row_cx| {
                    let mut rows = Vec::new();
                    for (_pos, stop) in stop_rows.iter().cloned() {
                        rows.push(cx.keyed(("gradient_stop_row", stop.id), |cx| {
                            stop_row(cx, density, enabled, stop, row_cx.row_options.clone())
                        }));
                    }
                    if rows.is_empty() {
                        rows.push(cx.text("No stops"));
                    }
                    rows
                })]
            },
        );

        if let Some(test_id) = options.stops_test_id.as_ref() {
            stops_group = stops_group.test_id(test_id.clone());
        }

        let mut content = Vec::new();
        content.push(preview);
        if let Some(angle_row) = angle_row {
            content.push(angle_row);
        }
        content.push(stops_group);

        let mut root = cx.flex(
            FlexProps {
                layout: options.layout,
                direction: Axis::Vertical,
                gap: Px(8.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| content,
        );

        if let Some(test_id) = options.test_id.as_ref() {
            root = root.test_id(test_id.clone());
        }
        root
    }
}

fn stop_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled: bool,
    stop: GradientStopBinding,
    row_options: PropertyRowOptions,
) -> AnyElement {
    let fmt = percent_0_1_format(0);
    let parse = percent_0_1_parse();

    let remove = stop.remove.clone();
    let stop_id = stop.id;

    PropertyRow::new().options(row_options).into_element(
        cx,
        |cx| cx.text("Stop"),
        move |cx| {
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
                    gap: Px(6.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let pos = DragValue::new(stop.position.clone(), fmt.clone(), parse.clone())
                        .into_element(cx);
                    let color = ColorEdit::new(stop.color.clone())
                        .options(ColorEditOptions {
                            swatch_test_id: None,
                            input_test_id: None,
                            popup_test_id: None,
                            ..Default::default()
                        })
                        .into_element(cx);
                    vec![pos, color]
                },
            )
        },
        move |cx| {
            let remove = remove.clone()?;
            Some(remove_button(cx, density, enabled, stop_id, remove))
        },
    )
}

fn remove_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled: bool,
    stop_id: fret_ui::ItemKey,
    on_remove: OnGradientStopAction,
) -> AnyElement {
    cx.pressable(
        PressableProps {
            enabled,
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(density.hit_thickness),
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Auto,
                    align_self: None,
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(Arc::from("Remove stop")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let on_activate: OnActivate = Arc::new({
                let on_remove = on_remove.clone();
                move |host, action_cx: ActionCx, _reason: ActivateReason| {
                    on_remove(host, action_cx, stop_id);
                    host.request_redraw(action_cx.window);
                }
            });
            cx.pressable_add_on_activate(on_activate);

            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;

            let (bg, border, icon_fg) = {
                let theme = Theme::global(&*cx.app);
                (
                    editor_icon_button_bg(theme, enabled, hovered, pressed),
                    editor_icon_button_border(theme, enabled, hovered, pressed),
                    theme.color_token("muted-foreground"),
                )
            };
            let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: bg,
                    border: Edges::all(border_width),
                    border_color: border,
                    corner_radii: Corners::all(Px(6.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            vec![fret_ui_kit::declarative::icon::icon_with(
                                cx,
                                fret_icons::ids::ui::CLOSE,
                                Some(Px(12.0)),
                                Some(ColorRef::Color(icon_fg)),
                            )]
                        },
                    )]
                },
            )]
        },
    )
}

fn add_stop_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    density: EditorDensity,
    enabled: bool,
    on_add: OnGradientAction,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut el = cx.pressable(
        PressableProps {
            enabled,
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(density.hit_thickness),
                    height: Length::Px(density.row_height),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Auto,
                    align_self: None,
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(Arc::from("Add stop")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let on_activate: OnActivate = Arc::new({
                let on_add = on_add.clone();
                move |host, action_cx: ActionCx, _reason: ActivateReason| {
                    on_add(host, action_cx);
                    host.request_redraw(action_cx.window);
                }
            });
            cx.pressable_add_on_activate(on_activate);

            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;

            let (bg, border, icon_fg) = {
                let theme = Theme::global(&*cx.app);
                (
                    editor_icon_button_bg(theme, enabled, hovered, pressed),
                    editor_icon_button_border(theme, enabled, hovered, pressed),
                    theme.color_token("muted-foreground"),
                )
            };
            let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: bg,
                    border: Edges::all(border_width),
                    border_color: border,
                    corner_radii: Corners::all(Px(6.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.flex(
                        FlexProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            vec![fret_ui_kit::declarative::icon::icon_with(
                                cx,
                                fret_icons::ids::ui::PLUS,
                                Some(Px(12.0)),
                                Some(ColorRef::Color(icon_fg)),
                            )]
                        },
                    )]
                },
            )]
        },
    );

    if let Some(test_id) = test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }
    el
}

#[derive(Debug, Clone, Copy)]
struct PreviewStop {
    id: fret_ui::ItemKey,
    position: f32,
    color: Color,
}

#[derive(Debug, Clone, Copy, Default)]
struct GradientPreviewState {
    dragging: bool,
    pointer_id: Option<PointerId>,
    active_stop: Option<fret_ui::ItemKey>,
}

fn gradient_preview_canvas<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    angle_deg: f64,
    stops: Vec<PreviewStop>,
    height: Px,
    active_stop: Option<fret_ui::ItemKey>,
    preview_state: Arc<Mutex<GradientPreviewState>>,
    stop_models: Arc<[(fret_ui::ItemKey, Model<f64>)]>,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Px(height);

    let state_for_down = preview_state.clone();
    let state_for_move = preview_state.clone();
    let state_for_paint = preview_state.clone();

    let el = cx.pressable(
        PressableProps {
            enabled,
            layout,
            a11y: PressableA11y {
                label: Some(Arc::from("Gradient preview")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, _st| {
            let stops_for_hit = stops.clone();
            let stop_models_for_hit = stop_models.clone();

            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if !enabled {
                    return PressablePointerDownResult::Continue;
                }
                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }

                let bounds = host.bounds();
                let width = bounds.size.width.0.max(1.0) as f32;
                let x = down.position_local.x.0 as f32;
                let x = x.clamp(0.0, width);

                let mut best: Option<(f32, fret_ui::ItemKey)> = None;
                for s in stops_for_hit.iter() {
                    let sx = s.position.clamp(0.0, 1.0) * width;
                    let d = (sx - x).abs();
                    if best.is_none() || d < best.unwrap().0 {
                        best = Some((d, s.id));
                    }
                }

                let Some((dist, stop_id)) = best else {
                    return PressablePointerDownResult::Continue;
                };

                // Keep the hit target forgiving; preview is a compact strip.
                if dist > 12.0 {
                    return PressablePointerDownResult::Continue;
                }

                let t = (x / width).clamp(0.0, 1.0) as f64;
                if let Some((_id, model)) =
                    stop_models_for_hit.iter().find(|(id, _)| *id == stop_id)
                {
                    let _ = host.models_mut().update(model, |v| *v = t);
                }

                let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                st.dragging = true;
                st.pointer_id = Some(down.pointer_id);
                st.active_stop = Some(stop_id);

                host.request_redraw(action_cx.window);
                PressablePointerDownResult::Continue
            }));

            let stops_for_drag = stop_models.clone();
            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                let mut st = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
                if !st.dragging || st.pointer_id != Some(mv.pointer_id) {
                    return false;
                }

                if !mv.buttons.left {
                    st.dragging = false;
                    st.pointer_id = None;
                    return false;
                }

                let Some(stop_id) = st.active_stop else {
                    return false;
                };

                let bounds = host.bounds();
                let width = bounds.size.width.0.max(1.0) as f64;
                let x = (mv.position_local.x.0 as f64).clamp(0.0, width);
                let t = (x / width).clamp(0.0, 1.0);

                if let Some((_id, model)) = stops_for_drag.iter().find(|(id, _)| *id == stop_id) {
                    let _ = host.models_mut().update(model, |v| *v = t);
                    host.request_redraw(action_cx.window);
                    return true;
                }
                false
            }));

            let state_for_up = preview_state.clone();
            cx.pressable_add_on_pointer_up(Arc::new(move |_host, _action_cx, up| {
                let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
                if st.pointer_id == Some(up.pointer_id) {
                    st.dragging = false;
                    st.pointer_id = None;
                }
                PressablePointerUpResult::Continue
            }));

            let on_paint: OnCanvasPaint = Arc::new(move |p| {
                let bounds = p.bounds();
                let rect = Rect {
                    origin: bounds.origin,
                    size: bounds.size,
                };

                let muted = p.theme().color_token("muted");
                let border = p.theme().color_token("border");
                let accent = p.theme().color_token("accent");

                let angle = (angle_deg as f32).to_radians();
                let dx = angle.cos();
                let dy = angle.sin();

                let len = (rect.size.width.0.powi(2) + rect.size.height.0.powi(2))
                    .sqrt()
                    .max(1.0);
                let half = len * 0.5;
                let cx0 = rect.origin.x.0 + rect.size.width.0 * 0.5;
                let cy0 = rect.origin.y.0 + rect.size.height.0 * 0.5;
                let start = Point::new(Px(cx0 - dx * half), Px(cy0 - dy * half));
                let end = Point::new(Px(cx0 + dx * half), Px(cy0 + dy * half));

                let mut fixed = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
                let mut n: u8 = 0;
                for (i, s) in stops.iter().take(MAX_STOPS).enumerate() {
                    fixed[i] = GradientStop::new(s.position.clamp(0.0, 1.0), s.color);
                    n = (i as u8) + 1;
                }
                if n == 0 {
                    fixed[0] = GradientStop::new(0.0, muted);
                    fixed[1] = GradientStop::new(1.0, muted);
                    n = 2;
                }

                let gradient = LinearGradient {
                    start,
                    end,
                    tile_mode: TileMode::Clamp,
                    color_space: ColorSpace::Srgb,
                    stop_count: n,
                    stops: fixed,
                };

                p.scene().push(fret_core::SceneOp::Quad {
                    order: fret_core::DrawOrder(0),
                    rect,
                    background: Paint::LinearGradient(gradient),
                    border: Edges::all(Px(1.0)),
                    border_paint: Paint::Solid(border),
                    corner_radii: Corners::all(Px(6.0)),
                });

                let w = rect.size.width.0.max(1.0);
                let h = rect.size.height.0.max(1.0);

                let marker_d = (h * 0.55).min(12.0).max(6.0);
                let marker_y = rect.origin.y.0 + h - marker_d * 0.5 - 1.0;
                let marker_radius = Px(marker_d * 0.5);

                let active = state_for_paint
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .active_stop
                    .or(active_stop);

                for s in stops.iter() {
                    let x = rect.origin.x.0 + w * s.position.clamp(0.0, 1.0);
                    let marker_rect = Rect {
                        origin: Point::new(Px(x - marker_d * 0.5), Px(marker_y - marker_d * 0.5)),
                        size: fret_core::Size::new(Px(marker_d), Px(marker_d)),
                    };

                    let outline = if Some(s.id) == active {
                        Paint::Solid(accent)
                    } else {
                        Paint::Solid(border)
                    };
                    let stroke_w = if Some(s.id) == active {
                        Px(2.0)
                    } else {
                        Px(1.0)
                    };

                    p.scene().push(fret_core::SceneOp::Quad {
                        order: fret_core::DrawOrder(1),
                        rect: marker_rect,
                        background: Paint::Solid(s.color),
                        border: Edges::all(stroke_w),
                        border_paint: outline,
                        corner_radii: Corners::all(marker_radius),
                    });
                }
            });

            let mut props = CanvasProps::default();
            props.layout = LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            };

            vec![cx.canvas(props, move |p| on_paint(p))]
        },
    );
    el
}

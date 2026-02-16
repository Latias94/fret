//! Gradient editor spike (v1).
//!
//! Goal: validate that editor primitives (DragValue, ColorEdit, PropertyGrid) are sufficient to
//! build an editor-grade gradient stop editor without adding new runtime contracts.
//!
//! This is intentionally a policy/composition surface:
//! - callers own stop models and mutations (add/remove/reorder),
//! - the editor crate provides consistent layout, chrome, and a compact preview.

use std::panic::Location;
use std::sync::Arc;

use fret_core::scene::{ColorSpace, GradientStop, LinearGradient, MAX_STOPS, Paint, TileMode};
use fret_core::{Axis, Color, Corners, Edges, Point, Px, Rect};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
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

#[derive(Debug, Clone)]
pub struct GradientEditorOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub preview_height: Px,
    pub show_angle: bool,
    pub a11y_label: Option<Arc<str>>,
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub preview_test_id: Option<Arc<str>>,
    pub stops_test_id: Option<Arc<str>>,
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
            a11y_label: None,
            id_source: None,
            test_id: None,
            preview_test_id: None,
            stops_test_id: None,
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
    pub options: GradientEditorOptions,
}

impl GradientEditor {
    pub fn new(stops: Arc<[GradientStopBinding]>) -> Self {
        Self {
            angle_degrees: None,
            stops,
            options: GradientEditorOptions::default(),
        }
    }

    pub fn angle_degrees(mut self, angle: Option<Model<f64>>) -> Self {
        self.angle_degrees = angle;
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
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);

        let angle = self
            .angle_degrees
            .as_ref()
            .and_then(|m| cx.get_model_copied(m, Invalidation::Paint))
            .unwrap_or(0.0);

        let mut stop_values: Vec<(f32, Color)> = Vec::new();
        for stop in self.stops.iter() {
            let pos = cx
                .get_model_copied(&stop.position, Invalidation::Paint)
                .unwrap_or(0.0);
            let color = cx
                .get_model_copied(&stop.color, Invalidation::Paint)
                .unwrap_or(Color::TRANSPARENT);
            stop_values.push((pos as f32, color));
        }
        stop_values.sort_by(|a, b| a.0.total_cmp(&b.0));

        let preview_h = Px(self.options.preview_height.0.max(1.0));
        let mut preview = gradient_preview_canvas(cx, angle, stop_values.clone(), preview_h);
        if let Some(test_id) = self.options.preview_test_id.as_ref() {
            preview = preview.test_id(test_id.clone());
        }

        let angle_row = (self.options.show_angle)
            .then_some(self.angle_degrees.clone())
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

        let mut stops_group = PropertyGroup::new("Stops").into_element(
            cx,
            |_cx| None,
            move |cx| {
                vec![PropertyGrid::new().into_element(cx, move |cx, row_cx| {
                    let mut rows = Vec::new();
                    for stop in self.stops.iter().cloned() {
                        rows.push(cx.keyed(("gradient_stop_row", stop.id), |cx| {
                            stop_row(
                                cx,
                                density,
                                self.options.enabled,
                                stop,
                                row_cx.row_options.clone(),
                            )
                        }));
                    }
                    if rows.is_empty() {
                        rows.push(cx.text("No stops"));
                    }
                    rows
                })]
            },
        );

        if let Some(test_id) = self.options.stops_test_id.as_ref() {
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
                layout: self.options.layout,
                direction: Axis::Vertical,
                gap: Px(8.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Stretch,
                wrap: false,
            },
            move |_cx| content,
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
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

fn gradient_preview_canvas<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    angle_deg: f64,
    stops: Vec<(f32, Color)>,
    height: Px,
) -> AnyElement {
    let mut props = CanvasProps::default();
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Px(height),
            ..Default::default()
        },
        ..Default::default()
    };

    let on_paint: OnCanvasPaint = Arc::new(move |p| {
        let bounds = p.bounds();
        let rect = Rect {
            origin: bounds.origin,
            size: bounds.size,
        };

        let muted = p.theme().color_token("muted");
        let border = p.theme().color_token("border");

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
        for (i, (off, c)) in stops.iter().take(MAX_STOPS).enumerate() {
            fixed[i] = GradientStop::new(off.clamp(0.0, 1.0), *c);
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
    });

    let el = cx.canvas(props, move |p| on_paint(p));

    // For now, keep the preview non-interactive; stop manipulation happens in the list.
    el
}

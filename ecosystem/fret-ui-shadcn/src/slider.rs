use std::sync::Arc;

use fret_core::{Color, Corners, CursorIcon, Edges, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PointerRegionProps, PositionStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::slider as radix_slider;

#[derive(Debug, Clone)]
pub struct SliderStyle {
    pub track_height: Px,
    pub track_background: Color,
    pub track_border: Edges,
    pub track_border_color: Color,
    pub range_background: Color,
    pub thumb_size: Px,
    pub thumb_background: Color,
    pub thumb_border: Edges,
    pub thumb_border_color: Color,
    pub focus_ring: Option<fret_ui::element::RingStyle>,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_height: Px(4.0),
            track_background: Color {
                r: 0.2,
                g: 0.2,
                b: 0.25,
                a: 1.0,
            },
            track_border: Edges::all(Px(0.0)),
            track_border_color: Color::TRANSPARENT,
            range_background: Color {
                r: 0.45,
                g: 0.7,
                b: 1.0,
                a: 1.0,
            },
            thumb_size: Px(16.0),
            thumb_background: Color {
                r: 0.12,
                g: 0.12,
                b: 0.16,
                a: 1.0,
            },
            thumb_border: Edges::all(Px(1.0)),
            thumb_border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.35,
            },
            focus_ring: None,
        }
    }
}

impl SliderStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        let snapshot = theme.snapshot();
        Self {
            track_height: theme
                .metric_by_key("component.slider.track_height")
                .unwrap_or(Px(4.0)),
            track_background: theme
                .color_by_key("muted")
                .unwrap_or(snapshot.colors.panel_background),
            track_border: Edges::all(Px(0.0)),
            track_border_color: Color::TRANSPARENT,
            range_background: theme
                .color_by_key("primary")
                .or_else(|| theme.color_by_key("accent"))
                .unwrap_or(snapshot.colors.accent),
            thumb_size: theme
                .metric_by_key("component.slider.thumb_size")
                .unwrap_or(Px(16.0)),
            thumb_background: theme
                .color_by_key("background")
                .unwrap_or(snapshot.colors.surface_background),
            thumb_border: Edges::all(Px(1.0)),
            thumb_border_color: theme
                .color_by_key("input")
                .or_else(|| theme.color_by_key("border"))
                .unwrap_or(snapshot.colors.panel_border),
            focus_ring: None,
        }
    }
}

#[derive(Clone)]
pub struct Slider {
    model: Model<Vec<f32>>,
    min: f32,
    max: f32,
    step: f32,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
    style: Option<SliderStyle>,
}

impl Slider {
    pub fn new(model: Model<Vec<f32>>) -> Self {
        Self {
            model,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            disabled: false,
            a11y_label: None,
            layout: LayoutRefinement::default(),
            style: None,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        slider(
            cx,
            self.model,
            self.min,
            self.max,
            self.step,
            self.disabled,
            self.a11y_label,
            self.layout,
            self.style,
        )
    }
}

pub fn slider<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<f32>>,
    min: f32,
    max: f32,
    step: f32,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    layout: LayoutRefinement,
    style: Option<SliderStyle>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let style = style.unwrap_or_else(|| {
        let mut style = SliderStyle::from_theme(&theme);
        let radius = Px((style.thumb_size.0 * 0.5).max(0.0));
        style.focus_ring = Some(decl_style::focus_ring(&theme, radius));
        style
    });

    cx.scope(|cx| {
        let root_layout = decl_style::layout_style(&theme, layout.relative().w_full());
        let root_h = style.thumb_size.0.max(style.track_height.0).max(0.0);

        let mut semantics_layout = root_layout;
        semantics_layout.size.height = Length::Px(Px(root_h));

        let value = cx
            .watch_model(&model)
            .read_ref(|values| values.first().copied())
            .ok()
            .flatten()
            .unwrap_or(min);
        let t = radix_slider::normalize_value(value, min, max);
        let mut semantics =
            radix_slider::slider_root_semantics(a11y_label.clone(), value, disabled);
        semantics.layout = semantics_layout;

        let min_value = min;
        let max_value = max;
        let step_value = step;
        let thumb_size = style.thumb_size;
        let model_on_down = model.clone();
        let model_on_move = model.clone();

        cx.semantics_with_id(semantics, |cx, semantics_id| {
            let on_down = Arc::new(
                move |host: &mut dyn UiPointerActionHost, cx: ActionCx, down: PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }

                    host.request_focus(semantics_id);
                    host.set_cursor_icon(CursorIcon::Pointer);
                    host.capture_pointer();

                    let bounds = host.bounds();
                    radix_slider::update_single_slider_model_from_pointer_x(
                        host,
                        &model_on_down,
                        bounds,
                        down.position.x,
                        min_value,
                        max_value,
                        step_value,
                        thumb_size,
                    );
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_move = Arc::new(
                move |host: &mut dyn UiPointerActionHost, cx: ActionCx, mv: PointerMoveCx| {
                    host.set_cursor_icon(CursorIcon::Pointer);
                    if !mv.buttons.left {
                        return false;
                    }

                    let bounds = host.bounds();
                    radix_slider::update_single_slider_model_from_pointer_x(
                        host,
                        &model_on_move,
                        bounds,
                        mv.position.x,
                        min_value,
                        max_value,
                        step_value,
                        thumb_size,
                    );
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_up = Arc::new(
                move |host: &mut dyn UiPointerActionHost, cx: ActionCx, up: PointerUpCx| {
                    if up.button != MouseButton::Left {
                        return false;
                    }
                    host.release_pointer_capture();
                    host.request_redraw(cx.window);
                    true
                },
            );

            let model_on_key = model.clone();
            cx.key_on_key_down_for(
                semantics_id,
                Arc::new(move |host, cx, down| {
                    if down.repeat
                        || down.modifiers.alt
                        || down.modifiers.ctrl
                        || down.modifiers.meta
                    {
                        return false;
                    }

                    let step = if step_value.is_finite() && step_value > 0.0 {
                        step_value
                    } else {
                        1.0
                    };

                    let cur = host
                        .models_mut()
                        .read(&model_on_key, |v| v.first().copied())
                        .ok()
                        .flatten()
                        .unwrap_or(min_value);

                    let next = match down.key {
                        fret_core::KeyCode::ArrowLeft | fret_core::KeyCode::ArrowDown => cur - step,
                        fret_core::KeyCode::ArrowRight | fret_core::KeyCode::ArrowUp => cur + step,
                        fret_core::KeyCode::Home => min_value,
                        fret_core::KeyCode::End => max_value,
                        _ => return false,
                    };

                    let v = radix_slider::snap_value(next, min_value, max_value, step);
                    let mut values = host
                        .models_mut()
                        .get_cloned(&model_on_key)
                        .unwrap_or_default();
                    if values.is_empty() {
                        values.push(min_value);
                    }
                    values[0] = v;
                    let _ = host.models_mut().update(&model_on_key, |dst| *dst = values);
                    host.request_redraw(cx.window);
                    true
                }),
            );

            let track_top = (root_h - style.track_height.0.max(0.0)) * 0.5;
            let thumb_top = (root_h - style.thumb_size.0.max(0.0)) * 0.5;
            let thumb_r = Px(style.thumb_size.0.max(0.0) * 0.5);

            let root_container = ContainerProps {
                layout: semantics_layout,
                padding: Edges::all(Px(0.0)),
                background: None,
                shadow: None,
                border: Edges::all(Px(0.0)),
                border_color: None,
                corner_radii: Corners::all(Px(0.0)),
            };

            let track = ContainerProps {
                layout: LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: fret_ui::element::InsetStyle {
                        left: Some(thumb_r),
                        right: Some(thumb_r),
                        top: Some(Px(track_top)),
                        ..Default::default()
                    },
                    size: fret_ui::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(style.track_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges::all(Px(0.0)),
                background: Some(style.track_background),
                shadow: None,
                border: style.track_border,
                border_color: Some(style.track_border_color),
                corner_radii: Corners::all(Px(style.track_height.0.max(0.0) * 0.5)),
            };

            let pointer = PointerRegionProps {
                layout: semantics_layout,
                enabled: !disabled,
            };

            vec![cx.pointer_region(pointer, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_move(on_move);
                cx.pointer_region_on_pointer_up(on_up);

                let track_w = cx
                    .last_bounds_for_element(cx.root_id())
                    .map(|b| (b.size.width.0 - style.thumb_size.0.max(0.0)).max(0.0))
                    .unwrap_or(0.0);
                let fill_w = track_w * t;
                let thumb_left = track_w * t;

                let range = ContainerProps {
                    layout: LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: fret_ui::element::InsetStyle {
                            left: Some(thumb_r),
                            top: Some(Px(track_top)),
                            ..Default::default()
                        },
                        size: fret_ui::element::SizeStyle {
                            width: Length::Px(Px(fill_w)),
                            height: Length::Px(style.track_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(Px(0.0)),
                    background: Some(style.range_background),
                    shadow: None,
                    border: Edges::all(Px(0.0)),
                    border_color: None,
                    corner_radii: Corners::all(Px(style.track_height.0.max(0.0) * 0.5)),
                };

                let thumb = ContainerProps {
                    layout: LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: fret_ui::element::InsetStyle {
                            left: Some(Px(thumb_left)),
                            top: Some(Px(thumb_top)),
                            ..Default::default()
                        },
                        size: fret_ui::element::SizeStyle {
                            width: Length::Px(style.thumb_size),
                            height: Length::Px(style.thumb_size),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(Px(0.0)),
                    background: Some(style.thumb_background),
                    shadow: None,
                    border: style.thumb_border,
                    border_color: Some(style.thumb_border_color),
                    corner_radii: Corners::all(thumb_r),
                };

                vec![cx.container(root_container, |cx| {
                    vec![
                        cx.container(track, |_| Vec::new()),
                        cx.container(range, |_| Vec::new()),
                        cx.container(thumb, |_| Vec::new()),
                    ]
                })]
            })]
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle,
        Point, Rect, Scene, Size as CoreSize, SvgId, SvgService, TextBlobId, TextConstraints,
        TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_ui::tree::UiTree;

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: CoreSize::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn slider_updates_model_on_pointer_down() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(60.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(vec![0.0]);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-slider-updates-model-on-pointer-down",
            |cx| {
                vec![
                    Slider::new(model.clone())
                        .range(0.0, 100.0)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let slider_node = ui.children(root)[0];
        let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
        let position = Point::new(
            Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 - 1.0),
            Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let v = app
            .models()
            .get_cloned(&model)
            .and_then(|values| values.first().copied())
            .unwrap_or(f32::NAN);
        assert!((v - 100.0).abs() < 0.01, "expected slider=100, got {v}");

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert!(!scene.ops().is_empty());
    }
}

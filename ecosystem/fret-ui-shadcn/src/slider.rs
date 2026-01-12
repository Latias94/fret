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
    min_steps_between_thumbs: u32,
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
            min_steps_between_thumbs: 0,
            disabled: false,
            a11y_label: None,
            layout: LayoutRefinement::default(),
            style: None,
        }
    }

    /// Creates a slider with a controlled/uncontrolled values model (Radix `value` / `defaultValue`).
    ///
    /// Note: If `value` is `None`, the internal model is stored in element state at the call site.
    /// Call this from a stable subtree (key the parent node if needed).
    pub fn new_controllable<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        value: Option<Model<Vec<f32>>>,
        default_value: impl FnOnce() -> Vec<f32>,
    ) -> Self {
        let model = radix_slider::slider_use_values_model(cx, value, default_value).model();
        Self::new(model)
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

    /// Minimum number of steps between thumbs (Radix `minStepsBetweenThumbs`).
    pub fn min_steps_between_thumbs(mut self, min_steps_between_thumbs: u32) -> Self {
        self.min_steps_between_thumbs = min_steps_between_thumbs;
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
            self.min_steps_between_thumbs,
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
    min_steps_between_thumbs: u32,
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
        #[derive(Default)]
        struct DragIndexState {
            model: Option<Model<usize>>,
        }

        let drag_index_model = cx.with_state(DragIndexState::default, |st| st.model.clone());
        let drag_index_model = if let Some(drag_index_model) = drag_index_model {
            drag_index_model
        } else {
            let drag_index_model = cx.app.models_mut().insert(0usize);
            cx.with_state(DragIndexState::default, |st| {
                st.model = Some(drag_index_model.clone());
            });
            drag_index_model
        };

        let root_layout = decl_style::layout_style(&theme, layout.relative().w_full());
        let root_h = style.thumb_size.0.max(style.track_height.0).max(0.0);

        let mut semantics_layout = root_layout;
        semantics_layout.size.height = Length::Px(Px(root_h));

        let values = cx
            .watch_model(&model)
            .read_ref(|values| values.clone())
            .ok()
            .unwrap_or_else(|| vec![min]);
        let mut values_sorted = values.clone();
        if values_sorted.is_empty() {
            values_sorted.push(min);
        }
        values_sorted.sort_by(|a, b| a.total_cmp(b));
        if values_sorted.len() > 1 && values_sorted != values {
            let values_sorted = values_sorted.clone();
            let _ = cx
                .app
                .models_mut()
                .update(&model, |values| *values = values_sorted);
        }

        let active_index = cx.app.models().get_cloned(&drag_index_model).unwrap_or(0);
        let active_index = active_index.min(values_sorted.len().saturating_sub(1));
        let active_value = values_sorted.get(active_index).copied().unwrap_or(min);

        let percentages: Vec<f32> = values_sorted
            .iter()
            .copied()
            .map(|value| radix_slider::normalize_value(value, min, max))
            .collect();
        let values_count = percentages.len();
        let range_start_t = if values_count > 1 {
            percentages.iter().copied().reduce(f32::min).unwrap_or(0.0)
        } else {
            0.0
        };
        let range_end_t = percentages.iter().copied().reduce(f32::max).unwrap_or(0.0);

        let mut semantics =
            radix_slider::slider_root_semantics(a11y_label.clone(), active_value, disabled);
        semantics.layout = semantics_layout;

        let min_value = min;
        let max_value = max;
        let step_value = step;
        let thumb_size = style.thumb_size;
        let min_steps_between_thumbs_value = min_steps_between_thumbs;
        let model_on_down = model.clone();
        let model_on_move = model.clone();
        let drag_index_on_down = drag_index_model.clone();
        let drag_index_on_move = drag_index_model.clone();

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
                    let next_index = radix_slider::start_slider_drag_from_pointer_x(
                        host,
                        &model_on_down,
                        bounds,
                        down.position.x,
                        min_value,
                        max_value,
                        step_value,
                        thumb_size,
                        min_steps_between_thumbs_value,
                    );
                    let _ = host
                        .models_mut()
                        .update(&drag_index_on_down, |idx| *idx = next_index);
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
                    let value_index_to_change = host
                        .models_mut()
                        .get_cloned(&drag_index_on_move)
                        .unwrap_or(0);
                    let next_index = radix_slider::update_slider_model_from_pointer_x(
                        host,
                        &model_on_move,
                        bounds,
                        mv.position.x,
                        min_value,
                        max_value,
                        step_value,
                        thumb_size,
                        value_index_to_change,
                        min_steps_between_thumbs_value,
                    );
                    let _ = host
                        .models_mut()
                        .update(&drag_index_on_move, |idx| *idx = next_index);
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
            let drag_index_on_key = drag_index_model.clone();
            let min_steps_between_thumbs_value = min_steps_between_thumbs;
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

                    let mut values = host
                        .models_mut()
                        .get_cloned(&model_on_key)
                        .unwrap_or_else(|| vec![min_value]);
                    if values.is_empty() {
                        values.push(min_value);
                    }

                    let value_index_to_change = host
                        .models_mut()
                        .get_cloned(&drag_index_on_key)
                        .unwrap_or(0)
                        .min(values.len().saturating_sub(1));
                    let cur = values
                        .get(value_index_to_change)
                        .copied()
                        .unwrap_or(min_value);

                    let next = match down.key {
                        fret_core::KeyCode::ArrowLeft | fret_core::KeyCode::ArrowDown => cur - step,
                        fret_core::KeyCode::ArrowRight | fret_core::KeyCode::ArrowUp => cur + step,
                        fret_core::KeyCode::Home => min_value,
                        fret_core::KeyCode::End => max_value,
                        _ => return false,
                    };

                    let commit_index = match down.key {
                        fret_core::KeyCode::Home => 0,
                        fret_core::KeyCode::End => values.len().saturating_sub(1),
                        _ => value_index_to_change,
                    };

                    if let Some(update) = radix_slider::update_multi_thumb_values(
                        &values,
                        next,
                        commit_index,
                        min_value,
                        max_value,
                        step,
                        min_steps_between_thumbs_value,
                    ) {
                        let _ = host
                            .models_mut()
                            .update(&model_on_key, |values| *values = update.values);
                        let _ = host.models_mut().update(&drag_index_on_key, |idx| {
                            *idx = update.value_index_to_change;
                        });
                    }
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
                let thumb_lefts: Vec<f32> =
                    percentages.iter().copied().map(|t| track_w * t).collect();
                let fill_w = track_w * (range_end_t - range_start_t).max(0.0);

                let range = ContainerProps {
                    layout: LayoutStyle {
                        position: PositionStyle::Absolute,
                        inset: fret_ui::element::InsetStyle {
                            left: Some(Px(thumb_r.0 + track_w * range_start_t)),
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

                vec![cx.container(root_container, |cx| {
                    let mut out = vec![
                        cx.container(track, |_| Vec::new()),
                        cx.container(range, |_| Vec::new()),
                    ];

                    for thumb_left in thumb_lefts {
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
                        out.push(cx.container(thumb, |_| Vec::new()));
                    }

                    out
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

    fn pointer_x_for_value(bounds: Rect, value: f32, min: f32, max: f32, thumb_size: Px) -> Px {
        let t = radix_slider::normalize_value(value, min, max);
        let thumb = thumb_size.0.max(0.0);
        let track_w = (bounds.size.width.0 - thumb).max(0.0);
        let left = bounds.origin.x.0 + thumb * 0.5;
        Px(left + track_w * t)
    }

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
                click_count: 1,
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

    #[test]
    fn slider_updates_closest_thumb_when_multi_value() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(60.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(vec![10.0, 90.0]);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-slider-updates-closest-thumb-when-multi-value",
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
                click_count: 1,
            }),
        );

        let values = app.models().get_cloned(&model).unwrap_or_default();
        assert_eq!(values.len(), 2);
        assert!(
            (values[0] - 10.0).abs() < 0.01,
            "expected first thumb ~= 10, got {}",
            values[0]
        );
        assert!(
            (values[1] - 100.0).abs() < 0.01,
            "expected second thumb ~= 100, got {}",
            values[1]
        );
    }

    #[test]
    fn slider_respects_min_steps_between_thumbs() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(240.0), Px(60.0)),
        );
        let mut services = FakeServices;

        let model = app.models_mut().insert(vec![10.0, 14.0]);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-slider-respects-min-steps-between-thumbs",
            |cx| {
                vec![
                    Slider::new(model.clone())
                        .range(0.0, 100.0)
                        .step(1.0)
                        .min_steps_between_thumbs(5)
                        .style(SliderStyle::default())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let slider_node = ui.children(root)[0];
        let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
        let position = Point::new(
            pointer_x_for_value(slider_bounds, 12.0, 0.0, 100.0, Px(16.0)),
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
                click_count: 1,
            }),
        );

        let values = app.models().get_cloned(&model).unwrap_or_default();
        assert_eq!(values, vec![10.0, 14.0]);
    }
}

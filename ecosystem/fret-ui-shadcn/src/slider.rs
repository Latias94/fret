use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, CursorIcon, Edges, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, LayoutStyle, Length, MainAlign, MarginEdge,
    MarginEdges, Overflow, PointerRegionProps, PositionStyle, RowProps,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::slider as radix_slider;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement};

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
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
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
            test_id: None,
            chrome: ChromeRefinement::default(),
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
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
            self.test_id,
            self.chrome,
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
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
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

        let mut root_layout = decl_style::layout_style(&theme, layout.relative().w_full());
        root_layout.overflow = fret_ui::element::Overflow::Visible;

        // Match shadcn/Radix DOM semantics: the layout height follows the track, while the thumb
        // is allowed to overflow (hit-testing is not clipped unless overflow=Clip).
        let root_h = style.track_height.0.max(0.0);

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
        semantics.test_id = test_id.clone();

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
            let active_thumb_focus_target: Rc<Cell<Option<GlobalElementId>>> =
                Rc::new(Cell::new(None));
            let active_thumb_focus_target_on_down = active_thumb_focus_target.clone();
            let on_down = Arc::new(
                move |host: &mut dyn UiPointerActionHost, cx: ActionCx, down: PointerDownCx| {
                    if down.button != MouseButton::Left {
                        return false;
                    }

                    host.request_focus(
                        active_thumb_focus_target_on_down
                            .get()
                            .unwrap_or(semantics_id),
                    );
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

            let track_top = 0.0;
            let thumb_r = Px(style.thumb_size.0.max(0.0) * 0.5);

            let mut root_container =
                decl_style::container_props(&theme, chrome.clone(), LayoutRefinement::default());
            root_container.layout = semantics_layout;

            let track = ContainerProps {
                layout: LayoutStyle {
                    position: PositionStyle::Absolute,
                    overflow: Overflow::Clip,
                    inset: fret_ui::element::InsetStyle {
                        left: Some(Px(0.0)),
                        right: Some(Px(0.0)),
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
                ..Default::default()
            };

            let pointer = PointerRegionProps {
                layout: semantics_layout,
                enabled: !disabled,
            };

            vec![cx.pointer_region(pointer, |cx| {
                cx.pointer_region_on_pointer_down(on_down);
                cx.pointer_region_on_pointer_move(on_move);
                cx.pointer_region_on_pointer_up(on_up);

                let grow_left = range_start_t.clamp(0.0, 1.0);
                let grow_range = (range_end_t - range_start_t).clamp(0.0, 1.0);
                let grow_right = (1.0 - range_end_t).clamp(0.0, 1.0);

                let mut flex_segment_layout = LayoutStyle::default();
                flex_segment_layout.size.height = Length::Fill;
                flex_segment_layout.flex.shrink = 1.0;
                flex_segment_layout.flex.basis = Length::Px(Px(0.0));

                let corner_radius = Px(style.track_height.0.max(0.0) * 0.5);

                vec![cx.container(root_container, |cx| {
                    let track_el = cx.container(track, |cx| {
                        let range_row = RowProps {
                            layout: LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Stretch,
                        };

                        let mut left_layout = flex_segment_layout;
                        left_layout.flex.grow = grow_left;
                        let mut range_layout = flex_segment_layout;
                        range_layout.flex.grow = grow_range;
                        let mut right_layout = flex_segment_layout;
                        right_layout.flex.grow = grow_right;

                        let left = ContainerProps {
                            layout: left_layout,
                            ..Default::default()
                        };
                        let range = ContainerProps {
                            layout: range_layout,
                            background: Some(style.range_background),
                            corner_radii: Corners::all(corner_radius),
                            ..Default::default()
                        };
                        let right = ContainerProps {
                            layout: right_layout,
                            ..Default::default()
                        };

                        vec![cx.row(range_row, |cx| {
                            vec![
                                cx.container(left, |_| Vec::new()),
                                cx.container(range, |_| Vec::new()),
                                cx.container(right, |_| Vec::new()),
                            ]
                        })]
                    });

                    let mut out = vec![track_el];

                    for (thumb_index, t) in percentages.iter().copied().enumerate() {
                        let t = t.clamp(0.0, 1.0);
                        let mut left_layout = flex_segment_layout;
                        left_layout.flex.grow = t;
                        let mut right_layout = flex_segment_layout;
                        right_layout.flex.grow = 1.0 - t;

                        let thumb_row = RowProps {
                            layout: LayoutStyle {
                                position: PositionStyle::Absolute,
                                inset: fret_ui::element::InsetStyle {
                                    left: Some(Px(0.0)),
                                    right: Some(Px(0.0)),
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
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                        };

                        let left = ContainerProps {
                            layout: left_layout,
                            ..Default::default()
                        };
                        let right = ContainerProps {
                            layout: right_layout,
                            ..Default::default()
                        };

                        let mut thumb_layout = LayoutStyle::default();
                        thumb_layout.size.width = Length::Px(style.thumb_size);
                        thumb_layout.size.height = Length::Px(style.thumb_size);
                        thumb_layout.flex.shrink = 0.0;
                        thumb_layout.margin = MarginEdges {
                            left: MarginEdge::Px(Px(-thumb_r.0)),
                            right: MarginEdge::Px(Px(-thumb_r.0)),
                            ..Default::default()
                        };

                        let thumb = ContainerProps {
                            layout: LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
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
                            ..Default::default()
                        };

                        out.push(cx.row(thumb_row, |cx| {
                            let thumb_value = values_sorted
                                .get(thumb_index)
                                .copied()
                                .unwrap_or(active_value);
                            let mut thumb_semantics = radix_slider::slider_thumb_semantics(
                                a11y_label.clone(),
                                thumb_value,
                                disabled,
                            );
                            thumb_semantics.layout = thumb_layout;
                            vec![
                                cx.container(left, |_| Vec::new()),
                                cx.semantics_with_id(thumb_semantics, |cx, thumb_semantics_id| {
                                    let model_on_key = model.clone();
                                    let drag_index_on_key = drag_index_model.clone();
                                    let min_steps_between_thumbs_value = min_steps_between_thumbs;
                                    cx.key_on_key_down_for(
                                        thumb_semantics_id,
                                        Arc::new(move |host, cx, down| {
                                            if down.repeat
                                                || down.modifiers.alt
                                                || down.modifiers.ctrl
                                                || down.modifiers.meta
                                            {
                                                return false;
                                            }

                                            let step = if step_value.is_finite() && step_value > 0.0
                                            {
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

                                            let value_index_to_change =
                                                thumb_index.min(values.len().saturating_sub(1));
                                            let cur = values
                                                .get(value_index_to_change)
                                                .copied()
                                                .unwrap_or(min_value);

                                            let next = match down.key {
                                                fret_core::KeyCode::ArrowLeft
                                                | fret_core::KeyCode::ArrowDown => cur - step,
                                                fret_core::KeyCode::ArrowRight
                                                | fret_core::KeyCode::ArrowUp => cur + step,
                                                fret_core::KeyCode::Home => min_value,
                                                fret_core::KeyCode::End => max_value,
                                                _ => return false,
                                            };

                                            if let Some(update) =
                                                radix_slider::update_multi_thumb_values(
                                                    &values,
                                                    next,
                                                    value_index_to_change,
                                                    min_value,
                                                    max_value,
                                                    step,
                                                    min_steps_between_thumbs_value,
                                                )
                                            {
                                                let _ = host.models_mut().update(
                                                    &model_on_key,
                                                    |values| {
                                                        *values = update.values;
                                                    },
                                                );
                                                let _ = host.models_mut().update(
                                                    &drag_index_on_key,
                                                    |idx| {
                                                        *idx = update.value_index_to_change;
                                                    },
                                                );
                                            }
                                            host.request_redraw(cx.window);
                                            true
                                        }),
                                    );
                                    if thumb_index == active_index {
                                        active_thumb_focus_target.set(Some(thumb_semantics_id));
                                    }
                                    vec![cx.container(thumb, |_| Vec::new())]
                                }),
                                cx.container(right, |_| Vec::new()),
                            ]
                        }));
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
        let _ = thumb_size;
        let track_w = bounds.size.width.0.max(0.0);
        let left = bounds.origin.x.0;
        Px(left + track_w * t)
    }

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
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
                pointer_id: fret_core::PointerId(0),
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
                pointer_id: fret_core::PointerId(0),
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
                pointer_id: fret_core::PointerId(0),
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

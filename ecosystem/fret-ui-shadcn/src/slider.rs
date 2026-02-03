use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, CursorIcon, Edges, MouseButton, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, LayoutStyle, Length, MainAlign, MarginEdge,
    MarginEdges, Overflow, PointerRegionProps, PositionStyle, RowProps, SemanticsProps,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::slider as radix_slider;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, OverrideSlot, WidgetState, WidgetStateProperty,
    WidgetStates,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct SliderStyle {
    pub track_height: Option<Px>,
    pub track_background: OverrideSlot<ColorRef>,
    pub track_border_color: OverrideSlot<ColorRef>,
    pub range_background: OverrideSlot<ColorRef>,
    pub thumb_size: Option<Px>,
    pub thumb_background: OverrideSlot<ColorRef>,
    pub thumb_border_color: OverrideSlot<ColorRef>,
    pub thumb_ring_color: OverrideSlot<ColorRef>,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_height: None,
            track_background: None,
            track_border_color: None,
            range_background: None,
            thumb_size: None,
            thumb_background: None,
            thumb_border_color: None,
            thumb_ring_color: None,
        }
    }
}

impl SliderStyle {
    pub fn track_height(mut self, track_height: Px) -> Self {
        self.track_height = Some(track_height);
        self
    }

    pub fn track_background(
        mut self,
        track_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.track_background = Some(track_background);
        self
    }

    pub fn track_border_color(
        mut self,
        track_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.track_border_color = Some(track_border_color);
        self
    }

    pub fn range_background(
        mut self,
        range_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.range_background = Some(range_background);
        self
    }

    pub fn thumb_size(mut self, thumb_size: Px) -> Self {
        self.thumb_size = Some(thumb_size);
        self
    }

    pub fn thumb_background(
        mut self,
        thumb_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.thumb_background = Some(thumb_background);
        self
    }

    pub fn thumb_border_color(
        mut self,
        thumb_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.thumb_border_color = Some(thumb_border_color);
        self
    }

    pub fn thumb_ring_color(
        mut self,
        thumb_ring_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.thumb_ring_color = Some(thumb_ring_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.track_height.is_some() {
            self.track_height = other.track_height;
        }
        if other.track_background.is_some() {
            self.track_background = other.track_background;
        }
        if other.track_border_color.is_some() {
            self.track_border_color = other.track_border_color;
        }
        if other.range_background.is_some() {
            self.range_background = other.range_background;
        }
        if other.thumb_size.is_some() {
            self.thumb_size = other.thumb_size;
        }
        if other.thumb_background.is_some() {
            self.thumb_background = other.thumb_background;
        }
        if other.thumb_border_color.is_some() {
            self.thumb_border_color = other.thumb_border_color;
        }
        if other.thumb_ring_color.is_some() {
            self.thumb_ring_color = other.thumb_ring_color;
        }
        self
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
    style: SliderStyle,
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
            style: SliderStyle::default(),
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
        self.style = self.style.merged(style);
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
    style: SliderStyle,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    cx.scope(|cx| {
        #[derive(Default)]
        struct DragIndexState {
            model: Option<Model<usize>>,
            dragging: Option<Model<bool>>,
            hovered: Option<Model<bool>>,
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

        let dragging_model = cx.with_state(DragIndexState::default, |st| st.dragging.clone());
        let dragging_model = if let Some(dragging_model) = dragging_model {
            dragging_model
        } else {
            let dragging_model = cx.app.models_mut().insert(false);
            cx.with_state(DragIndexState::default, |st| {
                st.dragging = Some(dragging_model.clone());
            });
            dragging_model
        };

        let hovered_model = cx.with_state(DragIndexState::default, |st| st.hovered.clone());
        let hovered_model = if let Some(hovered_model) = hovered_model {
            hovered_model
        } else {
            let hovered_model = cx.app.models_mut().insert(false);
            cx.with_state(DragIndexState::default, |st| {
                st.hovered = Some(hovered_model.clone());
            });
            hovered_model
        };

        let snapshot = theme.snapshot();
        let enabled = !disabled;

        let SliderStyle {
            track_height: track_height_override,
            track_background: track_background_override,
            track_border_color: track_border_color_override,
            range_background: range_background_override,
            thumb_size: thumb_size_override,
            thumb_background: thumb_background_override,
            thumb_border_color: thumb_border_color_override,
            thumb_ring_color: thumb_ring_color_override,
        } = style;

        let track_height = track_height_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.slider.track_height")
                .unwrap_or(Px(4.0))
        });
        let thumb_size = thumb_size_override.unwrap_or_else(|| {
            theme
                .metric_by_key("component.slider.thumb_size")
                .unwrap_or(Px(16.0))
        });

        let track_bg = theme
            .color_by_key("muted")
            .unwrap_or(snapshot.colors.panel_background);
        let range_bg = theme
            .color_by_key("primary")
            .or_else(|| theme.color_by_key("accent"))
            .unwrap_or(snapshot.colors.accent);
        let thumb_bg = theme
            .color_by_key("background")
            .unwrap_or(snapshot.colors.surface_background);
        let thumb_border = theme
            .color_by_key("input")
            .or_else(|| theme.color_by_key("border"))
            .unwrap_or(snapshot.colors.panel_border);
        let ring_color = theme.color_required("ring");

        let default_track_background = WidgetStateProperty::new(ColorRef::Color(track_bg));
        let default_track_border_color =
            WidgetStateProperty::new(ColorRef::Color(Color::TRANSPARENT)).when(
                WidgetStates::ACTIVE,
                ColorRef::Color(alpha_mul(ring_color, 0.55)),
            );
        let default_range_background = WidgetStateProperty::new(ColorRef::Color(range_bg));
        let default_thumb_background = WidgetStateProperty::new(ColorRef::Color(thumb_bg));
        let default_thumb_border_color = WidgetStateProperty::new(ColorRef::Color(thumb_border))
            .when(
                WidgetStates::ACTIVE,
                ColorRef::Color(alpha_mul(ring_color, 0.85)),
            )
            .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring_color));
        let default_thumb_ring_color = WidgetStateProperty::new(ColorRef::Color(ring_color));

        let mut root_layout = decl_style::layout_style(&theme, layout.relative().w_full());
        root_layout.overflow = fret_ui::element::Overflow::Visible;

        // Match shadcn/Radix DOM semantics: the layout height follows the track, while the thumb
        // is allowed to overflow (hit-testing is not clipped unless overflow=Clip).
        let root_h = track_height.0.max(0.0);

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
        let test_id_prefix = test_id.clone();

        let min_value = min;
        let max_value = max;
        let step_value = step;
        let thumb_size = thumb_size;
        let min_steps_between_thumbs_value = min_steps_between_thumbs;
        let model_on_down = model.clone();
        let model_on_move = model.clone();
        let drag_index_on_down = drag_index_model.clone();
        let drag_index_on_move = drag_index_model.clone();
        let dragging_on_down = dragging_model.clone();
        let dragging_on_move = dragging_model.clone();
        let dragging_on_up = dragging_model.clone();
        let dragging_on_cancel = dragging_model.clone();
        let hovered_on_down = hovered_model.clone();
        let hovered_on_move = hovered_model.clone();
        let hovered_on_up = hovered_model.clone();
        let hovered_on_cancel = hovered_model.clone();

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
                    let _ = host.models_mut().update(&dragging_on_down, |v| *v = true);
                    let _ = host.models_mut().update(&hovered_on_down, |v| *v = true);

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
                    let hovered = host.bounds().contains(mv.position);
                    let _ = host.models_mut().update(&hovered_on_move, |v| *v = hovered);
                    if !mv.buttons.left {
                        return false;
                    }
                    let _ = host.models_mut().update(&dragging_on_move, |v| *v = true);

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
                    let _ = host.models_mut().update(&dragging_on_up, |v| *v = false);
                    let hovered = host.bounds().contains(up.position);
                    let _ = host.models_mut().update(&hovered_on_up, |v| *v = hovered);
                    host.request_redraw(cx.window);
                    true
                },
            );

            let on_cancel = Arc::new(
                move |host: &mut dyn UiPointerActionHost, cx: ActionCx, _cancel| {
                    host.release_pointer_capture();
                    let _ = host
                        .models_mut()
                        .update(&dragging_on_cancel, |v| *v = false);
                    let _ = host.models_mut().update(&hovered_on_cancel, |v| *v = false);
                    host.request_redraw(cx.window);
                    true
                },
            );

            let track_top = 0.0;
            let thumb_r = Px(thumb_size.0.max(0.0) * 0.5);

            let mut root_container =
                decl_style::container_props(&theme, chrome.clone(), LayoutRefinement::default());
            root_container.layout = semantics_layout;

            let is_dragging = cx.app.models().get_copied(&dragging_model).unwrap_or(false);
            let is_hovered = cx.app.models().get_copied(&hovered_model).unwrap_or(false);

            let mut root_states = WidgetStates::default();
            root_states.set(WidgetState::Disabled, !enabled);
            root_states.set(WidgetState::Hovered, is_hovered && enabled);
            root_states.set(WidgetState::Active, is_dragging && enabled);

            let track_bg = track_background_override
                .as_ref()
                .and_then(|p| p.resolve(root_states).clone())
                .unwrap_or_else(|| default_track_background.resolve(root_states).clone())
                .resolve(&theme);
            let track_border_color = track_border_color_override
                .as_ref()
                .and_then(|p| p.resolve(root_states).clone())
                .unwrap_or_else(|| default_track_border_color.resolve(root_states).clone())
                .resolve(&theme);
            let range_bg = range_background_override
                .as_ref()
                .and_then(|p| p.resolve(root_states).clone())
                .unwrap_or_else(|| default_range_background.resolve(root_states).clone())
                .resolve(&theme);

            let track_border = Edges::all(Px(0.0));
            let thumb_border = Edges::all(Px(1.0));

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
                        height: Length::Px(track_height),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges::all(Px(0.0)),
                background: Some(track_bg),
                shadow: None,
                border: track_border,
                border_color: Some(track_border_color),
                corner_radii: Corners::all(Px(track_height.0.max(0.0) * 0.5)),
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
                cx.pointer_region_on_pointer_cancel(on_cancel);

                let grow_left = range_start_t.clamp(0.0, 1.0);
                let grow_range = (range_end_t - range_start_t).clamp(0.0, 1.0);
                let grow_right = (1.0 - range_end_t).clamp(0.0, 1.0);

                let mut flex_segment_layout = LayoutStyle::default();
                flex_segment_layout.size.height = Length::Fill;
                flex_segment_layout.flex.shrink = 1.0;
                flex_segment_layout.flex.basis = Length::Px(Px(0.0));

                let corner_radius = Px(track_height.0.max(0.0) * 0.5);

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
                            background: Some(range_bg),
                            corner_radii: Corners::all(corner_radius),
                            ..Default::default()
                        };
                        let right = ContainerProps {
                            layout: right_layout,
                            ..Default::default()
                        };

                        vec![cx.row(range_row, |cx| {
                            let range_el = cx.container(range, |_| Vec::new());
                            let range_el = if let Some(test_id) = test_id_prefix.as_ref() {
                                cx.semantics(
                                    SemanticsProps {
                                        layout: range.layout,
                                        test_id: Some(Arc::<str>::from(format!("{test_id}-range"))),
                                        ..Default::default()
                                    },
                                    move |_cx| vec![range_el],
                                )
                            } else {
                                range_el
                            };

                            vec![
                                cx.container(left, |_| Vec::new()),
                                range_el,
                                cx.container(right, |_| Vec::new()),
                            ]
                        })]
                    });

                    let track_el = if let Some(test_id) = test_id_prefix.as_ref() {
                        cx.semantics(
                            SemanticsProps {
                                layout: track.layout,
                                test_id: Some(Arc::<str>::from(format!("{test_id}-track"))),
                                ..Default::default()
                            },
                            move |_cx| vec![track_el],
                        )
                    } else {
                        track_el
                    };

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
                                    height: Length::Px(track_height),
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
                        thumb_layout.size.width = Length::Px(thumb_size);
                        thumb_layout.size.height = Length::Px(thumb_size);
                        thumb_layout.flex.shrink = 0.0;
                        // Radix keeps the thumb inside the slider bounds at the edges via
                        // `getThumbInBoundsOffset`. Model the same outcome by shifting the thumb
                        // with asymmetric negative margins while keeping the flex footprint at 0.
                        let thumb_in_bounds_offset = Px(thumb_r.0 * (1.0 - 2.0 * t));
                        thumb_layout.margin = MarginEdges {
                            left: MarginEdge::Px(Px(-thumb_r.0 + thumb_in_bounds_offset.0)),
                            right: MarginEdge::Px(Px(-thumb_r.0 - thumb_in_bounds_offset.0)),
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
                            if let Some(test_id) = test_id_prefix.as_ref() {
                                thumb_semantics.test_id = Some(Arc::<str>::from(format!(
                                    "{test_id}-thumb-{thumb_index}"
                                )));
                            }
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

                                    let is_focused = cx.is_focused_element(thumb_semantics_id);
                                    let focus_visible = is_focused
                                        && fret_ui::focus_visible::is_focus_visible(
                                            cx.app,
                                            Some(cx.window),
                                        );
                                    let active_idx =
                                        cx.app.models().get_copied(&drag_index_model).unwrap_or(0);

                                    let mut thumb_states = WidgetStates::default();
                                    thumb_states.set(WidgetState::Disabled, !enabled);
                                    thumb_states.set(WidgetState::Hovered, is_hovered && enabled);
                                    thumb_states.set(
                                        WidgetState::Active,
                                        is_dragging && enabled && active_idx == thumb_index,
                                    );
                                    thumb_states
                                        .set(WidgetState::FocusVisible, focus_visible && enabled);

                                    let bg = thumb_background_override
                                        .as_ref()
                                        .and_then(|p| p.resolve(thumb_states).clone())
                                        .unwrap_or_else(|| {
                                            default_thumb_background.resolve(thumb_states).clone()
                                        })
                                        .resolve(&theme);
                                    let border_color = thumb_border_color_override
                                        .as_ref()
                                        .and_then(|p| p.resolve(thumb_states).clone())
                                        .unwrap_or_else(|| {
                                            default_thumb_border_color.resolve(thumb_states).clone()
                                        })
                                        .resolve(&theme);

                                    let layout_fill = LayoutStyle {
                                        size: fret_ui::element::SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    };

                                    let thumb = ContainerProps {
                                        layout: layout_fill,
                                        padding: Edges::all(Px(0.0)),
                                        background: Some(bg),
                                        shadow: None,
                                        border: thumb_border,
                                        border_color: Some(border_color),
                                        corner_radii: Corners::all(thumb_r),
                                        ..Default::default()
                                    };

                                    if !focus_visible || !enabled {
                                        return vec![cx.container(thumb, |_| Vec::new())];
                                    }

                                    let ring_color = thumb_ring_color_override
                                        .as_ref()
                                        .and_then(|p| p.resolve(thumb_states).clone())
                                        .unwrap_or_else(|| {
                                            default_thumb_ring_color.resolve(thumb_states).clone()
                                        })
                                        .resolve(&theme);
                                    let ring = ContainerProps {
                                        layout: layout_fill,
                                        padding: Edges::all(Px(0.0)),
                                        background: Some(Color::TRANSPARENT),
                                        shadow: None,
                                        border: Edges::all(Px(2.0)),
                                        border_color: Some(ring_color),
                                        corner_radii: Corners::all(thumb_r),
                                        ..Default::default()
                                    };

                                    vec![cx.stack_props(
                                        fret_ui::element::StackProps {
                                            layout: layout_fill,
                                        },
                                        |cx| {
                                            vec![
                                                cx.container(ring, |_| Vec::new()),
                                                cx.container(thumb, |_| Vec::new()),
                                            ]
                                        },
                                    )]
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
        TextMetrics, TextService,
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

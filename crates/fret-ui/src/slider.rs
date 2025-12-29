use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Px, Rect, SemanticsRole, Size,
};
use fret_runtime::{Model, ModelId};

use crate::element::RingStyle;
use crate::widget::{EventCx, LayoutCx, PaintCx, SemanticsCx, Widget};
use crate::{Invalidation, Theme, UiHost};

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
    pub focus_ring: Option<RingStyle>,
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

#[derive(Debug, Clone, Copy)]
struct DragState {
    index: usize,
    grab_offset_x: f32,
}

pub struct BoundSlider {
    model: Model<Vec<f32>>,
    last_revision: Option<u64>,
    dirty_since_sync: bool,
    min: f32,
    max: f32,
    step: f32,
    enabled: bool,
    style: SliderStyle,
    dragging: Option<DragState>,
    active_index: usize,
    last_bounds: Rect,
}

impl BoundSlider {
    pub fn new(model: Model<Vec<f32>>) -> Self {
        Self {
            model,
            last_revision: None,
            dirty_since_sync: false,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            enabled: true,
            style: SliderStyle::default(),
            dragging: None,
            active_index: 0,
            last_bounds: Rect::default(),
        }
    }

    pub fn model_id(&self) -> ModelId {
        self.model.id()
    }

    pub fn set_model(&mut self, model: Model<Vec<f32>>) {
        if self.model.id() == model.id() {
            return;
        }
        self.model = model;
        self.last_revision = None;
        self.dirty_since_sync = false;
        self.active_index = 0;
        self.dragging = None;
    }

    pub fn set_range(&mut self, min: f32, max: f32) {
        self.min = min;
        self.max = max;
    }

    pub fn set_step(&mut self, step: f32) {
        self.step = step;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.dragging = None;
        }
    }

    pub fn set_style(&mut self, style: SliderStyle) {
        self.style = style;
    }

    fn range_span(&self) -> Option<f32> {
        let span = self.max - self.min;
        if !span.is_finite() || span.abs() <= f32::EPSILON {
            None
        } else {
            Some(span)
        }
    }

    fn clamp_value(&self, v: f32) -> f32 {
        if !v.is_finite() {
            return self.min;
        }
        let mut out = v.clamp(self.min, self.max);
        if self.step.is_finite() && self.step > 0.0 {
            let steps = ((out - self.min) / self.step).round();
            out = (self.min + steps * self.step).clamp(self.min, self.max);
        }
        out
    }

    fn values_from_model<H: UiHost>(&mut self, app: &H, force: bool) -> Vec<f32> {
        let revision = app.models().revision(self.model);
        if !force && revision == self.last_revision {
            let Some(v) = app.models().get(self.model) else {
                return vec![self.min];
            };
            return v.clone();
        }
        self.last_revision = revision;

        let Some(v) = app.models().get(self.model) else {
            return vec![self.min];
        };
        v.clone()
    }

    fn sanitize_values(&self, mut values: Vec<f32>) -> Vec<f32> {
        if values.is_empty() {
            values.push(self.min);
        }
        for v in &mut values {
            *v = self.clamp_value(*v);
        }
        values.sort_by(|a, b| a.total_cmp(b));
        values
    }

    fn maybe_update_model<H: UiHost>(&mut self, app: &mut H, next: Vec<f32>) {
        let next = self.sanitize_values(next);
        if app.models_mut().update(self.model, |v| *v = next).is_ok() {
            self.dirty_since_sync = false;
            self.last_revision = app.models().revision(self.model);
        }
    }

    pub fn cleanup_resources(&mut self, _services: &mut dyn fret_core::UiServices) {}

    fn track_rect(&self, bounds: Rect) -> Rect {
        let h = self.style.track_height.0.max(0.0);
        let thumb = self.style.thumb_size.0.max(0.0);
        let left = bounds.origin.x.0 + thumb * 0.5;
        let right = bounds.origin.x.0 + bounds.size.width.0 - thumb * 0.5;
        let w = (right - left).max(0.0);
        let y = bounds.origin.y.0 + (bounds.size.height.0 - h) * 0.5;
        Rect::new(
            fret_core::Point::new(Px(left), Px(y)),
            Size::new(Px(w), Px(h)),
        )
    }

    fn value_to_x(&self, track: Rect, v: f32) -> Px {
        let Some(span) = self.range_span() else {
            return track.origin.x;
        };
        let t = ((v - self.min) / span).clamp(0.0, 1.0);
        Px(track.origin.x.0 + track.size.width.0 * t)
    }

    fn x_to_value(&self, track: Rect, x: Px) -> f32 {
        let Some(span) = self.range_span() else {
            return self.min;
        };
        if track.size.width.0 <= 0.0 {
            return self.min;
        }
        let t = ((x.0 - track.origin.x.0) / track.size.width.0).clamp(0.0, 1.0);
        self.clamp_value(self.min + span * t)
    }

    fn thumb_rect(&self, bounds: Rect, track: Rect, v: f32) -> Rect {
        let size = self.style.thumb_size.0.max(0.0);
        let cx = self.value_to_x(track, v).0;
        let cy = bounds.origin.y.0 + bounds.size.height.0 * 0.5;
        Rect::new(
            fret_core::Point::new(Px(cx - size * 0.5), Px(cy - size * 0.5)),
            Size::new(Px(size), Px(size)),
        )
    }

    fn pick_nearest_index(&self, values: &[f32], track: Rect, x: Px) -> usize {
        if values.is_empty() {
            return 0;
        }
        let target = self.x_to_value(track, x);
        let mut best = 0usize;
        let mut best_dist = f32::INFINITY;
        for (i, v) in values.iter().copied().enumerate() {
            let dist = (v - target).abs();
            if dist < best_dist {
                best = i;
                best_dist = dist;
            }
        }
        best
    }

    fn set_index_value(&self, values: &mut [f32], index: usize, next: f32) {
        if values.is_empty() {
            return;
        }
        let next = self.clamp_value(next);
        let min_neighbor = if index > 0 {
            values[index - 1]
        } else {
            self.min
        };
        let max_neighbor = if index + 1 < values.len() {
            values[index + 1]
        } else {
            self.max
        };
        values[index] = next.clamp(min_neighbor, max_neighbor);
    }
}

impl<H: UiHost> Widget<H> for BoundSlider {
    fn is_focusable(&self) -> bool {
        self.enabled
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Slider);
        cx.set_focusable(self.enabled);
        cx.set_disabled(!self.enabled);

        let raw = self.values_from_model(cx.app, false);
        let values = self.sanitize_values(raw);
        let text = values.first().copied().unwrap_or(self.min).to_string();
        cx.set_value(text);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if !self.enabled {
            return;
        }

        self.last_bounds = cx.bounds;
        let track = self.track_rect(cx.bounds);

        match event {
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                ..
            }) => {
                let mut raw = self.values_from_model(cx.app, false);
                raw = self.sanitize_values(raw);

                let mut picked = None;
                for (i, v) in raw.iter().copied().enumerate() {
                    let rect = self.thumb_rect(cx.bounds, track, v);
                    if rect.contains(*position) {
                        picked = Some((i, rect));
                        break;
                    }
                }

                let (index, grab_offset_x) = if let Some((i, rect)) = picked {
                    let center_x = rect.origin.x.0 + rect.size.width.0 * 0.5;
                    (i, position.x.0 - center_x)
                } else {
                    let idx = self.pick_nearest_index(&raw, track, position.x);
                    let next = self.x_to_value(track, position.x);
                    self.set_index_value(&mut raw, idx, next);
                    self.maybe_update_model(cx.app, raw.clone());
                    (idx, 0.0)
                };

                self.active_index = index;
                self.dragging = Some(DragState {
                    index,
                    grab_offset_x,
                });
                self.dirty_since_sync = true;
                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let mut raw = self.values_from_model(cx.app, false);
                raw = self.sanitize_values(raw);

                if let Some(drag) = self.dragging {
                    let clamped_index = drag.index.min(raw.len().saturating_sub(1));
                    let x = Px(position.x.0 - drag.grab_offset_x);
                    let next = self.x_to_value(track, x);
                    self.set_index_value(&mut raw, clamped_index, next);
                    self.active_index = clamped_index;
                    self.maybe_update_model(cx.app, raw);
                    cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                } else {
                    cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                button: MouseButton::Left,
                ..
            }) => {
                if cx.captured == Some(cx.node) {
                    self.dragging = None;
                    self.dirty_since_sync = false;
                    cx.release_pointer_capture();
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if modifiers.ctrl || modifiers.alt || modifiers.meta {
                    return;
                }

                let step = if self.step.is_finite() && self.step > 0.0 {
                    self.step
                } else {
                    1.0
                };

                let mut raw = self.values_from_model(cx.app, false);
                raw = self.sanitize_values(raw);
                if raw.is_empty() {
                    raw.push(self.min);
                }
                let idx = self.active_index.min(raw.len() - 1);

                let mut next = raw[idx];
                match key {
                    fret_core::KeyCode::ArrowLeft | fret_core::KeyCode::ArrowDown => {
                        next -= step;
                    }
                    fret_core::KeyCode::ArrowRight | fret_core::KeyCode::ArrowUp => {
                        next += step;
                    }
                    fret_core::KeyCode::Home => {
                        next = self.min;
                    }
                    fret_core::KeyCode::End => {
                        next = self.max;
                    }
                    _ => return,
                }
                self.set_index_value(&mut raw, idx, next);
                self.maybe_update_model(cx.app, raw);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let theme = Theme::global(&*cx.app).clone();
        if self.style.focus_ring.is_none() {
            self.style.focus_ring = Some(crate::element::RingStyle {
                placement: crate::element::RingPlacement::Outset,
                width: theme
                    .metric_by_key("component.ring.width")
                    .unwrap_or(Px(2.0)),
                offset: theme
                    .metric_by_key("component.ring.offset")
                    .unwrap_or(Px(2.0)),
                color: theme
                    .color_by_key("ring")
                    .unwrap_or(theme.colors.focus_ring),
                offset_color: Some(
                    theme
                        .color_by_key("ring-offset-background")
                        .unwrap_or(theme.colors.surface_background),
                ),
                corner_radii: Corners::all(Px(self.style.thumb_size.0.max(0.0) * 0.5)),
            });
        }

        let desired_h = Px(self
            .style
            .thumb_size
            .0
            .max(self.style.track_height.0)
            .max(0.0));
        Size::new(
            cx.available.width,
            Px(desired_h.0.min(cx.available.height.0.max(0.0))),
        )
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::Paint);
        let raw = self.values_from_model(cx.app, false);
        let values = self.sanitize_values(raw);

        let track = self.track_rect(cx.bounds);
        let radius = Px(track.size.height.0.max(0.0) * 0.5);
        cx.scene.push(fret_core::SceneOp::Quad {
            order: DrawOrder(0),
            rect: track,
            background: self.style.track_background,
            border: self.style.track_border,
            border_color: self.style.track_border_color,
            corner_radii: Corners::all(radius),
        });

        if !values.is_empty() {
            let (fill_start, fill_end) = if values.len() == 1 {
                (self.min, values[0])
            } else {
                (values[0], *values.last().unwrap_or(&values[0]))
            };
            let x0 = self.value_to_x(track, fill_start);
            let x1 = self.value_to_x(track, fill_end);
            let left = Px(x0.0.min(x1.0));
            let right = Px(x0.0.max(x1.0));
            let w = Px((right.0 - left.0).max(0.0));
            let fill_rect = Rect::new(
                fret_core::Point::new(left, track.origin.y),
                Size::new(w, track.size.height),
            );
            cx.scene.push(fret_core::SceneOp::Quad {
                order: DrawOrder(1),
                rect: fill_rect,
                background: self.style.range_background,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });
        }

        let focused = cx.focus == Some(cx.node);
        for (i, v) in values.iter().copied().enumerate() {
            let rect = self.thumb_rect(cx.bounds, track, v);
            let thumb_radius = Px(rect.size.width.0.max(0.0) * 0.5);
            cx.scene.push(fret_core::SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                background: self.style.thumb_background,
                border: self.style.thumb_border,
                border_color: self.style.thumb_border_color,
                corner_radii: Corners::all(thumb_radius),
            });

            if focused
                && i == self.active_index
                && crate::focus_visible::is_focus_visible(cx.app, cx.window)
                && let Some(mut ring) = self.style.focus_ring
            {
                ring.corner_radii = Corners::all(thumb_radius);
                crate::paint::paint_focus_ring(cx.scene, DrawOrder(3), rect, ring);
            }
        }
    }
}

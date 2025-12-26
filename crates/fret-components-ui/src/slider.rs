use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Px, Rect, SceneOp,
    SemanticsRole, Size,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::style::{ColorFallback, MetricFallback, component_color, component_metric};
use crate::{Sizable, Size as ComponentSize};

#[derive(Debug, Clone)]
struct ResolvedSliderStyle {
    height: Px,
    track_height: Px,
    track_radius: Px,
    border_width: Px,
    padding_x: Px,
    thumb_radius: Px,
    track_bg: Color,
    track_fill: Color,
    border: Color,
    thumb: Color,
    thumb_border: Color,
    focus_ring: Color,
}

impl Default for ResolvedSliderStyle {
    fn default() -> Self {
        Self {
            height: Px(28.0),
            track_height: Px(6.0),
            track_radius: Px(999.0),
            border_width: Px(1.0),
            padding_x: Px(8.0),
            thumb_radius: Px(8.0),
            track_bg: Color::TRANSPARENT,
            track_fill: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            thumb: Color::TRANSPARENT,
            thumb_border: Color::TRANSPARENT,
            focus_ring: Color::TRANSPARENT,
        }
    }
}

pub struct Slider {
    model: Model<f32>,
    min: f32,
    max: f32,
    step: Option<f32>,
    disabled: bool,
    size: ComponentSize,

    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    resolved: ResolvedSliderStyle,
}

impl Slider {
    pub fn new(model: Model<f32>) -> Self {
        Self {
            model,
            min: 0.0,
            max: 1.0,
            step: None,
            disabled: false,
            size: ComponentSize::Medium,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            resolved: ResolvedSliderStyle::default(),
        }
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn step(mut self, step: Option<f32>) -> Self {
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn sync_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let (height_default, track_h_default, thumb_r_default) = match self.size {
            ComponentSize::XSmall => (Px(24.0), Px(5.0), Px(7.0)),
            ComponentSize::Small => (Px(26.0), Px(6.0), Px(7.0)),
            ComponentSize::Medium => (Px(28.0), Px(6.0), Px(8.0)),
            ComponentSize::Large => (Px(32.0), Px(7.0), Px(9.0)),
        };
        let height = component_metric(
            "component.slider.height",
            MetricFallback::Px(height_default),
        )
        .resolve(theme);
        let track_height = component_metric(
            "component.slider.track_height",
            MetricFallback::Px(track_h_default),
        )
        .resolve(theme);
        let track_radius = component_metric(
            "component.slider.track_radius",
            MetricFallback::Px(Px(999.0)),
        )
        .resolve(theme);
        let border_width =
            component_metric("component.slider.border_width", MetricFallback::Px(Px(1.0)))
                .resolve(theme);
        let padding_x = component_metric(
            "component.slider.padding_x",
            MetricFallback::Px(self.size.input_px(theme)),
        )
        .resolve(theme);
        let thumb_radius = component_metric(
            "component.slider.thumb_radius",
            MetricFallback::Px(thumb_r_default),
        )
        .resolve(theme);

        let track_bg = component_color(
            "component.slider.track_bg",
            // Use a subtle overlay color rather than a flat background token so the track remains
            // visible across dark themes.
            ColorFallback::ThemeHoverBackground,
        )
        .resolve(theme);
        let track_fill = component_color("component.slider.track_fill", ColorFallback::ThemeAccent)
            .resolve(theme);
        let border = component_color("component.slider.border", ColorFallback::ThemePanelBorder)
            .resolve(theme);
        let thumb = component_color("component.slider.thumb", ColorFallback::ThemeTextPrimary)
            .resolve(theme);
        let thumb_border = component_color(
            "component.slider.thumb_border",
            ColorFallback::ThemePanelBorder,
        )
        .resolve(theme);

        self.resolved = ResolvedSliderStyle {
            height,
            track_height,
            track_radius,
            border_width,
            padding_x,
            thumb_radius,
            track_bg,
            track_fill,
            border,
            thumb,
            thumb_border,
            focus_ring: theme.colors.focus_ring,
        };
    }

    fn value<H: UiHost>(&self, app: &H) -> f32 {
        app.models().get(self.model).copied().unwrap_or(self.min)
    }

    fn set_value<H: UiHost>(&self, app: &mut H, v: f32) {
        let min = self.min.min(self.max);
        let max = self.min.max(self.max);
        let mut next = v;
        if !next.is_finite() {
            return;
        }
        next = next.clamp(min, max);
        if let Some(step) = self.step {
            if step.is_finite() && step > 0.0 {
                let t = (next - min) / step;
                next = min + t.round() * step;
                next = next.clamp(min, max);
            }
        }
        let _ = app.models_mut().update(self.model, |cur| *cur = next);
    }

    fn normalized(&self, value: f32) -> f32 {
        let min = self.min;
        let max = self.max;
        let span = max - min;
        if !span.is_finite() || span.abs() <= f32::EPSILON {
            return 0.0;
        }
        ((value - min) / span).clamp(0.0, 1.0)
    }

    fn track_rect(&self, bounds: Rect) -> Rect {
        let pad_x = self.resolved.padding_x.0.max(0.0);
        let track_h = self
            .resolved
            .track_height
            .0
            .max(0.0)
            .min(bounds.size.height.0);
        let x0 = bounds.origin.x.0 + pad_x;
        let x1 = (bounds.origin.x.0 + bounds.size.width.0 - pad_x).max(x0);
        let y = bounds.origin.y.0 + (bounds.size.height.0 - track_h) * 0.5;
        Rect::new(
            fret_core::Point::new(Px(x0), Px(y)),
            Size::new(Px((x1 - x0).max(0.0)), Px(track_h)),
        )
    }

    fn thumb_rect(&self, bounds: Rect, t: f32) -> Rect {
        let track = self.track_rect(bounds);
        let r = self.resolved.thumb_radius.0.max(0.0);
        let d = r * 2.0;
        let x = track.origin.x.0 + (track.size.width.0 * t).clamp(0.0, track.size.width.0);
        let cx = x;
        let cy = bounds.origin.y.0 + bounds.size.height.0 * 0.5;
        Rect::new(
            fret_core::Point::new(Px((cx - r).max(track.origin.x.0 - r)), Px(cy - r)),
            Size::new(Px(d), Px(d)),
        )
    }

    fn value_from_position(&self, bounds: Rect, x: Px) -> f32 {
        let track = self.track_rect(bounds);
        if track.size.width.0 <= 0.0 {
            return self.min;
        }
        let local = (x.0 - track.origin.x.0).clamp(0.0, track.size.width.0);
        let t = local / track.size.width.0;
        self.min + (self.max - self.min) * t
    }
}

impl Sizable for Slider {
    fn with_size(self, size: ComponentSize) -> Self {
        Slider::with_size(self, size)
    }
}

impl<H: UiHost> Widget<H> for Slider {
    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Generic);
        cx.set_disabled(self.disabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = cx.bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if !self.disabled && (hovered || cx.captured == Some(cx.node)) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }

                    if self.pressed && cx.captured == Some(cx.node) && !self.disabled {
                        let next = self.value_from_position(cx.bounds, position.x);
                        self.set_value(cx.app, next);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left || self.disabled {
                        return;
                    }
                    if !cx.bounds.contains(*position) {
                        return;
                    }
                    self.pressed = true;
                    cx.capture_pointer(cx.node);
                    cx.request_focus(cx.node);
                    let next = self.value_from_position(cx.bounds, position.x);
                    self.set_value(cx.app, next);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up { button, .. } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if self.pressed {
                        self.pressed = false;
                        cx.release_pointer_capture();
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let h = self.resolved.height.0.max(0.0).min(cx.available.height.0);
        Size::new(cx.available.width, Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        let value = self.value(cx.app);
        let t = self.normalized(value);

        let track = self.track_rect(cx.bounds);
        let border_w = Px(self.resolved.border_width.0.max(0.0));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: track,
            background: self.resolved.track_bg,
            border: Edges::all(border_w),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.track_radius),
        });

        let fill_w = Px((track.size.width.0 * t).max(0.0));
        let fill = Rect::new(track.origin, Size::new(fill_w, track.size.height));
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: fill,
            background: self.resolved.track_fill,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(self.resolved.track_radius),
        });

        let thumb = self.thumb_rect(cx.bounds, t);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(2),
            rect: thumb,
            background: self.resolved.thumb,
            border: Edges::all(Px(1.0)),
            border_color: self.resolved.thumb_border,
            corner_radii: Corners::all(Px(thumb.size.width.0 * 0.5)),
        });

        if cx.focus == Some(cx.node) && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
        {
            let focus = self.resolved.focus_ring;
            let inset = Px(1.0);
            let w = (cx.bounds.size.width.0 - inset.0 * 2.0).max(0.0);
            let h = (cx.bounds.size.height.0 - inset.0 * 2.0).max(0.0);
            let rect = Rect::new(
                fret_core::Point::new(
                    Px(cx.bounds.origin.x.0 + inset.0),
                    Px(cx.bounds.origin.y.0 + inset.0),
                ),
                Size::new(Px(w), Px(h)),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: Color { a: 0.0, ..focus },
                border: Edges::all(Px(2.0)),
                border_color: focus,
                corner_radii: Corners::all(cx.theme().metrics.radius_md),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, PathCommand, PathConstraints, PathId, PathMetrics,
        PathService, PathStyle, Point, PointerEvent, Px, Rect, Size, TextBlobId, TextConstraints,
        TextMetrics, TextService, TextStyle,
    };
    use fret_ui::UiTree;

    #[derive(Default)]
    struct FakeText;

    impl TextService for FakeText {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::default(),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeText {
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

    #[test]
    fn slider_drag_updates_model_value() {
        let mut app = App::new();
        let model = app.models_mut().insert(0.0f32);

        let mut ui = UiTree::<App>::new();
        ui.set_window(AppWindowId::default());

        let slider = ui.create_node(Slider::new(model).range(0.0, 1.0));
        ui.set_root(slider);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
        let mut text = FakeText::default();

        ui.layout_all(&mut app, &mut text, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Pointer(PointerEvent::Down {
                position: Point::new(Px(100.0), Px(20.0)),
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
            }),
        );

        ui.dispatch_event(
            &mut app,
            &mut text,
            &Event::Pointer(PointerEvent::Move {
                position: Point::new(Px(180.0), Px(20.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: Modifiers::default(),
            }),
        );

        let v = app.models().get(model).copied().unwrap_or_default();
        assert!(v > 0.5, "expected drag to increase value, got {v}");
    }
}

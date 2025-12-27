use fret_core::{
    AppWindowId, Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Effect, Model};
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};
use std::{collections::HashMap, sync::Arc, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Normal,
    Success,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToastAction {
    pub label: Arc<str>,
    pub command: CommandId,
}

impl ToastAction {
    pub fn new(label: impl Into<Arc<str>>, command: CommandId) -> Self {
        Self {
            label: label.into(),
            command,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastRequest {
    pub kind: ToastKind,
    pub title: Arc<str>,
    pub description: Option<Arc<str>>,
    pub action: Option<ToastAction>,
    pub duration: Duration,
}

impl ToastRequest {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            kind: ToastKind::Normal,
            title: title.into(),
            description: None,
            action: None,
            duration: Duration::from_secs(4),
        }
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn action(mut self, action: ToastAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

#[derive(Debug)]
struct ToastEntry {
    id: u64,
    request: ToastRequest,
    timer: Option<fret_core::TimerToken>,
}

#[derive(Debug)]
struct WindowToasts {
    serial: u64,
    items: Vec<ToastEntry>,
}

#[derive(Default)]
pub struct ToastService {
    next_id: u64,
    touch_counter: u64,
    by_window: HashMap<AppWindowId, WindowToasts>,
    serial_model: Option<Model<u64>>,
}

impl ToastService {
    fn ensure_serial_model<H: UiHost>(&mut self, app: &mut H) -> Model<u64> {
        if let Some(model) = self.serial_model {
            return model;
        }
        let model = app.models_mut().insert(0u64);
        self.serial_model = Some(model);
        model
    }

    pub fn serial_model(&self) -> Option<Model<u64>> {
        self.serial_model
    }

    pub fn touch_counter(&self) -> u64 {
        self.touch_counter
    }

    fn items(&self, window: AppWindowId) -> Option<(u64, &[ToastEntry])> {
        let entry = self.by_window.get(&window)?;
        Some((self.touch_counter, entry.items.as_slice()))
    }

    pub fn count(&self, window: AppWindowId) -> usize {
        self.by_window
            .get(&window)
            .map(|w| w.items.len())
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub fn debug_first_timer(&self, window: AppWindowId) -> Option<fret_core::TimerToken> {
        self.by_window
            .get(&window)
            .and_then(|w| w.items.first())
            .and_then(|t| t.timer)
    }

    pub fn push<H: UiHost>(&mut self, app: &mut H, window: AppWindowId, request: ToastRequest) {
        self.touch_counter = self.touch_counter.saturating_add(1);

        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;

        let token = if request.duration.as_millis() == 0 {
            None
        } else {
            let token = app.next_timer_token();
            app.push_effect(Effect::SetTimer {
                window: Some(window),
                token,
                after: request.duration,
                repeat: None,
            });
            Some(token)
        };

        let toasts = self.by_window.entry(window).or_insert(WindowToasts {
            serial: 0,
            items: Vec::new(),
        });
        toasts.items.push(ToastEntry {
            id,
            request,
            timer: token,
        });

        // Shadcn/Sonner defaults to a small stack; keep it bounded.
        const MAX_TOASTS: usize = 3;
        while toasts.items.len() > MAX_TOASTS {
            if let Some(removed) = toasts.items.first().and_then(|e| e.timer) {
                app.push_effect(Effect::CancelTimer { token: removed });
            }
            toasts.items.remove(0);
        }

        toasts.serial = toasts.serial.saturating_add(1);
        let model = self.ensure_serial_model(app);
        let _ = app.models_mut().update(model, |v| *v = self.touch_counter);

        app.request_redraw(window);
    }

    pub fn dismiss<H: UiHost>(&mut self, app: &mut H, window: AppWindowId, id: u64) -> bool {
        let Some(toasts) = self.by_window.get_mut(&window) else {
            return false;
        };
        let idx = toasts.items.iter().position(|t| t.id == id);
        let Some(idx) = idx else {
            return false;
        };
        if let Some(token) = toasts.items[idx].timer {
            app.push_effect(Effect::CancelTimer { token });
        }
        toasts.items.remove(idx);
        toasts.serial = toasts.serial.saturating_add(1);
        self.touch_counter = self.touch_counter.saturating_add(1);
        let model = self.ensure_serial_model(app);
        let _ = app.models_mut().update(model, |v| *v = self.touch_counter);
        app.request_redraw(window);
        true
    }

    pub fn handle_timer<H: UiHost>(
        &mut self,
        app: &mut H,
        window: AppWindowId,
        token: fret_core::TimerToken,
    ) -> bool {
        let Some(toasts) = self.by_window.get_mut(&window) else {
            return false;
        };
        let idx = toasts.items.iter().position(|t| t.timer == Some(token));
        let Some(idx) = idx else {
            return false;
        };
        toasts.items.remove(idx);
        toasts.serial = toasts.serial.saturating_add(1);
        self.touch_counter = self.touch_counter.saturating_add(1);
        let model = self.ensure_serial_model(app);
        let _ = app.models_mut().update(model, |v| *v = self.touch_counter);
        app.request_redraw(window);
        true
    }
}

#[derive(Debug)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug)]
struct PreparedToast {
    id: u64,
    kind: ToastKind,
    bounds: Rect,
    close_bounds: Rect,
    action_bounds: Option<Rect>,
    action_command: Option<CommandId>,
    title: PreparedText,
    description: Option<PreparedText>,
    action_label: Option<PreparedText>,
}

#[derive(Debug, Clone)]
pub struct ToastStyle {
    pub width: Px,
    pub margin: Px,
    pub gap: Px,
    pub padding_x: Px,
    pub padding_y: Px,
    pub corner_radius: Px,
    pub border: Edges,
    pub background: Color,
    pub border_color: Color,
    pub title_color: Color,
    pub description_color: Color,
    pub title_style: TextStyle,
    pub description_style: TextStyle,
    pub action_style: TextStyle,
    pub action_color: Color,
    pub close_style: TextStyle,
    pub close_color: Color,
    pub accent_success: Color,
    pub accent_info: Color,
    pub accent_warning: Color,
    pub accent_error: Color,
    pub accent_width: Px,
}

impl Default for ToastStyle {
    fn default() -> Self {
        Self {
            width: Px(360.0),
            margin: Px(16.0),
            gap: Px(10.0),
            padding_x: Px(14.0),
            padding_y: Px(12.0),
            corner_radius: Px(10.0),
            border: Edges::all(Px(1.0)),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            title_color: Color::TRANSPARENT,
            description_color: Color::TRANSPARENT,
            title_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            description_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
                ..Default::default()
            },
            action_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
                ..Default::default()
            },
            action_color: Color::TRANSPARENT,
            close_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(14.0),
                ..Default::default()
            },
            close_color: Color::TRANSPARENT,
            accent_success: Color::TRANSPARENT,
            accent_info: Color::TRANSPARENT,
            accent_warning: Color::TRANSPARENT,
            accent_error: Color::TRANSPARENT,
            accent_width: Px(3.0),
        }
    }
}

pub struct ToastOverlay {
    style: ToastStyle,
    last_theme_revision: Option<u64>,
    last_serial: Option<u64>,
    last_bounds: Rect,
    last_scale_factor_bits: Option<u32>,
    prepared: Vec<PreparedToast>,
    hovered_toast: Option<u64>,
}

impl ToastOverlay {
    pub fn new() -> Self {
        Self {
            style: ToastStyle::default(),
            last_theme_revision: None,
            last_serial: None,
            last_bounds: Rect::default(),
            last_scale_factor_bits: None,
            prepared: Vec::new(),
            hovered_toast: None,
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        self.style.title_style = crate::Size::Small.control_text_style(theme);
        self.style.description_style = crate::Size::XSmall.control_text_style(theme);
        self.style.action_style = crate::Size::XSmall.control_text_style(theme);
        self.style.close_style = crate::Size::Large.control_text_style(theme);

        self.style.background = theme.colors.menu_background;
        self.style.border_color = theme.colors.menu_border;
        self.style.corner_radius = theme.metrics.radius_lg;
        self.style.padding_x = theme.metrics.padding_md;
        self.style.padding_y = theme.metrics.padding_sm;
        self.style.title_color = theme.colors.text_primary;
        self.style.description_color = theme.colors.text_muted;
        self.style.action_color = theme.colors.accent;
        self.style.close_color = theme.colors.text_muted;

        self.style.accent_success = Color {
            a: 1.0,
            ..theme.colors.accent
        };
        self.style.accent_info = Color {
            a: 1.0,
            ..theme.colors.accent
        };
        self.style.accent_warning = Color {
            r: theme.colors.accent.r,
            g: theme.colors.accent.g,
            b: theme.colors.accent.b,
            a: 0.8,
        };
        self.style.accent_error = Color {
            r: theme.colors.accent.r,
            g: theme.colors.accent.g * 0.4,
            b: theme.colors.accent.b * 0.4,
            a: 1.0,
        };
    }

    fn cleanup(&mut self, services: &mut dyn fret_core::UiServices) {
        for toast in self.prepared.drain(..) {
            services.text().release(toast.title.blob);
            if let Some(desc) = toast.description {
                services.text().release(desc.blob);
            }
            if let Some(action) = toast.action_label {
                services.text().release(action.blob);
            }
        }
    }

    fn hit_test_toast(&self, position: Point) -> Option<u64> {
        self.prepared
            .iter()
            .find(|t| t.bounds.contains(position))
            .map(|t| t.id)
    }

    fn hit_test_close(&self, position: Point) -> Option<u64> {
        self.prepared
            .iter()
            .find(|t| t.close_bounds.contains(position))
            .map(|t| t.id)
    }

    fn hit_test_action(&self, position: Point) -> Option<(u64, CommandId)> {
        for t in &self.prepared {
            if let Some(bounds) = t.action_bounds
                && bounds.contains(position)
                && let Some(action) = t.action_command.clone()
            {
                return Some((t.id, action));
            }
        }
        None
    }
}

impl Default for ToastOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for ToastOverlay {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.hit_test_toast(position).is_some()
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        // Ensure the service has a stable model so the overlay repaints as soon as toasts change.
        let model = cx
            .app
            .with_global_mut(ToastService::default, |service, app| {
                service.ensure_serial_model(app)
            });
        cx.observe_model(model, Invalidation::Paint);
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        self.sync_style_from_theme(cx.theme());

        if let Event::Timer { token } = event {
            let handled = cx.app.with_global_mut(ToastService::default, |svc, app| {
                svc.handle_timer(app, window, *token)
            });
            if handled {
                cx.stop_propagation();
            }
            return;
        }

        // Hover state.
        if let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event {
            let next = self.hit_test_toast(*position);
            if next != self.hovered_toast {
                self.hovered_toast = next;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
            return;
        }

        // Click dismissal + action.
        if let Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) = event
            && *button == MouseButton::Left
        {
            if let Some(id) = self.hit_test_close(*position) {
                cx.app.with_global_mut(ToastService::default, |svc, app| {
                    let _ = svc.dismiss(app, window, id);
                });
                cx.stop_propagation();
            }

            if let Some((_id, command)) = self.hit_test_action(*position) {
                if let Some(id) = self.hit_test_toast(*position) {
                    cx.app.with_global_mut(ToastService::default, |svc, app| {
                        let _ = svc.dismiss(app, window, id);
                    });
                }
                cx.dispatch_command(command);
                cx.stop_propagation();
            }
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        let Some(window) = cx.window else {
            self.cleanup(cx.services);
            self.last_serial = None;
            self.last_scale_factor_bits = None;
            return;
        };

        let Some(service) = cx.app.global::<ToastService>() else {
            self.cleanup(cx.services);
            self.last_serial = None;
            return;
        };

        let Some((serial, items)) = service.items(window) else {
            self.cleanup(cx.services);
            self.last_serial = None;
            self.last_scale_factor_bits = None;
            return;
        };

        // Rebuild prepared text when the toast list changes or scale factor changes.
        let scale_bits = cx.scale_factor.to_bits();
        let rebuild = self.last_serial != Some(serial)
            || self.last_scale_factor_bits != Some(scale_bits)
            || self.prepared.is_empty();
        self.last_serial = Some(serial);
        self.last_scale_factor_bits = Some(scale_bits);
        if rebuild {
            self.cleanup(cx.services);
            self.prepared.clear();

            let pad_x = self.style.padding_x.0.max(0.0);
            let pad_y = self.style.padding_y.0.max(0.0);
            let gap = self.style.gap.0.max(0.0);
            let margin = self.style.margin.0.max(0.0);

            let w = self
                .style
                .width
                .0
                .min((cx.bounds.size.width.0 - margin * 2.0).max(0.0))
                .max(0.0);
            let max_text_w = Px((w - pad_x * 2.0).max(0.0));

            let mut y = cx.bounds.origin.y.0 + cx.bounds.size.height.0 - margin;
            for entry in items.iter().rev() {
                let title_constraints = TextConstraints {
                    max_width: Some(max_text_w),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor,
                };
                let (title_blob, title_metrics) = cx.services.text().prepare(
                    entry.request.title.as_ref(),
                    self.style.title_style,
                    title_constraints,
                );
                let title = PreparedText {
                    blob: title_blob,
                    metrics: title_metrics,
                };

                let mut desc_prepared = None;
                if let Some(desc) = entry.request.description.as_ref() {
                    let constraints = TextConstraints {
                        max_width: Some(max_text_w),
                        wrap: TextWrap::Word,
                        overflow: TextOverflow::Clip,
                        scale_factor: cx.scale_factor,
                    };
                    let (blob, metrics) = cx.services.text().prepare(
                        desc.as_ref(),
                        self.style.description_style,
                        constraints,
                    );
                    desc_prepared = Some(PreparedText { blob, metrics });
                }

                let mut action_prepared = None;
                let mut action_bounds = None;
                let mut action_command = None;
                if let Some(action) = entry.request.action.as_ref() {
                    let constraints = TextConstraints {
                        max_width: None,
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        scale_factor: cx.scale_factor,
                    };
                    let (blob, metrics) = cx.services.text().prepare(
                        action.label.as_ref(),
                        self.style.action_style,
                        constraints,
                    );
                    action_command = Some(action.command.clone());
                    action_prepared = Some(PreparedText { blob, metrics });
                }

                let content_h = title.metrics.size.height.0
                    + desc_prepared
                        .as_ref()
                        .map(|d| gap + d.metrics.size.height.0)
                        .unwrap_or(0.0);
                let h = (content_h + pad_y * 2.0).max(0.0);
                y -= h;

                let x = cx.bounds.origin.x.0 + cx.bounds.size.width.0 - margin - w;
                let bounds = Rect::new(
                    Point::new(
                        Px(x.max(cx.bounds.origin.x.0)),
                        Px(y.max(cx.bounds.origin.y.0)),
                    ),
                    Size::new(Px(w), Px(h)),
                );

                let close_w = Px(20.0);
                let close_bounds = Rect::new(
                    Point::new(
                        Px(bounds.origin.x.0 + bounds.size.width.0 - close_w.0 - pad_x),
                        bounds.origin.y,
                    ),
                    Size::new(close_w, Px(20.0)),
                );

                if let Some(action_label) = action_prepared.as_ref() {
                    let aw = action_label.metrics.size.width.0.max(0.0) + pad_x;
                    let ah = Px((action_label.metrics.size.height.0 + pad_y).max(0.0));
                    let x = bounds.origin.x.0 + bounds.size.width.0 - pad_x - aw;
                    let y = bounds.origin.y.0 + bounds.size.height.0 - pad_y - ah.0;
                    action_bounds =
                        Some(Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(aw), ah)));
                }

                self.prepared.push(PreparedToast {
                    id: entry.id,
                    kind: entry.request.kind,
                    bounds,
                    close_bounds,
                    action_bounds,
                    action_command,
                    title,
                    description: desc_prepared,
                    action_label: action_prepared,
                });

                y -= self.style.gap.0.max(0.0);
            }
        }

        for toast in self.prepared.iter() {
            let accent = match toast.kind {
                ToastKind::Normal => Color::TRANSPARENT,
                ToastKind::Success => self.style.accent_success,
                ToastKind::Info => self.style.accent_info,
                ToastKind::Warning => self.style.accent_warning,
                ToastKind::Error => self.style.accent_error,
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(11_500),
                rect: toast.bounds,
                background: self.style.background,
                border: self.style.border,
                border_color: self.style.border_color,
                corner_radii: Corners::all(self.style.corner_radius),
            });

            if accent.a > 0.0 {
                let stripe = Rect::new(
                    toast.bounds.origin,
                    Size::new(self.style.accent_width, toast.bounds.size.height),
                );
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(11_501),
                    rect: stripe,
                    background: accent,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let pad_x = self.style.padding_x.0.max(0.0);
            let pad_y = self.style.padding_y.0.max(0.0);
            let gap = self.style.gap.0.max(0.0);

            let title_x = Px(toast.bounds.origin.x.0 + pad_x);
            let title_y = Px(toast.bounds.origin.y.0 + pad_y + toast.title.metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11_510),
                origin: Point::new(title_x, title_y),
                text: toast.title.blob,
                color: self.style.title_color,
            });

            if let Some(desc) = toast.description.as_ref() {
                let y = toast.bounds.origin.y.0
                    + pad_y
                    + toast.title.metrics.size.height.0
                    + gap
                    + desc.metrics.baseline.0;
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(11_511),
                    origin: Point::new(title_x, Px(y)),
                    text: desc.blob,
                    color: self.style.description_color,
                });
            }

            if let Some(action) = toast.action_label.as_ref()
                && let Some(bounds) = toast.action_bounds
            {
                let x = Px(bounds.origin.x.0 + self.style.padding_x.0 * 0.5);
                let y = Px(bounds.origin.y.0
                    + (bounds.size.height.0 - action.metrics.size.height.0) * 0.5
                    + action.metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(11_512),
                    origin: Point::new(x, y),
                    text: action.blob,
                    color: self.style.action_color,
                });
            }

            // Close "×" glyph (best-effort, no icon dependency).
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let (close_blob, close_metrics) =
                cx.services
                    .text()
                    .prepare("×", self.style.close_style, constraints);
            let close_x = Px(toast.close_bounds.origin.x.0
                + (toast.close_bounds.size.width.0 - close_metrics.size.width.0) * 0.5);
            let close_y = Px(toast.close_bounds.origin.y.0 + pad_y + close_metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(11_513),
                origin: Point::new(close_x, close_y),
                text: close_blob,
                color: self.style.close_color,
            });
            cx.services.text().release(close_blob);
        }
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.cleanup(services);
        self.last_serial = None;
        self.last_theme_revision = None;
        self.last_bounds = Rect::default();
        self.last_scale_factor_bits = None;
        self.hovered_toast = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_ui::{
        UiHost,
        widget::{LayoutCx, PaintCx, Widget},
    };

    #[derive(Default)]
    struct FakeTextService;

    #[derive(Debug, Default)]
    struct TestContainer;

    impl<H: UiHost> Widget<H> for TestContainer {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                cx.layout_in(child, cx.bounds);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in cx.children {
                cx.paint(child, cx.bounds);
            }
        }
    }

    impl fret_core::TextService for FakeTextService {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeTextService {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeTextService {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    #[test]
    fn toast_timer_dismissal_is_routed_without_focus() {
        let window = AppWindowId::default();

        let mut app = App::new();
        let mut ui: fret_ui::UiTree<App> = fret_ui::UiTree::new();
        ui.set_window(window);

        let base = ui.create_node(TestContainer);
        ui.set_root(base);

        let toast_node = ui.create_node(ToastOverlay::new());
        let toast_layer = ui.push_overlay_root_ex(toast_node, false, true);
        ui.set_layer_wants_timer_events(toast_layer, true);

        let token = app.with_global_mut(ToastService::default, |svc, app| {
            svc.push(app, window, ToastRequest::new("Hello toast"));
            svc.debug_first_timer(window)
                .expect("toast should schedule a timer")
        });

        let mut text = FakeTextService;
        ui.layout_all(
            &mut app,
            &mut text,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(200.0), Px(200.0)),
            ),
            1.0,
        );

        assert_eq!(
            app.global::<ToastService>()
                .map(|s| s.count(window))
                .unwrap_or(0),
            1
        );

        ui.dispatch_event(&mut app, &mut text, &Event::Timer { token });

        assert_eq!(
            app.global::<ToastService>()
                .map(|s| s.count(window))
                .unwrap_or(0),
            0
        );
    }
}

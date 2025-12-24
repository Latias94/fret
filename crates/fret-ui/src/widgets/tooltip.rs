use crate::{
    Theme, UiHost,
    widget::{Invalidation, LayoutCx, PaintCx, Widget},
};
use fret_core::{
    Color, Corners, DrawOrder, Edges, NodeId, Point, Px, Rect, SceneOp, Size, TextConstraints,
    TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::Model;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct TooltipRequest {
    pub owner: NodeId,
    pub anchor: Rect,
    pub text: Arc<str>,
}

#[derive(Debug)]
struct TooltipEntry {
    window_serial: u64,
    request: TooltipRequest,
}

#[derive(Default)]
pub struct TooltipService {
    next_serial: u64,
    touch_counter: u64,
    by_window: HashMap<fret_core::AppWindowId, TooltipEntry>,
    serial_model: Option<Model<u64>>,
}

impl TooltipService {
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

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &TooltipRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.window_serial, &entry.request))
    }

    pub fn set_request<H: UiHost>(
        &mut self,
        app: &mut H,
        window: fret_core::AppWindowId,
        request: TooltipRequest,
    ) {
        self.touch_counter = self.touch_counter.saturating_add(1);

        if self.by_window.get(&window).is_some_and(|e| {
            e.request.owner == request.owner
                && e.request.text == request.text
                && e.request.anchor == request.anchor
        }) {
            return;
        }

        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window.insert(
            window,
            TooltipEntry {
                window_serial: serial,
                request,
            },
        );

        let model = self.ensure_serial_model(app);
        let _ = app.models_mut().update(model, |v| *v = serial);
    }

    pub fn clear_request<H: UiHost>(&mut self, app: &mut H, window: fret_core::AppWindowId) {
        if self.by_window.remove(&window).is_none() {
            return;
        }
        self.next_serial = self.next_serial.saturating_add(1);
        let model = self.ensure_serial_model(app);
        let _ = app.models_mut().update(model, |v| *v = self.next_serial);
    }
}

#[derive(Debug)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
    scale_factor_bits: u32,
    text: Arc<str>,
}

#[derive(Debug)]
pub struct TooltipOverlay {
    style: TooltipStyle,
    last_serial: Option<u64>,
    last_theme_revision: Option<u64>,
    prepared: Option<PreparedText>,
}

#[derive(Debug, Clone)]
pub struct TooltipStyle {
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    pub text_color: Color,
    pub text_style: TextStyle,
    pub padding_x: Px,
    pub padding_y: Px,
    pub max_width: Px,
    pub offset: Px,
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 0.94,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.40,
            },
            corner_radii: Corners::all(Px(8.0)),
            text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(12.0),
            },
            padding_x: Px(10.0),
            padding_y: Px(8.0),
            max_width: Px(320.0),
            offset: Px(8.0),
        }
    }
}

impl TooltipOverlay {
    pub fn new() -> Self {
        Self {
            style: TooltipStyle::default(),
            last_serial: None,
            last_theme_revision: None,
            prepared: None,
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let radius = theme.metrics.radius_sm;
        self.style.background = Color {
            a: 0.94,
            ..theme.colors.menu_background
        };
        self.style.border_color = theme.colors.menu_border;
        self.style.corner_radii = Corners::all(radius);
        self.style.text_color = theme.colors.text_primary;
        self.style.padding_x = theme.metrics.padding_sm;
        self.style.padding_y = theme.metrics.padding_sm;
    }

    fn cleanup(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(p) = self.prepared.take() {
            text.release(p.blob);
        }
    }

    fn ensure_prepared<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        serial: u64,
        request: &TooltipRequest,
    ) -> Option<TextMetrics> {
        let scale_bits = cx.scale_factor.to_bits();
        let needs_prepare = self.prepared.is_none()
            || self.last_serial != Some(serial)
            || self
                .prepared
                .as_ref()
                .is_some_and(|p| p.scale_factor_bits != scale_bits || p.text != request.text);
        if !needs_prepare {
            return self.prepared.as_ref().map(|p| p.metrics);
        }

        self.cleanup(cx.text);
        self.last_serial = Some(serial);

        let constraints = TextConstraints {
            max_width: Some(self.style.max_width),
            wrap: TextWrap::Word,
            scale_factor: cx.scale_factor,
        };
        let (blob, metrics) =
            cx.text
                .prepare(request.text.as_ref(), self.style.text_style, constraints);
        self.prepared = Some(PreparedText {
            blob,
            metrics,
            scale_factor_bits: scale_bits,
            text: request.text.clone(),
        });
        Some(metrics)
    }

    fn compute_bounds(&self, request: &TooltipRequest, screen: Size, content: Size) -> Rect {
        let pad_x = self.style.padding_x.0.max(0.0);
        let pad_y = self.style.padding_y.0.max(0.0);
        let w = (content.width.0 + pad_x * 2.0).max(0.0);
        let h = (content.height.0 + pad_y * 2.0).max(0.0);

        let mut x = request.anchor.origin.x.0;
        let mut y = request.anchor.origin.y.0 + request.anchor.size.height.0 + self.style.offset.0;

        // If it doesn't fit below, try above.
        if y + h > screen.height.0 {
            y = request.anchor.origin.y.0 - h - self.style.offset.0;
        }

        x = x.clamp(0.0, (screen.width.0 - w).max(0.0));
        y = y.clamp(0.0, (screen.height.0 - h).max(0.0));

        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }
}

impl Default for TooltipOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for TooltipOverlay {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        self.cleanup(text);
        self.last_serial = None;
        self.last_theme_revision = None;
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        // Ensure the service has a stable model so we can observe it even before the first
        // tooltip request is issued (otherwise the overlay may never repaint).
        let model = cx
            .app
            .with_global_mut(TooltipService::default, |service, app| {
                service.ensure_serial_model(app)
            });
        cx.observe_model(model, Invalidation::Paint);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());

        let Some(window) = cx.window else {
            self.cleanup(cx.text);
            self.last_serial = None;
            return;
        };

        let Some(service) = cx.app.global::<TooltipService>() else {
            self.cleanup(cx.text);
            self.last_serial = None;
            return;
        };

        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            self.cleanup(cx.text);
            self.last_serial = None;
            return;
        };

        let Some(metrics) = self.ensure_prepared(cx, serial, &request) else {
            return;
        };
        let Some(prepared) = self.prepared.as_ref() else {
            return;
        };

        let bubble = self.compute_bounds(&request, cx.bounds.size, metrics.size);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(12_000),
            rect: bubble,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        let origin = Point::new(
            Px(bubble.origin.x.0 + self.style.padding_x.0),
            Px(bubble.origin.y.0 + self.style.padding_y.0 + metrics.baseline.0),
        );
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(12_001),
            origin,
            text: prepared.blob,
            color: self.style.text_color,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiTree, test_host::TestHost};
    use fret_core::{
        AppWindowId, Point, Px, Rect, Size, TextBlobId, TextConstraints, TextMetrics, TextStyle,
    };

    #[derive(Default)]
    struct FakeTextService;

    impl fret_core::TextService for FakeTextService {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::default(),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    #[test]
    fn tooltip_overlay_layout_initializes_and_observes_service_model() {
        let mut app = TestHost::new();
        let mut ui = UiTree::<TestHost>::new();
        ui.set_window(AppWindowId::default());

        let root = ui.create_node(TooltipOverlay::new());
        ui.set_root(root);

        let mut text = FakeTextService::default();
        ui.layout_all(
            &mut app,
            &mut text,
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(200.0), Px(120.0)),
            ),
            1.0,
        );

        let svc = app
            .global::<TooltipService>()
            .expect("tooltip service exists");
        assert!(svc.serial_model().is_some());
    }
}

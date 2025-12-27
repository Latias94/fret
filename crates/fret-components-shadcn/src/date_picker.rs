use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{CommandId, Model};
use fret_ui::focus_visible::is_focus_visible;
use fret_ui::paint::paint_focus_ring;
use fret_ui::widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget};
use fret_ui::{Theme, UiHost};

use fret_components_ui::popover_surface::open_popover_surface;
use fret_components_ui::{
    MetricRef, PopoverSurfaceAlign, PopoverSurfaceRequest, PopoverSurfaceService,
    PopoverSurfaceSide, Size as ComponentSize,
};

use crate::Date;

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: fret_core::TextMetrics,
}

#[derive(Debug, Clone, Copy)]
struct ResolvedStyle {
    padding_x: Px,
    padding_y: Px,
    min_height: Px,
    radius: Px,
    border_width: Px,
    text_size: Px,
    bg: Color,
    bg_hover: Color,
    bg_active: Color,
    border: Color,
    fg: Color,
    fg_disabled: Color,
}

impl Default for ResolvedStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(12.0),
            padding_y: Px(6.0),
            min_height: Px(32.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            text_size: Px(13.0),
            bg: Color::TRANSPARENT,
            bg_hover: Color::TRANSPARENT,
            bg_active: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            fg: Color::TRANSPARENT,
            fg_disabled: Color::TRANSPARENT,
        }
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn format_date(d: Date) -> Arc<str> {
    Arc::from(format!("{:04}-{:02}-{:02}", d.year, d.month, d.day))
}

fn resolve_outline_button_colors(theme: &Theme) -> (Color, Color, Color, Color, Color) {
    let transparent = Color::TRANSPARENT;
    let fg = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);
    let border = theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border);
    let hover_bg = theme
        .color_by_key("accent")
        .or_else(|| theme.color_by_key("accent.background"))
        .unwrap_or(theme.colors.hover_background);
    let active_bg = alpha_mul(hover_bg, 0.8);
    (
        transparent,
        alpha_mul(hover_bg, 0.35),
        alpha_mul(active_bg, 0.55),
        border,
        fg,
    )
}

/// shadcn/ui `DatePicker` (first pass).
///
/// This is a trigger button that opens a `PopoverSurfaceOverlay` anchored to its bounds.
///
/// Notes:
/// - The popover content is a UI node specified by `content_node` (typically a `Calendar`),
///   installed under `WindowOverlays::popover_surface_node()`.
pub struct DatePicker {
    model: Model<Option<Date>>,
    content_node: fret_core::NodeId,
    placeholder: Arc<str>,
    disabled: bool,

    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    prepared: Option<PreparedText>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedStyle,
}

impl DatePicker {
    pub fn new(model: Model<Option<Date>>, content_node: fret_core::NodeId) -> Self {
        Self {
            model,
            content_node,
            placeholder: Arc::from("Pick a date"),
            disabled: false,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            prepared: None,
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            last_theme_revision: None,
            resolved: ResolvedStyle::default(),
        }
    }

    pub fn placeholder(mut self, text: impl Into<Arc<str>>) -> Self {
        self.placeholder = text.into();
        self.prepared = None;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn read_value<H: UiHost>(&self, app: &H) -> Option<Date> {
        app.models().get(self.model).copied().flatten()
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let size = ComponentSize::Medium;
        let default_px = size.button_px(theme);
        let default_py = size.button_py(theme);
        let default_h = size.button_h(theme);
        let default_radius = size.control_radius(theme);

        let padding_x = theme
            .metric_by_key("component.button.padding_x")
            .unwrap_or(default_px);
        let padding_y = theme
            .metric_by_key("component.button.padding_y")
            .unwrap_or(default_py);
        let min_height = theme
            .metric_by_key("component.button.min_height")
            .unwrap_or(default_h);
        let radius = theme
            .metric_by_key("component.button.radius")
            .unwrap_or(default_radius);
        let border_width = MetricRef::Px(Px(1.0)).resolve(theme);

        let text_size = theme
            .metric_by_key("component.button.text_px")
            .unwrap_or_else(|| size.control_text_px(theme));

        let fg_disabled = theme
            .color_by_key("component.button.fg_disabled")
            .unwrap_or(theme.colors.text_disabled);

        let (bg, bg_hover, bg_active, border, fg) = resolve_outline_button_colors(theme);

        self.resolved = ResolvedStyle {
            padding_x,
            padding_y,
            min_height: Px(min_height.0.max(0.0)),
            radius: Px(radius.0.max(0.0)),
            border_width: Px(border_width.0.max(0.0)),
            text_size,
            bg,
            bg_hover,
            bg_active,
            border,
            fg,
            fg_disabled,
        };
    }

    fn is_open<H: UiHost>(
        &self,
        app: &mut H,
        window: fret_core::AppWindowId,
        owner: fret_core::NodeId,
    ) -> bool {
        app.global::<PopoverSurfaceService>()
            .and_then(|s| s.request(window))
            .is_some_and(|(_, r)| r.owner == owner)
    }
}

impl<H: UiHost> Widget<H> for DatePicker {
    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, bounds: Rect, position: Point) -> bool {
        if self.disabled {
            return false;
        }
        bounds.contains(position)
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
        cx.set_disabled(self.disabled);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(self.model, Invalidation::Layout);
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;

        let text = self
            .read_value(cx.app)
            .map(format_date)
            .unwrap_or_else(|| self.placeholder.clone());

        let scale_bits = cx.scale_factor.to_bits();
        let theme_rev = cx.theme().revision();
        if self.prepared_scale_factor_bits != Some(scale_bits)
            || self.prepared_theme_revision != Some(theme_rev)
        {
            self.prepared = None;
            self.prepared_scale_factor_bits = Some(scale_bits);
            self.prepared_theme_revision = Some(theme_rev);
        }

        if self.prepared.is_none() {
            let style = TextStyle {
                font: fret_core::FontId::default(),
                size: self.resolved.text_size,
                ..Default::default()
            };
            let constraints = TextConstraints {
                max_width: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) = cx.services.text().prepare(&text, style, constraints);
            self.prepared = Some(PreparedText { blob, metrics });
        }

        let metrics = self.prepared.as_ref().expect("prepared").metrics;
        let w = metrics.size.width.0 + self.resolved.padding_x.0 * 2.0;
        let h = (metrics.size.height.0 + self.resolved.padding_y.0 * 2.0)
            .max(self.resolved.min_height.0);
        Size::new(Px(w.max(0.0)), Px(h.max(0.0)))
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        if self.disabled {
            return;
        }

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.last_bounds.contains(*position);
                    if hovered != self.hovered {
                        self.hovered = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if hovered || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if !self.last_bounds.contains(*position) {
                        return;
                    }
                    self.pressed = true;
                    cx.capture_pointer(cx.node);
                    cx.request_focus(cx.node);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Up {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    cx.release_pointer_capture();
                    let was_pressed = std::mem::take(&mut self.pressed);
                    let hovered = self.last_bounds.contains(*position);
                    self.hovered = hovered;

                    if was_pressed && hovered {
                        if self.is_open(cx.app, window, cx.node) {
                            cx.dispatch_command(CommandId::from("popover_surface.close"));
                        } else {
                            open_popover_surface(
                                cx,
                                window,
                                PopoverSurfaceRequest::new(
                                    cx.node,
                                    self.last_bounds,
                                    self.content_node,
                                )
                                .side(PopoverSurfaceSide::Bottom)
                                .align(PopoverSurfaceAlign::Start),
                            );
                        }
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                _ => {}
            },
            Event::KeyDown {
                key,
                modifiers,
                repeat,
            } => {
                if *repeat {
                    return;
                }
                if modifiers.ctrl || modifiers.meta || modifiers.alt {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space) {
                    return;
                }
                if !self.pressed {
                    self.pressed = true;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                cx.stop_propagation();
            }
            Event::KeyUp { key, modifiers } => {
                if modifiers.ctrl || modifiers.meta || modifiers.alt {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space) {
                    return;
                }

                let was_pressed = std::mem::take(&mut self.pressed);
                if !was_pressed {
                    return;
                }

                if self.is_open(cx.app, window, cx.node) {
                    cx.dispatch_command(CommandId::from("popover_surface.close"));
                } else {
                    open_popover_surface(
                        cx,
                        window,
                        PopoverSurfaceRequest::new(cx.node, self.last_bounds, self.content_node)
                            .side(PopoverSurfaceSide::Bottom)
                            .align(PopoverSurfaceAlign::Start),
                    );
                }

                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(self.model, Invalidation::Paint);
        self.sync_style_from_theme(cx.theme());

        let bg = if self.pressed {
            self.resolved.bg_active
        } else if self.hovered {
            self.resolved.bg_hover
        } else {
            self.resolved.bg
        };
        let fg = if self.disabled {
            self.resolved.fg_disabled
        } else {
            self.resolved.fg
        };

        let rect = cx.bounds;
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: bg,
            border: Edges::all(self.resolved.border_width),
            border_color: self.resolved.border,
            corner_radii: Corners::all(self.resolved.radius),
        });

        let Some(prepared) = self.prepared.as_ref() else {
            return;
        };
        let x = rect.origin.x.0 + self.resolved.padding_x.0;
        let top = rect.origin.y.0 + (rect.size.height.0 - prepared.metrics.size.height.0) * 0.5;
        let y = top + prepared.metrics.baseline.0;
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(1),
            origin: Point::new(Px(x), Px(y)),
            text: prepared.blob,
            color: fg,
        });

        if cx.focus == Some(cx.node) && is_focus_visible(cx.app, cx.window) {
            let ring_width = cx
                .theme()
                .metric_by_key("component.ring.width")
                .unwrap_or(Px(2.0));
            let ring_offset = cx
                .theme()
                .metric_by_key("component.ring.offset")
                .unwrap_or(Px(2.0));
            let ring_color = cx
                .theme()
                .color_by_key("ring")
                .unwrap_or(cx.theme().colors.focus_ring);
            let ring_offset_color = cx
                .theme()
                .color_by_key("ring-offset-background")
                .unwrap_or(cx.theme().colors.surface_background);

            paint_focus_ring(
                cx.scene,
                DrawOrder(2),
                rect,
                fret_ui::element::RingStyle {
                    placement: fret_ui::element::RingPlacement::Outset,
                    width: ring_width,
                    offset: ring_offset,
                    color: ring_color,
                    offset_color: Some(ring_offset_color),
                    corner_radii: Corners::all(self.resolved.radius),
                },
            );
        }
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(prepared) = self.prepared.take() {
            services.text().release(prepared.blob);
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_components_ui::{PopoverSurfaceRequest, PopoverSurfaceService};
    use fret_core::{
        AppWindowId, Modifiers, MouseButton, NodeId, PathCommand, PathConstraints, PathId,
        PathMetrics, PathService, PathStyle, Point, Px, Rect, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService, TextStyle,
    };
    use fret_runtime::{Effect, InputContext, Platform};
    use fret_ui::widget::EventCx;

    #[derive(Default)]
    struct FakeServices(());

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
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
            false
        }
    }

    #[test]
    fn keyboard_activation_opens_and_toggles_close() {
        let mut host = TestHost::default();
        let window = AppWindowId::default();

        let model = host.models_mut().insert(None::<Date>);
        let content_node = NodeId::default();
        let mut picker = DatePicker::new(model, content_node);
        picker.last_bounds = Rect::new(
            Point::new(Px(10.0), Px(10.0)),
            Size::new(Px(120.0), Px(32.0)),
        );

        let node = NodeId::default();
        let mut services = FakeServices::default();
        {
            let mut cx = EventCx {
                app: &mut host,
                services: &mut services,
                node,
                window: Some(window),
                input_ctx: InputContext {
                    platform: Platform::Linux,
                    caps: fret_core::PlatformCapabilities::default(),
                    ui_has_modal: false,
                    focus_is_text_input: false,
                },
                children: &[],
                focus: Some(node),
                captured: None,
                bounds: picker.last_bounds,
                invalidations: Vec::new(),
                requested_focus: None,
                requested_capture: None,
                requested_cursor: None,
                stop_propagation: false,
            };

            picker.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Space,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
            picker.event(
                &mut cx,
                &Event::KeyUp {
                    key: fret_core::KeyCode::Space,
                    modifiers: Modifiers::default(),
                },
            );
        }

        let requested = host
            .global::<PopoverSurfaceService>()
            .and_then(|s| s.request(window))
            .is_some();
        assert!(requested, "expected popover surface request to be set");

        let dispatched_open = host.effects().iter().any(|e| {
            matches!(
                e,
                Effect::Command { window: Some(w), command }
                    if *w == window && command.as_str() == "popover_surface.open"
            )
        });
        assert!(
            dispatched_open,
            "expected popover_surface.open command to dispatch"
        );

        // Mark it as open (owner=node), then activation should dispatch close.
        host.with_global_mut(PopoverSurfaceService::default, |service, _app| {
            service.set_request(
                window,
                PopoverSurfaceRequest::new(node, picker.last_bounds, content_node),
            );
        });

        {
            let mut cx = EventCx {
                app: &mut host,
                services: &mut services,
                node,
                window: Some(window),
                input_ctx: InputContext {
                    platform: Platform::Linux,
                    caps: fret_core::PlatformCapabilities::default(),
                    ui_has_modal: false,
                    focus_is_text_input: false,
                },
                children: &[],
                focus: Some(node),
                captured: None,
                bounds: picker.last_bounds,
                invalidations: Vec::new(),
                requested_focus: None,
                requested_capture: None,
                requested_cursor: None,
                stop_propagation: false,
            };
            picker.event(
                &mut cx,
                &Event::KeyDown {
                    key: fret_core::KeyCode::Space,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
            picker.event(
                &mut cx,
                &Event::KeyUp {
                    key: fret_core::KeyCode::Space,
                    modifiers: Modifiers::default(),
                },
            );
        }

        let dispatched_close = host.effects().iter().any(|e| {
            matches!(
                e,
                Effect::Command { window: Some(w), command }
                    if *w == window && command.as_str() == "popover_surface.close"
            )
        });
        assert!(
            dispatched_close,
            "expected popover_surface.close command to dispatch"
        );

        // Pointer input remains supported (smoke).
        {
            let mut cx = EventCx {
                app: &mut host,
                services: &mut services,
                node,
                window: Some(window),
                input_ctx: InputContext {
                    platform: Platform::Linux,
                    caps: fret_core::PlatformCapabilities::default(),
                    ui_has_modal: false,
                    focus_is_text_input: false,
                },
                children: &[],
                focus: Some(node),
                captured: None,
                bounds: picker.last_bounds,
                invalidations: Vec::new(),
                requested_focus: None,
                requested_capture: None,
                requested_cursor: None,
                stop_propagation: false,
            };
            picker.event(
                &mut cx,
                &Event::Pointer(fret_core::PointerEvent::Down {
                    position: Point::new(Px(12.0), Px(12.0)),
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                }),
            );
        }
    }
}

use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget};

use crate::recipes::input::{InputTokenKeys, resolve_input_chrome};
use crate::style::ChromeRefinement;
use crate::{PopoverItem, PopoverRequest, PopoverService, Sizable, Size as ComponentSize};

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub label: Arc<str>,
    pub enabled: bool,
}

impl SelectOption {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

pub struct Select {
    model: Model<usize>,
    options: Vec<SelectOption>,
    placeholder: Arc<str>,
    style: ChromeRefinement,
    size: ComponentSize,

    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,
    resolved: ResolvedSelectStyle,

    label_blob: Option<fret_core::TextBlobId>,
    label_metrics: Option<TextMetrics>,
    label_scale_factor_bits: Option<u32>,
    label_theme_revision: Option<u64>,
    last_label: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSelectStyle {
    padding_x: Px,
    padding_y: Px,
    min_height: Px,
    radius: Px,
    border_width: Px,
    background: Color,
    border_color: Color,
    text_color: Color,
    text_size: Px,
}

impl Default for ResolvedSelectStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(12.0),
            padding_y: Px(6.0),
            min_height: Px(32.0),
            radius: Px(8.0),
            border_width: Px(1.0),
            background: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            text_color: Color::TRANSPARENT,
            text_size: Px(13.0),
        }
    }
}

impl Select {
    pub fn new(model: Model<usize>, options: Vec<SelectOption>) -> Self {
        Self {
            model,
            options,
            placeholder: "Select...".into(),
            style: ChromeRefinement::default(),
            size: ComponentSize::Medium,
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            resolved: ResolvedSelectStyle::default(),
            label_blob: None,
            label_metrics: None,
            label_scale_factor_bits: None,
            label_theme_revision: None,
            last_label: None,
        }
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let resolved = resolve_input_chrome(
            theme,
            self.size,
            &self.style,
            InputTokenKeys {
                padding_x: Some("component.select.padding_x"),
                padding_y: Some("component.select.padding_y"),
                min_height: Some("component.select.min_height"),
                radius: Some("component.select.radius"),
                border_width: Some("component.select.border_width"),
                bg: Some("component.select.bg"),
                border: Some("component.select.border"),
                border_focus: Some("component.select.border_focus"),
                fg: Some("component.select.fg"),
                text_px: Some("component.select.text_px"),
                selection: Some("component.select.selection"),
            },
        );

        self.resolved = ResolvedSelectStyle {
            padding_x: resolved.padding.left,
            padding_y: resolved.padding.top,
            min_height: resolved.min_height,
            radius: resolved.radius,
            border_width: resolved.border_width,
            background: resolved.background,
            border_color: resolved.border_color,
            text_color: resolved.text_color,
            text_size: resolved.text_px,
        };

        // Any style change can affect text layout. Keep the old blob until the next paint can
        // release/prepare a new one.
        self.label_metrics = None;
    }

    fn current_index<H: UiHost>(&self, app: &H) -> usize {
        app.models()
            .get(self.model)
            .copied()
            .unwrap_or_default()
            .min(self.options.len().saturating_sub(1))
    }

    fn current_label<H: UiHost>(&self, app: &H) -> Arc<str> {
        let idx = self.current_index(app);
        self.options
            .get(idx)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.clone())
    }

    fn sync_result<H: UiHost>(
        &mut self,
        app: &mut H,
        window: fret_core::AppWindowId,
        owner: fret_core::NodeId,
    ) -> bool {
        let mut changed = false;
        app.with_global_mut(PopoverService::default, |service, app| {
            if let Some(selected) = service.take_result(window, owner) {
                let selected = selected.min(self.options.len().saturating_sub(1));
                let _ = app.models_mut().update(self.model, |v| {
                    if *v != selected {
                        *v = selected;
                        changed = true;
                    }
                });
            }
        });
        changed
    }

    fn toggle_popover<H: UiHost>(&mut self, cx: &mut fret_ui::EventCx<'_, H>) {
        let Some(window) = cx.window else {
            return;
        };

        if cx
            .app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .is_some_and(|(_, req)| req.owner == cx.node)
        {
            cx.dispatch_command(fret_runtime::CommandId::from("popover.close"));
            cx.stop_propagation();
            return;
        }

        let selected = self.current_index(cx.app);
        let items: Vec<PopoverItem> = self
            .options
            .iter()
            .map(|o| PopoverItem {
                label: o.label.clone(),
                enabled: o.enabled,
            })
            .collect();

        cx.app
            .with_global_mut(PopoverService::default, |service, _app| {
                service.set_request(
                    window,
                    PopoverRequest {
                        owner: cx.node,
                        anchor: self.last_bounds,
                        items,
                        selected: Some(selected),
                        request_focus: true,
                    },
                );
            });
        cx.dispatch_command(fret_runtime::CommandId::from("popover.open"));
        cx.stop_propagation();
    }
}

impl<H: UiHost> Widget<H> for Select {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        if let Some(b) = self.label_blob.take() {
            services.text().release(b);
        }
        self.label_metrics = None;
        self.label_scale_factor_bits = None;
        self.label_theme_revision = None;
        self.last_label = None;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
    }

    fn event(&mut self, cx: &mut fret_ui::EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };

        // `UiTree` may translate bounds without re-running layout (e.g. scroll). Keep our anchor
        // rect up-to-date for hit-testing and popover placement.
        self.last_bounds = cx.bounds;

        if self.sync_result(cx.app, window, cx.node) {
            cx.invalidate_self(Invalidation::Layout);
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
        }

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr {
                    return;
                }
                match key {
                    KeyCode::Enter | KeyCode::Space => {
                        self.toggle_popover(cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
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
                    let was_pressed = self.pressed;
                    self.pressed = false;
                    let hovered = self.last_bounds.contains(*position);
                    self.hovered = hovered;
                    if was_pressed && hovered {
                        self.toggle_popover(cx);
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        if let Some(window) = cx.window {
            if self.sync_result(cx.app, window, cx.node) {
                // The selected label can change without a direct pointer event on the select
                // itself (e.g. selection happens inside the popover overlay). Ensure we refresh
                // our cached text in the next paint.
                if let Some(blob) = self.label_blob.take() {
                    cx.services.text().release(blob);
                }
                self.label_metrics = None;
                self.label_scale_factor_bits = None;
                self.last_label = None;
            }
        }

        self.last_bounds = cx.bounds;
        cx.observe_model(self.model, Invalidation::Layout);

        let label = self.current_label(cx.app);
        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: self.resolved.text_size,
            ..Default::default()
        };
        let pad_x = self.resolved.padding_x;
        let pad_y = self.resolved.padding_y;

        let constraints = TextConstraints {
            max_width: Some(Px((cx.available.width.0 - pad_x.0 * 2.0).max(0.0))),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx
            .services
            .text()
            .measure(label.as_ref(), style, constraints);

        let content_h = (metrics.size.height.0 + pad_y.0 * 2.0).max(0.0);
        let h = Px(content_h.max(self.resolved.min_height.0.max(0.0)));
        Size::new(cx.available.width, Px(h.0.min(cx.available.height.0)))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        cx.observe_model(self.model, Invalidation::Paint);
        let snap = cx.theme().snapshot();

        let focused = cx.focus == Some(cx.node);
        let border_color = self.resolved.border_color;
        let pad_x = self.resolved.padding_x;
        let radius = self.resolved.radius;
        let border_w = self.resolved.border_width;

        let mut bg = self.resolved.background;
        if self.pressed || self.hovered {
            bg = snap.colors.hover_background;
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(border_w),
            border_color,
            corner_radii: Corners::all(radius),
        });

        if focused && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window) {
            // Draw an inset focus ring so it doesn't get clipped by parent clip/scissor.
            let inset = Px(1.0);
            let w = (cx.bounds.size.width.0 - inset.0 * 2.0).max(0.0);
            let h = (cx.bounds.size.height.0 - inset.0 * 2.0).max(0.0);
            let rect = Rect::new(
                Point::new(
                    Px(cx.bounds.origin.x.0 + inset.0),
                    Px(cx.bounds.origin.y.0 + inset.0),
                ),
                Size::new(Px(w), Px(h)),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: Color {
                    a: 0.0,
                    ..cx.theme().colors.focus_ring
                },
                border: Edges::all(Px(2.0)),
                border_color: cx.theme().colors.focus_ring,
                corner_radii: Corners::all(radius),
            });
        }

        let scale_bits = cx.scale_factor.to_bits();
        let label = self.current_label(cx.app);

        let needs_prepare = self.label_blob.is_none()
            || self.label_metrics.is_none()
            || self.label_scale_factor_bits != Some(scale_bits)
            || self.label_theme_revision != Some(cx.theme().revision())
            || self.last_label.as_ref().is_none_or(|l| **l != *label);

        if needs_prepare {
            if let Some(b) = self.label_blob.take() {
                cx.services.text().release(b);
            }

            let style = TextStyle {
                font: fret_core::FontId::default(),
                size: self.resolved.text_size,
                ..Default::default()
            };
            let constraints = TextConstraints {
                max_width: Some(Px((cx.bounds.size.width.0 - pad_x.0 * 2.0).max(0.0))),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) = cx
                .services
                .text()
                .prepare(label.as_ref(), style, constraints);
            self.label_blob = Some(blob);
            self.label_metrics = Some(metrics);
            self.label_scale_factor_bits = Some(scale_bits);
            self.label_theme_revision = Some(cx.theme().revision());
            self.last_label = Some(label);
        }

        let Some(blob) = self.label_blob else {
            return;
        };
        let Some(metrics) = self.label_metrics else {
            return;
        };

        let text_x = Px(cx.bounds.origin.x.0 + pad_x.0);
        let inner_y =
            cx.bounds.origin.y.0 + ((cx.bounds.size.height.0 - metrics.size.height.0) * 0.5);
        let text_y = Px(inner_y + metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(2),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: self.resolved.text_color,
        });
    }
}

impl Sizable for Select {
    fn with_size(self, size: ComponentSize) -> Self {
        Select::with_size(self, size)
    }
}

use std::sync::Arc;

use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, MouseButton, Point, Px, Rect, SceneOp,
    SemanticsRole, Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{
    Invalidation, LayoutCx, PaintCx, PopoverItem, PopoverRequest, PopoverService, Theme, UiHost,
    Widget,
};

use crate::style::StyleRefinement;

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
    style: StyleRefinement,

    hovered: bool,
    pressed: bool,
    last_bounds: Rect,
    last_theme_revision: Option<u64>,

    label_blob: Option<fret_core::TextBlobId>,
    label_metrics: Option<TextMetrics>,
    label_scale_factor_bits: Option<u32>,
    last_label: Option<Arc<str>>,
}

impl Select {
    pub fn new(model: Model<usize>, options: Vec<SelectOption>) -> Self {
        Self {
            model,
            options,
            placeholder: "Select...".into(),
            style: StyleRefinement::default(),
            hovered: false,
            pressed: false,
            last_bounds: Rect::default(),
            last_theme_revision: None,
            label_blob: None,
            label_metrics: None,
            label_scale_factor_bits: None,
            last_label: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn refine_style(mut self, style: StyleRefinement) -> Self {
        self.style = style;
        self.last_theme_revision = None;
        self
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
                    },
                );
            });
        cx.dispatch_command(fret_runtime::CommandId::from("popover.open"));
        cx.stop_propagation();
    }
}

impl<H: UiHost> Widget<H> for Select {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(b) = self.label_blob.take() {
            text.release(b);
        }
        self.label_metrics = None;
        self.label_scale_factor_bits = None;
        self.last_label = None;
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Button);
    }

    fn event(&mut self, cx: &mut fret_ui::EventCx<'_, H>, event: &Event) {
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

        let Event::Pointer(pe) = event else {
            return;
        };
        match pe {
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
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        if let Some(window) = cx.window {
            if self.sync_result(cx.app, window, cx.node) {
                // The selected label can change without a direct pointer event on the select
                // itself (e.g. selection happens inside the popover overlay). Ensure we refresh
                // our cached text in the next paint.
                if let Some(blob) = self.label_blob.take() {
                    cx.text.release(blob);
                }
                self.label_metrics = None;
                self.label_scale_factor_bits = None;
                self.last_label = None;
            }
        }

        self.last_bounds = cx.bounds;
        cx.observe_model(self.model, Invalidation::Layout);

        let theme = Theme::global(&*cx.app);
        if self.last_theme_revision != Some(theme.revision()) {
            self.last_theme_revision = Some(theme.revision());
        }

        let label = self.current_label(cx.app);
        let style = TextStyle {
            font: fret_core::FontId::default(),
            size: Px(13.0),
        };
        let pad_x = self
            .style
            .padding_x
            .clone()
            .map(|m| m.resolve(theme))
            .unwrap_or(theme.metrics.padding_md);
        let pad_y = self
            .style
            .padding_y
            .clone()
            .map(|m| m.resolve(theme))
            .unwrap_or(theme.metrics.padding_sm);

        let constraints = TextConstraints {
            max_width: Some(Px((cx.available.width.0 - pad_x.0 * 2.0).max(0.0))),
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };
        let metrics = cx.text.measure(label.as_ref(), style, constraints);

        let h = Px((metrics.size.height.0 + pad_y.0 * 2.0).max(28.0));
        Size::new(cx.available.width, Px(h.0.min(cx.available.height.0)))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.last_bounds = cx.bounds;
        cx.observe_model(self.model, Invalidation::Paint);
        let theme = cx.theme().snapshot();

        let focused = cx.focus == Some(cx.node);
        let base_border = self
            .style
            .border_color
            .clone()
            .map(|c| c.resolve(cx.theme()))
            .unwrap_or(theme.colors.panel_border);
        let border_color = base_border;

        let background = self
            .style
            .background
            .clone()
            .map(|c| c.resolve(cx.theme()))
            .unwrap_or(theme.colors.panel_background);

        let pad_x = self
            .style
            .padding_x
            .clone()
            .map(|m| m.resolve(cx.theme()))
            .unwrap_or(theme.metrics.padding_md);
        let radius = self
            .style
            .radius
            .clone()
            .map(|m| m.resolve(cx.theme()))
            .unwrap_or(theme.metrics.radius_sm);
        let border_w = self
            .style
            .border_width
            .clone()
            .map(|m| m.resolve(cx.theme()))
            .unwrap_or(Px(1.0));

        let mut bg = background;
        if self.pressed || self.hovered {
            bg = theme.colors.hover_background;
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: bg,
            border: Edges::all(border_w),
            border_color,
            corner_radii: Corners::all(radius),
        });

        if focused {
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
            || self.label_scale_factor_bits != Some(scale_bits)
            || self.last_label.as_ref().is_none_or(|l| **l != *label);

        if needs_prepare {
            if let Some(b) = self.label_blob.take() {
                cx.text.release(b);
            }

            let style = TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            };
            let pad_x = self
                .style
                .padding_x
                .clone()
                .map(|m| m.resolve(cx.theme()))
                .unwrap_or(theme.metrics.padding_md);
            let constraints = TextConstraints {
                max_width: Some(Px((cx.bounds.size.width.0 - pad_x.0 * 2.0).max(0.0))),
                wrap: TextWrap::None,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) = cx.text.prepare(label.as_ref(), style, constraints);
            self.label_blob = Some(blob);
            self.label_metrics = Some(metrics);
            self.label_scale_factor_bits = Some(scale_bits);
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
            color: theme.colors.text_primary,
        });
    }
}

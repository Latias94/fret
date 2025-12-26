use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, NodeId, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::{
    Theme, UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use std::{collections::HashMap, sync::Arc};

use crate::ChromeRefinement;
use crate::recipes::surface::{SurfaceTokenKeys, resolve_surface_chrome};

#[derive(Debug, Clone)]
pub struct DialogAction {
    pub label: Arc<str>,
    pub command: Option<CommandId>,
    pub enabled: bool,
}

impl DialogAction {
    pub fn new(label: impl Into<Arc<str>>, command: CommandId) -> Self {
        Self {
            label: label.into(),
            command: Some(command),
            enabled: true,
        }
    }

    pub fn cancel(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            command: None,
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone)]
pub struct DialogRequest {
    pub owner: NodeId,
    pub title: Arc<str>,
    pub message: Arc<str>,
    pub actions: Vec<DialogAction>,
    pub default_action: Option<usize>,
    pub cancel_command: Option<CommandId>,
}

#[derive(Debug, Default)]
pub struct DialogService {
    next_serial: u64,
    by_window: HashMap<fret_core::AppWindowId, DialogEntry>,
}

#[derive(Debug)]
struct DialogEntry {
    serial: u64,
    request: DialogRequest,
    pending_action: Option<CommandId>,
}

impl DialogService {
    pub fn set_request(&mut self, window: fret_core::AppWindowId, request: DialogRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window.insert(
            window,
            DialogEntry {
                serial,
                request,
                pending_action: None,
            },
        );
    }

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &DialogRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.serial, &entry.request))
    }

    pub fn set_pending_action(
        &mut self,
        window: fret_core::AppWindowId,
        action: Option<CommandId>,
    ) {
        let Some(entry) = self.by_window.get_mut(&window) else {
            return;
        };
        entry.pending_action = action;
    }

    pub fn take_pending_action(&mut self, window: fret_core::AppWindowId) -> Option<CommandId> {
        self.by_window
            .get_mut(&window)
            .and_then(|e| e.pending_action.take())
    }

    pub fn clear(&mut self, window: fret_core::AppWindowId) {
        self.by_window.remove(&window);
    }
}

#[derive(Debug, Clone)]
pub struct DialogStyle {
    pub backdrop_opacity: f32,
    pub max_width: Px,
    pub padding: Px,
    pub gap: Px,
    pub corner_radius: Px,
    pub shadow: Option<fret_ui::element::ShadowStyle>,
    pub border: Edges,
    pub title_style: TextStyle,
    pub body_style: TextStyle,
    pub button_style: TextStyle,
    pub button_padding_x: Px,
    pub button_padding_y: Px,
    pub button_gap: Px,
    pub button_radius: Px,
    pub button_min_width: Px,
    pub button_min_height: Px,
}

impl Default for DialogStyle {
    fn default() -> Self {
        Self {
            backdrop_opacity: 0.55,
            max_width: Px(520.0),
            padding: Px(14.0),
            gap: Px(10.0),
            corner_radius: Px(10.0),
            shadow: None,
            border: Edges::all(Px(1.0)),
            title_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(14.0),
                ..Default::default()
            },
            body_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            button_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            button_padding_x: Px(12.0),
            button_padding_y: Px(8.0),
            button_gap: Px(8.0),
            button_radius: Px(8.0),
            button_min_width: Px(84.0),
            button_min_height: Px(28.0),
        }
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
struct PreparedActionText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
    scale_factor_bits: u32,
    label: Arc<str>,
}

#[derive(Debug)]
pub struct DialogOverlay {
    style: DialogStyle,
    last_serial: Option<u64>,
    last_theme_revision: Option<u64>,

    request: Option<DialogRequest>,

    panel_bounds: Rect,
    action_bounds: Vec<Rect>,
    hover_action: Option<usize>,
    pressed_action: Option<usize>,

    prepared_title: Option<PreparedText>,
    prepared_body: Option<PreparedText>,
    prepared_actions: Vec<PreparedActionText>,
}

impl DialogOverlay {
    pub fn new() -> Self {
        Self {
            style: DialogStyle::default(),
            last_serial: None,
            last_theme_revision: None,
            request: None,
            panel_bounds: Rect::default(),
            action_bounds: Vec::new(),
            hover_action: None,
            pressed_action: None,
            prepared_title: None,
            prepared_body: None,
            prepared_actions: Vec::new(),
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let surface = resolve_surface_chrome(
            theme,
            &ChromeRefinement::default(),
            SurfaceTokenKeys {
                padding_x: Some("metric.padding.md"),
                padding_y: Some("metric.padding.md"),
                radius: Some("metric.radius.md"),
                border_width: None,
                bg: Some("color.panel.background"),
                border: Some("color.panel.border"),
            },
        );

        let mut title_style = crate::Size::Small.control_text_style(theme);
        title_style.size = title_style.size + Px(1.0);
        self.style.title_style = title_style;
        self.style.body_style = crate::Size::Small.control_text_style(theme);
        self.style.button_style = crate::Size::Small.control_text_style(theme);

        self.style.corner_radius = surface.radius;
        self.style.shadow = Some(crate::declarative::style::shadow_lg(theme, surface.radius));
        self.style.border = Edges::all(surface.border_width);
        self.style.padding = surface.padding_x;
        self.style.button_radius = surface.radius;
        self.style.gap = theme.metrics.padding_md;
        self.style.button_padding_x = theme.metrics.padding_md;
        self.style.button_padding_y = theme.metrics.padding_sm;
        self.style.button_gap = theme.metrics.padding_sm;
    }

    fn cleanup_text(&mut self, text: &mut dyn fret_core::TextService) {
        if let Some(p) = self.prepared_title.take() {
            text.release(p.blob);
        }
        if let Some(p) = self.prepared_body.take() {
            text.release(p.blob);
        }
        for p in self.prepared_actions.drain(..) {
            text.release(p.blob);
        }
    }

    fn close_with_optional_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        window: fret_core::AppWindowId,
        action: Option<CommandId>,
    ) {
        cx.app
            .with_global_mut(DialogService::default, |service, _app| {
                service.set_pending_action(window, action);
            });
        cx.dispatch_command(CommandId::from("dialog.close"));
        cx.stop_propagation();
    }

    fn action_at(&self, pos: Point) -> Option<usize> {
        for (i, bounds) in self.action_bounds.iter().enumerate() {
            if bounds.contains(pos) {
                return Some(i);
            }
        }
        None
    }

    fn recompute_layout<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>, request: &DialogRequest) {
        self.panel_bounds = Rect::default();
        self.action_bounds.clear();
        self.hover_action = None;
        self.pressed_action = None;

        let margin = Px(24.0);
        let max_w = self
            .style
            .max_width
            .0
            .min((cx.available.width.0 - margin.0 * 2.0).max(0.0));
        let panel_w = Px(max_w.max(0.0));

        let inner_w = Px((panel_w.0 - self.style.padding.0 * 2.0).max(0.0));
        let title_metrics = cx.text.measure(
            request.title.as_ref(),
            self.style.title_style,
            TextConstraints {
                max_width: Some(inner_w),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            },
        );
        let body_metrics = cx.text.measure(
            request.message.as_ref(),
            self.style.body_style,
            TextConstraints {
                max_width: Some(inner_w),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                scale_factor: cx.scale_factor,
            },
        );

        let button_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let mut button_sizes: Vec<Size> = Vec::new();
        let mut actions_w = 0.0f32;
        for action in &request.actions {
            let metrics = cx.text.measure(
                action.label.as_ref(),
                self.style.button_style,
                button_constraints,
            );
            let w = (metrics.size.width.0 + self.style.button_padding_x.0 * 2.0)
                .max(self.style.button_min_width.0);
            let h = (metrics.size.height.0 + self.style.button_padding_y.0 * 2.0)
                .max(self.style.button_min_height.0);
            button_sizes.push(Size::new(Px(w), Px(h)));
            actions_w += w;
        }
        if !button_sizes.is_empty() {
            actions_w += self.style.button_gap.0 * (button_sizes.len().saturating_sub(1) as f32);
        }

        let title_h = title_metrics.size.height.0.max(0.0);
        let body_h = body_metrics.size.height.0.max(0.0);
        let actions_h = button_sizes
            .iter()
            .map(|s| s.height.0)
            .fold(0.0f32, |a, b| a.max(b))
            .max(0.0);

        let pad = self.style.padding.0.max(0.0);
        let gap = self.style.gap.0.max(0.0);
        let panel_h = Px(pad + title_h + gap + body_h + gap + actions_h + pad);

        let x = Px(((cx.available.width.0 - panel_w.0) * 0.5).clamp(0.0, cx.available.width.0));
        let y = Px(((cx.available.height.0 - panel_h.0) * 0.5).clamp(0.0, cx.available.height.0));
        self.panel_bounds = Rect::new(Point::new(x, y), Size::new(panel_w, panel_h));

        let actions_y = y.0 + panel_h.0 - pad - actions_h;
        let mut cur_x = x.0 + panel_w.0 - pad - actions_w;
        for (i, size) in button_sizes.iter().enumerate() {
            let rect = Rect::new(
                Point::new(Px(cur_x), Px(actions_y)),
                Size::new(size.width, Px(actions_h)),
            );
            self.action_bounds.push(rect);
            cur_x += size.width.0;
            if i + 1 < request.actions.len() {
                cur_x += self.style.button_gap.0.max(0.0);
            }
        }
    }
}

impl Default for DialogOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for DialogOverlay {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        self.cleanup_text(text);
        self.last_serial = None;
        self.request = None;
        self.panel_bounds = Rect::default();
        self.action_bounds.clear();
        self.hover_action = None;
        self.pressed_action = None;
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Panel);
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };

        let request = self.request.clone();
        let Some(request) = request else {
            return;
        };

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.action_at(*position);
                    if hovered != self.hover_action {
                        self.hover_action = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if !self.panel_bounds.contains(*position) {
                        self.close_with_optional_action(cx, window, request.cancel_command.clone());
                        return;
                    }
                    if let Some(i) = self.action_at(*position) {
                        self.pressed_action = Some(i);
                        cx.capture_pointer(cx.node);
                        cx.request_focus(cx.node);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
                fret_core::PointerEvent::Up {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let pressed = self.pressed_action.take();
                    cx.release_pointer_capture();
                    if let Some(i) = pressed
                        && self.action_at(*position) == Some(i)
                        && request.actions.get(i).is_some_and(|a| a.enabled)
                    {
                        self.close_with_optional_action(
                            cx,
                            window,
                            request.actions[i].command.clone(),
                        );
                        return;
                    }
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                _ => {}
            },
            Event::KeyDown { key, modifiers, .. } => {
                if modifiers.ctrl || modifiers.meta || modifiers.alt {
                    return;
                }
                match key {
                    KeyCode::Escape => {
                        self.close_with_optional_action(cx, window, request.cancel_command.clone());
                    }
                    KeyCode::Enter => {
                        let i = request
                            .default_action
                            .unwrap_or(0)
                            .min(request.actions.len().saturating_sub(1));
                        if request.actions.get(i).is_some_and(|a| a.enabled) {
                            self.close_with_optional_action(
                                cx,
                                window,
                                request.actions[i].command.clone(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            self.request = None;
            return cx.available;
        };

        let Some((serial, request)) = cx
            .app
            .global::<DialogService>()
            .and_then(|s| s.request(window))
            .map(|(s, r)| (s, r.clone()))
        else {
            self.request = None;
            self.last_serial = None;
            return cx.available;
        };

        if self.last_serial != Some(serial) {
            self.last_serial = Some(serial);
        }
        self.request = Some(request.clone());
        self.recompute_layout(cx, &request);

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };
        let Some(service) = cx.app.global::<DialogService>() else {
            return;
        };
        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            return;
        };

        if self.last_serial != Some(serial) {
            self.cleanup_text(cx.text);
            self.last_serial = Some(serial);
            self.request = Some(request.clone());
        }

        let theme = cx.theme().snapshot();

        let backdrop = Color {
            a: self.style.backdrop_opacity.clamp(0.0, 1.0),
            ..theme.colors.surface_background
        };
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: backdrop,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        if let Some(shadow) = self.style.shadow {
            fret_ui::paint::paint_shadow(cx.scene, DrawOrder(1), self.panel_bounds, shadow);
        }
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(1),
            rect: self.panel_bounds,
            background: theme.colors.panel_background,
            border: self.style.border,
            border_color: theme.colors.panel_border,
            corner_radii: Corners::all(self.style.corner_radius),
        });

        let scale_bits = cx.scale_factor.to_bits();

        let inner_x = Px(self.panel_bounds.origin.x.0 + self.style.padding.0.max(0.0));
        let mut y = self.panel_bounds.origin.y.0 + self.style.padding.0.max(0.0);

        // Title
        let title_constraints = TextConstraints {
            max_width: Some(Px((self.panel_bounds.size.width.0
                - self.style.padding.0 * 2.0)
                .max(0.0))),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let title_metrics = cx.text.measure(
            request.title.as_ref(),
            self.style.title_style,
            title_constraints,
        );
        let title_blob = match self.prepared_title.as_ref() {
            Some(p)
                if p.scale_factor_bits == scale_bits
                    && p.text == request.title
                    && p.metrics == title_metrics =>
            {
                p.blob
            }
            _ => {
                if let Some(p) = self.prepared_title.take() {
                    cx.text.release(p.blob);
                }
                let (blob, metrics) = cx.text.prepare(
                    request.title.as_ref(),
                    self.style.title_style,
                    title_constraints,
                );
                self.prepared_title = Some(PreparedText {
                    blob,
                    metrics,
                    scale_factor_bits: scale_bits,
                    text: request.title.clone(),
                });
                blob
            }
        };
        let title_y = Px(y + title_metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(2),
            origin: Point::new(inner_x, title_y),
            text: title_blob,
            color: theme.colors.text_primary,
        });
        y += title_metrics.size.height.0.max(0.0) + self.style.gap.0.max(0.0);

        // Body
        let body_constraints = TextConstraints {
            max_width: Some(Px((self.panel_bounds.size.width.0
                - self.style.padding.0 * 2.0)
                .max(0.0))),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        let body_metrics = cx.text.measure(
            request.message.as_ref(),
            self.style.body_style,
            body_constraints,
        );
        let body_blob = match self.prepared_body.as_ref() {
            Some(p)
                if p.scale_factor_bits == scale_bits
                    && p.text == request.message
                    && p.metrics == body_metrics =>
            {
                p.blob
            }
            _ => {
                if let Some(p) = self.prepared_body.take() {
                    cx.text.release(p.blob);
                }
                let (blob, metrics) = cx.text.prepare(
                    request.message.as_ref(),
                    self.style.body_style,
                    body_constraints,
                );
                self.prepared_body = Some(PreparedText {
                    blob,
                    metrics,
                    scale_factor_bits: scale_bits,
                    text: request.message.clone(),
                });
                blob
            }
        };
        let body_y = Px(y + body_metrics.baseline.0);
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(2),
            origin: Point::new(inner_x, body_y),
            text: body_blob,
            color: theme.colors.text_muted,
        });

        // Actions
        let button_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };
        if self.prepared_actions.len() != request.actions.len() {
            for p in self.prepared_actions.drain(..) {
                cx.text.release(p.blob);
            }
        }
        for (i, action) in request.actions.iter().enumerate() {
            let metrics = cx.text.measure(
                action.label.as_ref(),
                self.style.button_style,
                button_constraints,
            );
            let needs_prepare = self.prepared_actions.get(i).is_none_or(|p| {
                p.scale_factor_bits != scale_bits || p.label != action.label || p.metrics != metrics
            });
            if needs_prepare {
                if let Some(p) = self.prepared_actions.get_mut(i) {
                    cx.text.release(p.blob);
                }
                let (blob, m) = cx.text.prepare(
                    action.label.as_ref(),
                    self.style.button_style,
                    button_constraints,
                );
                let p = PreparedActionText {
                    blob,
                    metrics: m,
                    scale_factor_bits: scale_bits,
                    label: action.label.clone(),
                };
                if self.prepared_actions.len() <= i {
                    self.prepared_actions.push(p);
                } else {
                    self.prepared_actions[i] = p;
                }
            }
        }

        for (i, rect) in self.action_bounds.iter().enumerate() {
            let Some(action) = request.actions.get(i) else {
                continue;
            };
            let hovered = self.hover_action == Some(i);
            let pressed = self.pressed_action == Some(i);
            let bg = if pressed {
                theme.colors.selection_background
            } else if hovered {
                theme.colors.hover_background
            } else {
                Color {
                    a: 0.0,
                    ..theme.colors.panel_background
                }
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect: *rect,
                background: bg,
                border: Edges::all(Px(1.0)),
                border_color: theme.colors.panel_border,
                corner_radii: Corners::all(self.style.button_radius),
            });

            let Some(p) = self.prepared_actions.get(i) else {
                continue;
            };
            let text_x = Px(rect.origin.x.0 + (rect.size.width.0 - p.metrics.size.width.0) * 0.5);
            let text_y = Px(rect.origin.y.0
                + ((rect.size.height.0 - p.metrics.size.height.0) * 0.5)
                + p.metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(4),
                origin: Point::new(text_x, text_y),
                text: p.blob,
                color: if action.enabled {
                    theme.colors.text_primary
                } else {
                    theme.colors.text_disabled
                },
            });
        }
    }
}

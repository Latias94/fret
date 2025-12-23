use crate::{
    Theme, UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, NodeId, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use fret_runtime::CommandId;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct PopoverItem {
    pub label: Arc<str>,
    pub enabled: bool,
}

impl PopoverItem {
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

#[derive(Debug, Clone)]
pub struct PopoverRequest {
    pub owner: NodeId,
    pub anchor: Rect,
    pub items: Vec<PopoverItem>,
    pub selected: Option<usize>,
}

#[derive(Debug, Default)]
pub struct PopoverService {
    next_serial: u64,
    by_window: HashMap<fret_core::AppWindowId, PopoverEntry>,
    results: HashMap<(fret_core::AppWindowId, NodeId), usize>,
}

#[derive(Debug)]
struct PopoverEntry {
    serial: u64,
    request: PopoverRequest,
}

impl PopoverService {
    pub fn set_request(&mut self, window: fret_core::AppWindowId, request: PopoverRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window
            .insert(window, PopoverEntry { serial, request });
    }

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &PopoverRequest)> {
        let entry = self.by_window.get(&window)?;
        Some((entry.serial, &entry.request))
    }

    pub fn clear_request(&mut self, window: fret_core::AppWindowId) {
        self.by_window.remove(&window);
    }

    pub fn set_result(&mut self, window: fret_core::AppWindowId, owner: NodeId, selected: usize) {
        self.results.insert((window, owner), selected);
    }

    pub fn take_result(&mut self, window: fret_core::AppWindowId, owner: NodeId) -> Option<usize> {
        self.results.remove(&(window, owner))
    }
}

#[derive(Debug, Clone)]
pub struct PopoverStyle {
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    pub row_hover: Color,
    pub row_selected: Color,
    pub text_color: Color,
    pub disabled_text_color: Color,
    pub text_style: TextStyle,
    pub padding_x: Px,
    pub padding_y: Px,
    pub row_height: Px,
    pub gap: Px,
}

impl Default for PopoverStyle {
    fn default() -> Self {
        Self {
            background: Color {
                r: 0.10,
                g: 0.10,
                b: 0.12,
                a: 1.0,
            },
            border: Edges::all(Px(1.0)),
            border_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.40,
            },
            corner_radii: Corners::all(Px(8.0)),
            row_hover: Color {
                r: 0.16,
                g: 0.17,
                b: 0.22,
                a: 0.95,
            },
            row_selected: Color {
                r: 0.24,
                g: 0.34,
                b: 0.52,
                a: 0.65,
            },
            text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 1.0,
            },
            disabled_text_color: Color {
                r: 0.92,
                g: 0.92,
                b: 0.92,
                a: 0.45,
            },
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
            padding_x: Px(10.0),
            padding_y: Px(8.0),
            row_height: Px(22.0),
            gap: Px(2.0),
        }
    }
}

#[derive(Debug)]
struct PreparedRow {
    label: fret_core::TextBlobId,
    metrics: TextMetrics,
    enabled: bool,
    bounds: Rect,
}

#[derive(Debug)]
pub struct Popover {
    style: PopoverStyle,
    last_bounds: Rect,
    last_serial: Option<u64>,
    last_theme_revision: Option<u64>,
    hover_row: Option<usize>,
    rows: Vec<PreparedRow>,
    panel_bounds: Rect,
}

impl Popover {
    pub fn new() -> Self {
        Self {
            style: PopoverStyle::default(),
            last_bounds: Rect::default(),
            last_serial: None,
            last_theme_revision: None,
            hover_row: None,
            rows: Vec::new(),
            panel_bounds: Rect::default(),
        }
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());
        let radius = theme.metrics.radius_md;
        self.style.background = theme.colors.menu_background;
        self.style.border_color = theme.colors.menu_border;
        self.style.corner_radii = Corners::all(radius);
        self.style.row_hover = theme.colors.menu_item_hover;
        self.style.row_selected = theme.colors.menu_item_selected;
        self.style.text_color = theme.colors.text_primary;
        self.style.disabled_text_color = theme.colors.text_disabled;
    }

    fn cleanup(&mut self, text: &mut dyn fret_core::TextService) {
        for row in self.rows.drain(..) {
            text.release(row.label);
        }
    }

    fn hit_test_row(&self, point: Point) -> Option<usize> {
        if !self.panel_bounds.contains(point) {
            return None;
        }
        for (i, row) in self.rows.iter().enumerate() {
            if row.bounds.contains(point) {
                return Some(i);
            }
        }
        None
    }

    fn close_popover<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        self.cleanup(cx.text);
        cx.dispatch_command(CommandId::from("popover.close"));
        cx.stop_propagation();
    }

    fn activate_row<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        window: fret_core::AppWindowId,
        request: &PopoverRequest,
        index: usize,
    ) {
        let Some(row) = self.rows.get(index) else {
            self.close_popover(cx);
            return;
        };
        if !row.enabled {
            return;
        }

        cx.app
            .with_global_mut(PopoverService::default, |service, _app| {
                service.set_result(window, request.owner, index);
            });
        self.cleanup(cx.text);
        cx.dispatch_command(CommandId::from("popover.close"));
        cx.stop_propagation();
    }

    fn rebuild_rows<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>, request: &PopoverRequest) {
        let text_constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        // Measure widths.
        let mut max_w = Px(120.0);
        let mut prepared: Vec<(fret_core::TextBlobId, TextMetrics, bool)> = Vec::new();
        for item in &request.items {
            let (blob, metrics) =
                cx.text
                    .prepare(item.label.as_ref(), self.style.text_style, text_constraints);
            max_w = Px(max_w.0.max(metrics.size.width.0));
            prepared.push((blob, metrics, item.enabled));
        }

        let panel_w = Px(max_w.0 + self.style.padding_x.0 * 2.0);
        let panel_h =
            Px((request.items.len() as f32) * self.style.row_height.0
                + self.style.padding_y.0 * 2.0);

        // Anchor below unless it doesn't fit.
        let gap = self.style.gap.0.max(0.0);
        let mut x = request.anchor.origin.x.0;
        let below_y = request.anchor.origin.y.0 + request.anchor.size.height.0 + gap;
        let above_y = request.anchor.origin.y.0 - panel_h.0 - gap;

        let bottom = cx.bounds.origin.y.0 + cx.bounds.size.height.0;
        let top = cx.bounds.origin.y.0;
        let mut y = if below_y + panel_h.0 <= bottom {
            below_y
        } else if above_y >= top {
            above_y
        } else {
            below_y
        };

        // Clamp to window bounds.
        let right = cx.bounds.origin.x.0 + cx.bounds.size.width.0;
        x = x.clamp(
            cx.bounds.origin.x.0,
            (right - panel_w.0).max(cx.bounds.origin.x.0),
        );
        y = y.clamp(
            cx.bounds.origin.y.0,
            (bottom - panel_h.0).max(cx.bounds.origin.y.0),
        );

        self.panel_bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(panel_w, panel_h));

        // Place rows.
        let mut row_y = self.panel_bounds.origin.y.0 + self.style.padding_y.0;
        self.rows.clear();
        for (blob, metrics, enabled) in prepared {
            let bounds = Rect::new(
                Point::new(self.panel_bounds.origin.x, Px(row_y)),
                Size::new(self.panel_bounds.size.width, self.style.row_height),
            );
            row_y += self.style.row_height.0;
            self.rows.push(PreparedRow {
                label: blob,
                metrics,
                enabled,
                bounds,
            });
        }
    }
}

impl Default for Popover {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for Popover {
    fn cleanup_resources(&mut self, text: &mut dyn fret_core::TextService) {
        self.cleanup(text);
        self.last_serial = None;
        self.hover_row = None;
        self.panel_bounds = Rect::default();
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Menu);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        self.sync_style_from_theme(cx.theme());

        let Some((_, request)) = cx
            .app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .map(|(serial, request)| (serial, request.clone()))
        else {
            return;
        };

        match event {
            Event::Pointer(fret_core::PointerEvent::Move { position, .. }) => {
                let hovered = self.hit_test_row(*position);
                if hovered != self.hover_row {
                    self.hover_row = hovered;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                if self.panel_bounds.contains(*position) {
                    cx.capture_pointer(cx.node);
                } else {
                    self.close_popover(cx);
                }
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position, button, ..
            }) => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.release_pointer_capture();
                if let Some(i) = self.hit_test_row(*position) {
                    self.activate_row(cx, window, &request, i);
                }
            }
            Event::KeyDown { key, .. } => match key {
                KeyCode::Escape => self.close_popover(cx),
                KeyCode::Enter => {
                    if let Some(i) = self.hover_row.or(request.selected) {
                        self.activate_row(cx, window, &request, i);
                    }
                }
                KeyCode::ArrowDown => {
                    let start = self.hover_row.or(request.selected).unwrap_or(0);
                    let next = (start + 1).min(self.rows.len().saturating_sub(1));
                    self.hover_row = Some(next);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                KeyCode::ArrowUp => {
                    let start = self.hover_row.or(request.selected).unwrap_or(0);
                    let next = start.saturating_sub(1);
                    self.hover_row = Some(next);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;
        Size::new(cx.available.width, cx.available.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some(window) = cx.window else {
            return;
        };

        self.sync_style_from_theme(cx.theme());

        let Some(service) = cx.app.global::<PopoverService>() else {
            return;
        };
        let Some((serial, request)) = service
            .request(window)
            .map(|(serial, request)| (serial, request.clone()))
        else {
            if self.last_serial.is_some() {
                self.cleanup(cx.text);
                self.last_serial = None;
                self.panel_bounds = Rect::default();
                self.hover_row = None;
            }
            return;
        };

        let rebuild = self.last_serial != Some(serial) || self.last_bounds != cx.bounds;
        self.last_serial = Some(serial);
        self.last_bounds = cx.bounds;

        if rebuild {
            self.cleanup(cx.text);
            self.rebuild_rows(cx, &request);
        }

        if request.items.is_empty() {
            return;
        }

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: self.panel_bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        for (i, row) in self.rows.iter().enumerate() {
            let selected = request.selected == Some(i);
            let hovered = self.hover_row == Some(i);
            let bg = if selected {
                self.style.row_selected
            } else if hovered {
                self.style.row_hover
            } else {
                Color {
                    a: 0.0,
                    ..self.style.background
                }
            };

            if selected || hovered {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: row.bounds,
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let text_x = Px(row.bounds.origin.x.0 + self.style.padding_x.0);
            let inner_y = row.bounds.origin.y.0
                + ((row.bounds.size.height.0 - row.metrics.size.height.0) * 0.5);
            let text_y = Px(inner_y + row.metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: row.label,
                color: if row.enabled {
                    self.style.text_color
                } else {
                    self.style.disabled_text_color
                },
            });
        }
    }
}

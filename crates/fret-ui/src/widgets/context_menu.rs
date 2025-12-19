use crate::widget::{EventCx, Invalidation, PaintCx, Widget};
use fret_app::{CommandId, InputContext, KeymapService, Menu, MenuItem, format_sequence};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect, SceneOp, Size,
    TextConstraints, TextMetrics, TextStyle, TextWrap,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct ContextMenuRequest {
    pub position: Point,
    pub menu: Menu,
    pub input_ctx: InputContext,
}

#[derive(Debug, Default)]
pub struct ContextMenuService {
    next_serial: u64,
    by_window: HashMap<fret_core::AppWindowId, ContextMenuEntry>,
}

#[derive(Debug)]
struct ContextMenuEntry {
    serial: u64,
    request: ContextMenuRequest,
    pending_action: Option<CommandId>,
}

impl ContextMenuService {
    pub fn set_request(&mut self, window: fret_core::AppWindowId, request: ContextMenuRequest) {
        self.next_serial = self.next_serial.saturating_add(1);
        let serial = self.next_serial;
        self.by_window.insert(
            window,
            ContextMenuEntry {
                serial,
                request,
                pending_action: None,
            },
        );
    }

    pub fn request(&self, window: fret_core::AppWindowId) -> Option<(u64, &ContextMenuRequest)> {
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
pub struct ContextMenuStyle {
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
    pub separator_height: Px,
}

impl Default for ContextMenuStyle {
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
            separator_height: Px(10.0),
        }
    }
}

#[derive(Debug)]
pub struct ContextMenu {
    style: ContextMenuStyle,
    last_bounds: Rect,
    last_serial: Option<u64>,
    open_path: Vec<usize>,
    selection: Vec<Option<usize>>,
    hover_panel: Option<usize>,
    hover_row: Option<usize>,
    panels: Vec<PreparedPanel>,
}

#[derive(Debug)]
struct PreparedPanel {
    bounds: Rect,
    rows: Vec<PreparedRow>,
}

#[derive(Debug)]
struct PreparedRow {
    raw_index: usize,
    height: Px,
    kind: PreparedRowKind,
    enabled: bool,
    label: Option<fret_core::TextBlobId>,
    label_metrics: Option<TextMetrics>,
    shortcut: Option<fret_core::TextBlobId>,
    shortcut_metrics: Option<TextMetrics>,
}

#[derive(Debug)]
enum PreparedRowKind {
    Separator,
    Command(CommandId),
    Submenu,
}

impl ContextMenu {
    pub fn new() -> Self {
        Self {
            style: ContextMenuStyle::default(),
            last_bounds: Rect::default(),
            last_serial: None,
            open_path: Vec::new(),
            selection: Vec::new(),
            hover_panel: None,
            hover_row: None,
            panels: Vec::new(),
        }
    }

    pub fn with_style(mut self, style: ContextMenuStyle) -> Self {
        self.style = style;
        self
    }

    fn cleanup(&mut self, text: &mut dyn fret_core::TextService) {
        for panel in self.panels.drain(..) {
            for row in panel.rows {
                if let Some(blob) = row.label {
                    text.release(blob);
                }
                if let Some(blob) = row.shortcut {
                    text.release(blob);
                }
            }
        }
    }

    fn current_depth(&self) -> usize {
        self.open_path.len()
    }

    fn selection_raw(&self, depth: usize) -> Option<usize> {
        self.selection.get(depth).copied().flatten()
    }

    fn set_selection_raw(&mut self, depth: usize, raw: Option<usize>) {
        if self.selection.len() <= depth {
            self.selection.resize(depth + 1, None);
        }
        self.selection[depth] = raw;
        self.selection.truncate(depth + 1);
    }

    fn ensure_prepared(&mut self, cx: &mut PaintCx<'_>) -> Option<fret_core::AppWindowId> {
        self.last_bounds = cx.bounds;

        let window = cx.window?;
        let Some(service) = cx.app.global::<ContextMenuService>() else {
            self.cleanup(cx.text);
            self.last_serial = None;
            return Some(window);
        };
        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            self.cleanup(cx.text);
            self.last_serial = None;
            return Some(window);
        };

        if self.last_serial != Some(serial) {
            self.cleanup(cx.text);
            self.open_path.clear();
            self.selection.clear();
            self.hover_panel = None;
            self.hover_row = None;
            self.last_serial = Some(serial);
        }

        self.rebuild_panels(cx, &request);
        Some(window)
    }

    fn rebuild_panels(&mut self, cx: &mut PaintCx<'_>, request: &ContextMenuRequest) {
        self.cleanup(cx.text);
        self.panels.clear();

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            scale_factor: cx.scale_factor,
        };

        let mut depth = 0usize;
        let mut items: &[MenuItem] = &request.menu.items;

        let mut next_bounds = self.compute_root_panel_bounds(request.position, cx.bounds.size);

        loop {
            let panel = self.build_panel(cx, items, request, constraints, next_bounds, depth);
            next_bounds = self.compute_next_submenu_bounds(cx.bounds.size, &panel, depth);
            self.panels.push(panel);

            if depth >= self.open_path.len() {
                break;
            }

            let idx = self.open_path[depth];
            let Some(MenuItem::Submenu { items: sub, .. }) = items.get(idx) else {
                self.open_path.truncate(depth);
                break;
            };
            items = sub;
            depth += 1;
        }

        if self.selection.len() < self.panels.len() {
            self.selection.resize(self.panels.len(), None);
        }
        self.selection.truncate(self.panels.len());
    }

    fn compute_root_panel_bounds(&self, position: Point, screen: Size) -> Rect {
        let x = Px(position.x.0 + 2.0);
        let y = Px(position.y.0 + 2.0);
        Rect::new(Point::new(x, y), screen)
    }

    fn build_panel(
        &mut self,
        cx: &mut PaintCx<'_>,
        items: &[MenuItem],
        request: &ContextMenuRequest,
        constraints: TextConstraints,
        anchor: Rect,
        depth: usize,
    ) -> PreparedPanel {
        let keymap = cx.app.global::<KeymapService>();
        let commands = cx.app.commands();

        let visible = visible_menu_indices(items, &request.input_ctx, commands);

        let mut rows: Vec<PreparedRow> = Vec::new();
        let mut max_content_w = Px(0.0);

        for raw_index in visible {
            let item = &items[raw_index];
            match item {
                MenuItem::Separator => {
                    rows.push(PreparedRow {
                        raw_index,
                        height: self.style.separator_height,
                        kind: PreparedRowKind::Separator,
                        enabled: false,
                        label: None,
                        label_metrics: None,
                        shortcut: None,
                        shortcut_metrics: None,
                    });
                }
                MenuItem::Command { command, .. } => {
                    let meta = commands.get(command.clone());
                    let title: Arc<str> = meta
                        .map(|m| m.title.clone())
                        .unwrap_or_else(|| Arc::from(command.as_str()));

                    let enabled = meta
                        .and_then(|m| m.when.as_ref())
                        .map_or(true, |expr| expr.eval(&request.input_ctx));

                    let shortcut = keymap.and_then(|service| {
                        service
                            .keymap
                            .shortcut_for_command_sequence(&request.input_ctx, command)
                            .map(|seq| format_sequence(request.input_ctx.platform, &seq))
                    });

                    let (label, label_metrics) =
                        cx.text
                            .prepare(title.as_ref(), self.style.text_style, constraints);
                    let label_w = label_metrics.size.width;

                    let (shortcut_blob, shortcut_metrics, shortcut_w) =
                        if let Some(s) = shortcut.as_deref() {
                            let (b, m) = cx.text.prepare(s, self.style.text_style, constraints);
                            let w = m.size.width;
                            (Some(b), Some(m), w)
                        } else {
                            (None, None, Px(0.0))
                        };

                    let row_w =
                        Px(label_w.0 + if shortcut_w.0 > 0.0 { 22.0 } else { 0.0 } + shortcut_w.0);
                    max_content_w = Px(max_content_w.0.max(row_w.0));

                    rows.push(PreparedRow {
                        raw_index,
                        height: self.style.row_height,
                        kind: PreparedRowKind::Command(command.clone()),
                        enabled,
                        label: Some(label),
                        label_metrics: Some(label_metrics),
                        shortcut: shortcut_blob,
                        shortcut_metrics,
                    });
                }
                MenuItem::Submenu { title, .. } => {
                    let label_text = format!("{title} ›");
                    let (label, label_metrics) =
                        cx.text
                            .prepare(&label_text, self.style.text_style, constraints);
                    max_content_w = Px(max_content_w.0.max(label_metrics.size.width.0));

                    rows.push(PreparedRow {
                        raw_index,
                        height: self.style.row_height,
                        kind: PreparedRowKind::Submenu,
                        enabled: true,
                        label: Some(label),
                        label_metrics: Some(label_metrics),
                        shortcut: None,
                        shortcut_metrics: None,
                    });
                }
            }
        }

        if self.selection_raw(depth).is_none() {
            let first = rows
                .iter()
                .find(|r| {
                    matches!(
                        r.kind,
                        PreparedRowKind::Command(_) | PreparedRowKind::Submenu
                    )
                })
                .map(|r| r.raw_index);
            self.set_selection_raw(depth, first);
        }

        let panel_w = Px(max_content_w.0 + self.style.padding_x.0 * 2.0);
        let panel_h =
            Px(rows.iter().map(|r| r.height.0).sum::<f32>() + self.style.padding_y.0 * 2.0);

        let mut x = anchor
            .origin
            .x
            .0
            .min((cx.bounds.size.width.0 - panel_w.0).max(0.0));
        let mut y = anchor
            .origin
            .y
            .0
            .min((cx.bounds.size.height.0 - panel_h.0).max(0.0));
        x = x.max(0.0);
        y = y.max(0.0);

        PreparedPanel {
            bounds: Rect::new(Point::new(Px(x), Px(y)), Size::new(panel_w, panel_h)),
            rows,
        }
    }

    fn compute_next_submenu_bounds(
        &self,
        screen: Size,
        panel: &PreparedPanel,
        depth: usize,
    ) -> Rect {
        let Some(selected_raw) = self.selection_raw(depth) else {
            return Rect::new(panel.bounds.origin, screen);
        };

        let mut y = panel.bounds.origin.y.0 + self.style.padding_y.0;
        for row in &panel.rows {
            if row.raw_index == selected_raw {
                break;
            }
            y += row.height.0;
        }

        let gap = 6.0;
        let right_x = panel.bounds.origin.x.0 + panel.bounds.size.width.0 + gap;
        Rect::new(Point::new(Px(right_x), Px(y)), screen)
    }

    fn hit_test(&self, point: Point) -> Option<(usize, usize)> {
        for (panel_index, panel) in self.panels.iter().enumerate().rev() {
            if !panel.bounds.contains(point) {
                continue;
            }
            let local_y = Px(point.y.0 - panel.bounds.origin.y.0);
            let mut y = self.style.padding_y.0;
            for (row_index, row) in panel.rows.iter().enumerate() {
                let h = row.height.0;
                if local_y.0 >= y && local_y.0 < y + h {
                    return Some((panel_index, row_index));
                }
                y += h;
            }
        }
        None
    }

    fn close_menu(&mut self, cx: &mut EventCx<'_>, window: fret_core::AppWindowId) {
        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_pending_action(window, None);
            });
        self.cleanup(cx.text);
        cx.dispatch_command(CommandId::from("context_menu.close"));
        cx.stop_propagation();
    }

    fn activate_command(
        &mut self,
        cx: &mut EventCx<'_>,
        window: fret_core::AppWindowId,
        command: CommandId,
    ) {
        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_pending_action(window, Some(command));
            });
        self.cleanup(cx.text);
        cx.dispatch_command(CommandId::from("context_menu.close"));
        cx.stop_propagation();
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ContextMenu {
    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut crate::widget::LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };

        let is_visible = cx
            .app
            .global::<ContextMenuService>()
            .and_then(|s| s.request(window))
            .is_some();
        if !is_visible {
            return;
        }

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if modifiers.ctrl || modifiers.meta || modifiers.alt {
                    return;
                }
                match key {
                    KeyCode::Escape => {
                        self.close_menu(cx, window);
                    }
                    KeyCode::Enter => {
                        let depth = self.current_depth();
                        let Some(raw) = self.selection_raw(depth) else {
                            return;
                        };
                        let Some(panel) = self.panels.get(depth) else {
                            return;
                        };
                        let Some(row) = panel.rows.iter().find(|r| r.raw_index == raw) else {
                            return;
                        };
                        if let PreparedRowKind::Command(cmd) = &row.kind {
                            if row.enabled {
                                self.activate_command(cx, window, cmd.clone());
                            }
                        }
                    }
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        let depth = self.current_depth();
                        let Some(panel) = self.panels.get(depth) else {
                            return;
                        };
                        let selectable: Vec<usize> = panel
                            .rows
                            .iter()
                            .filter(|r| {
                                r.enabled
                                    && matches!(
                                        r.kind,
                                        PreparedRowKind::Command(_) | PreparedRowKind::Submenu
                                    )
                            })
                            .map(|r| r.raw_index)
                            .collect();
                        if selectable.is_empty() {
                            return;
                        }

                        let cur = self
                            .selection_raw(depth)
                            .and_then(|raw| selectable.iter().position(|v| *v == raw));
                        let next = match (key, cur) {
                            (KeyCode::ArrowDown, Some(i)) => selectable[(i + 1) % selectable.len()],
                            (KeyCode::ArrowDown, None) => selectable[0],
                            (KeyCode::ArrowUp, Some(i)) => {
                                selectable[(i + selectable.len() - 1) % selectable.len()]
                            }
                            (KeyCode::ArrowUp, None) => selectable[selectable.len() - 1],
                            _ => selectable[0],
                        };
                        self.set_selection_raw(depth, Some(next));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        let depth = self.current_depth();
                        let Some(raw) = self.selection_raw(depth) else {
                            return;
                        };
                        let Some(panel) = self.panels.get(depth) else {
                            return;
                        };
                        let Some(row) = panel.rows.iter().find(|r| r.raw_index == raw) else {
                            return;
                        };
                        if matches!(row.kind, PreparedRowKind::Submenu) {
                            if self.open_path.len() == depth {
                                self.open_path.push(raw);
                                self.set_selection_raw(depth + 1, None);
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                                cx.stop_propagation();
                            }
                        }
                    }
                    KeyCode::ArrowLeft => {
                        if self.open_path.pop().is_some() {
                            self.selection.truncate(self.open_path.len() + 1);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    _ => {}
                }
            }
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    if let Some((panel, row)) = self.hit_test(*position) {
                        if self.hover_panel != Some(panel) || self.hover_row != Some(row) {
                            self.hover_panel = Some(panel);
                            self.hover_row = Some(row);

                            let raw = self.panels[panel].rows[row].raw_index;
                            self.set_selection_raw(panel, Some(raw));

                            if matches!(self.panels[panel].rows[row].kind, PreparedRowKind::Submenu)
                            {
                                self.open_path.truncate(panel);
                                if self.open_path.len() == panel {
                                    self.open_path.push(raw);
                                }
                                self.set_selection_raw(panel + 1, None);
                            } else {
                                self.open_path.truncate(panel);
                            }

                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left && *button != MouseButton::Right {
                        return;
                    }

                    if let Some((panel, row)) = self.hit_test(*position) {
                        let row = &self.panels[panel].rows[row];
                        match &row.kind {
                            PreparedRowKind::Command(cmd) => {
                                if row.enabled {
                                    self.activate_command(cx, window, cmd.clone());
                                }
                            }
                            PreparedRowKind::Submenu => {
                                self.open_path.truncate(panel);
                                if self.open_path.len() == panel {
                                    self.open_path.push(row.raw_index);
                                }
                                self.set_selection_raw(panel + 1, None);
                                cx.invalidate_self(Invalidation::Paint);
                                cx.request_redraw();
                                cx.stop_propagation();
                            }
                            PreparedRowKind::Separator => {
                                cx.stop_propagation();
                            }
                        }
                        return;
                    }

                    self.close_menu(cx, window);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(window) = self.ensure_prepared(cx) else {
            return;
        };

        if self.panels.is_empty() {
            return;
        }

        let Some(service) = cx.app.global::<ContextMenuService>() else {
            return;
        };
        if service.request(window).is_none() {
            return;
        }

        for (panel_index, panel) in self.panels.iter().enumerate() {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: panel.bounds,
                background: self.style.background,
                border: self.style.border,
                border_color: self.style.border_color,
                corner_radii: self.style.corner_radii,
            });

            let mut y = panel.bounds.origin.y.0 + self.style.padding_y.0;
            for row in &panel.rows {
                let row_rect = Rect::new(
                    Point::new(panel.bounds.origin.x, Px(y)),
                    Size::new(panel.bounds.size.width, row.height),
                );

                let selected = self
                    .selection_raw(panel_index)
                    .is_some_and(|raw| raw == row.raw_index)
                    && !matches!(row.kind, PreparedRowKind::Separator);

                if selected {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: row_rect,
                        background: self.style.row_selected,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                } else if self.hover_panel == Some(panel_index)
                    && self.hover_row.is_some_and(|r| {
                        panel
                            .rows
                            .get(r)
                            .is_some_and(|rr| rr.raw_index == row.raw_index)
                    })
                    && !matches!(row.kind, PreparedRowKind::Separator)
                {
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: row_rect,
                        background: self.style.row_hover,
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }

                if let (Some(label), Some(metrics)) = (row.label, row.label_metrics) {
                    let color = if row.enabled {
                        self.style.text_color
                    } else {
                        self.style.disabled_text_color
                    };
                    let inner_y = y + (row.height.0 - metrics.size.height.0) * 0.5;
                    let origin = Point::new(
                        Px(panel.bounds.origin.x.0 + self.style.padding_x.0),
                        Px(inner_y + metrics.baseline.0),
                    );
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(2),
                        origin,
                        text: label,
                        color,
                    });
                }

                if let (Some(shortcut), Some(metrics)) = (row.shortcut, row.shortcut_metrics) {
                    let x = panel.bounds.origin.x.0 + panel.bounds.size.width.0
                        - self.style.padding_x.0
                        - metrics.size.width.0;
                    let inner_y = y + (row.height.0 - metrics.size.height.0) * 0.5;
                    let origin = Point::new(Px(x), Px(inner_y + metrics.baseline.0));
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(2),
                        origin,
                        text: shortcut,
                        color: self.style.disabled_text_color,
                    });
                }

                if matches!(row.kind, PreparedRowKind::Separator) {
                    let mid_y = y + row.height.0 * 0.5;
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(2),
                        rect: Rect::new(
                            Point::new(
                                Px(panel.bounds.origin.x.0 + self.style.padding_x.0),
                                Px(mid_y),
                            ),
                            Size::new(
                                Px(panel.bounds.size.width.0 - self.style.padding_x.0 * 2.0),
                                Px(1.0),
                            ),
                        ),
                        background: Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.30,
                        },
                        border: Edges::all(Px(0.0)),
                        border_color: Color::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });
                }

                y += row.height.0;
            }
        }
    }
}

fn visible_menu_indices(
    items: &[MenuItem],
    ctx: &InputContext,
    commands: &fret_app::CommandRegistry,
) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();

    for (i, item) in items.iter().enumerate() {
        match item {
            MenuItem::Separator => out.push(i),
            MenuItem::Command { when, .. } => {
                if when.as_ref().is_some_and(|w| !w.eval(ctx)) {
                    continue;
                }
                out.push(i);
            }
            MenuItem::Submenu { when, items, .. } => {
                if when.as_ref().is_some_and(|w| !w.eval(ctx)) {
                    continue;
                }
                if visible_menu_indices(items, ctx, commands)
                    .into_iter()
                    .any(|idx| !matches!(items[idx], MenuItem::Separator))
                {
                    out.push(i);
                }
            }
        }
    }

    // Trim leading/trailing separators and collapse runs.
    while out
        .first()
        .is_some_and(|i| matches!(items[*i], MenuItem::Separator))
    {
        out.remove(0);
    }
    while out
        .last()
        .is_some_and(|i| matches!(items[*i], MenuItem::Separator))
    {
        out.pop();
    }
    let mut collapsed: Vec<usize> = Vec::new();
    let mut last_was_sep = false;
    for idx in out {
        let is_sep = matches!(items[idx], MenuItem::Separator);
        if is_sep && last_was_sep {
            continue;
        }
        last_was_sep = is_sep;
        collapsed.push(idx);
    }
    collapsed
}

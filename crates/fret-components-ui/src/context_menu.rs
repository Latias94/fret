use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, MouseButton, Point, Px, Rect, SceneOp,
    SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::{
    CommandId, CommandRegistry, InputContext, KeymapService, MenuItem, format_sequence,
};
use fret_ui::{
    ContextMenuRequest, ContextMenuService, MenuBarContextMenu, Theme, UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, SemanticsCx, Widget},
};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ChromeRefinement;
use crate::Size as ComponentSize;
use crate::recipes::menu_list::resolve_menu_list_row_chrome;
use crate::recipes::surface::{SurfaceTokenKeys, resolve_surface_chrome};

#[derive(Debug, Clone)]
pub struct ContextMenuStyle {
    pub background: Color,
    pub shadow: Option<fret_ui::element::ShadowStyle>,
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
            shadow: None,
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
                ..Default::default()
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
    style_override: bool,
    size: ComponentSize,
    last_bounds: Rect,
    last_serial: Option<u64>,
    last_theme_revision: Option<u64>,
    open_path: Vec<usize>,
    selection: Vec<Option<usize>>,
    hover_panel: Option<usize>,
    hover_row: Option<usize>,
    panels: Vec<PreparedPanel>,
    typeahead: String,
    typeahead_last: Option<Instant>,
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
    label_text: Option<Arc<str>>,
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
            style_override: false,
            size: ComponentSize::Small,
            last_bounds: Rect::default(),
            last_serial: None,
            last_theme_revision: None,
            open_path: Vec::new(),
            selection: Vec::new(),
            hover_panel: None,
            hover_row: None,
            panels: Vec::new(),
            typeahead: String::new(),
            typeahead_last: None,
        }
    }

    pub fn with_style(mut self, style: ContextMenuStyle) -> Self {
        self.style = style;
        self.style_override = true;
        self
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.style_override {
            return;
        }
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let surface = resolve_surface_chrome(
            theme,
            &ChromeRefinement::default(),
            SurfaceTokenKeys {
                padding_x: None,
                padding_y: None,
                radius: Some("metric.radius.md"),
                border_width: None,
                bg: Some("popover.background"),
                border: Some("border"),
            },
        );
        let rows = resolve_menu_list_row_chrome(theme, self.size);

        self.style.padding_x = rows.padding_x;
        self.style.padding_y = rows.padding_y;
        self.style.border = Edges::all(surface.border_width);
        self.style.background = surface.background;
        self.style.border_color = surface.border_color;
        self.style.corner_radii = Corners::all(surface.radius);
        self.style.shadow = Some(crate::declarative::style::shadow_md(theme, surface.radius));
        self.style.row_hover = rows.row_hover;
        self.style.row_selected = rows.row_selected;
        self.style.text_color = rows.text_color;
        self.style.disabled_text_color = rows.disabled_text_color;
        self.style.text_style = rows.text_style;
        self.style.row_height = rows.row_height;
        self.style.separator_height = rows.separator_height;
    }

    fn cleanup(&mut self, services: &mut dyn fret_core::UiServices) {
        for panel in self.panels.drain(..) {
            for row in panel.rows {
                if let Some(blob) = row.label {
                    services.text().release(blob);
                }
                if let Some(blob) = row.shortcut {
                    services.text().release(blob);
                }
            }
        }
    }

    fn typeahead_timeout() -> Duration {
        Duration::from_millis(1000)
    }

    fn clear_typeahead(&mut self) {
        self.typeahead.clear();
        self.typeahead_last = None;
    }

    fn typeahead_char(key: KeyCode, modifiers: &fret_core::Modifiers) -> Option<char> {
        if modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr {
            return None;
        }
        Some(match key {
            KeyCode::KeyA => 'a',
            KeyCode::KeyB => 'b',
            KeyCode::KeyC => 'c',
            KeyCode::KeyD => 'd',
            KeyCode::KeyE => 'e',
            KeyCode::KeyF => 'f',
            KeyCode::KeyG => 'g',
            KeyCode::KeyH => 'h',
            KeyCode::KeyI => 'i',
            KeyCode::KeyJ => 'j',
            KeyCode::KeyK => 'k',
            KeyCode::KeyL => 'l',
            KeyCode::KeyM => 'm',
            KeyCode::KeyN => 'n',
            KeyCode::KeyO => 'o',
            KeyCode::KeyP => 'p',
            KeyCode::KeyQ => 'q',
            KeyCode::KeyR => 'r',
            KeyCode::KeyS => 's',
            KeyCode::KeyT => 't',
            KeyCode::KeyU => 'u',
            KeyCode::KeyV => 'v',
            KeyCode::KeyW => 'w',
            KeyCode::KeyX => 'x',
            KeyCode::KeyY => 'y',
            KeyCode::KeyZ => 'z',
            KeyCode::Digit0 | KeyCode::Numpad0 => '0',
            KeyCode::Digit1 | KeyCode::Numpad1 => '1',
            KeyCode::Digit2 | KeyCode::Numpad2 => '2',
            KeyCode::Digit3 | KeyCode::Numpad3 => '3',
            KeyCode::Digit4 | KeyCode::Numpad4 => '4',
            KeyCode::Digit5 | KeyCode::Numpad5 => '5',
            KeyCode::Digit6 | KeyCode::Numpad6 => '6',
            KeyCode::Digit7 | KeyCode::Numpad7 => '7',
            KeyCode::Digit8 | KeyCode::Numpad8 => '8',
            KeyCode::Digit9 | KeyCode::Numpad9 => '9',
            _ => return None,
        })
    }

    fn update_typeahead(&mut self, typed: char) -> String {
        let now = Instant::now();
        if self
            .typeahead_last
            .is_some_and(|t| now.duration_since(t) > Self::typeahead_timeout())
        {
            self.typeahead.clear();
        }
        self.typeahead_last = Some(now);

        let lower = typed.to_ascii_lowercase();
        let cycle_same = self.typeahead.len() == 1 && self.typeahead.chars().next() == Some(lower);

        if self.typeahead.is_empty() {
            self.typeahead.push(lower);
        } else if !cycle_same {
            self.typeahead.push(lower);
        }
        self.typeahead.clone()
    }

    fn find_typeahead_match(
        rows: &[PreparedRow],
        selected_raw: Option<usize>,
        query: &str,
    ) -> Option<usize> {
        let start_idx = selected_raw
            .and_then(|raw| rows.iter().position(|r| r.raw_index == raw))
            .map(|i| i.saturating_add(1))
            .unwrap_or(0);

        let matches = |row: &PreparedRow| {
            row.enabled
                && matches!(
                    row.kind,
                    PreparedRowKind::Command(_) | PreparedRowKind::Submenu
                )
                && row
                    .label_text
                    .as_deref()
                    .is_some_and(|t| t.to_ascii_lowercase().starts_with(query))
        };

        (start_idx..rows.len())
            .chain(0..start_idx)
            .find_map(|i| rows.get(i).filter(|r| matches(r)).map(|r| r.raw_index))
    }

    fn selectable_rows(panel: &PreparedPanel) -> Vec<usize> {
        panel
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
            .collect()
    }

    fn page_step(&self, panel: &PreparedPanel) -> usize {
        let view_h = (panel.bounds.size.height.0 - self.style.padding_y.0 * 2.0).max(0.0);
        let row_h = self.style.row_height.0.max(1.0);
        let page = (view_h / row_h).floor() as usize;
        page.max(1)
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

    fn ensure_prepared<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
    ) -> Option<fret_core::AppWindowId> {
        self.last_bounds = cx.bounds;

        let window = cx.window?;
        let Some(service) = cx.app.global::<ContextMenuService>() else {
            self.cleanup(cx.services);
            self.last_serial = None;
            return Some(window);
        };
        let Some((serial, request)) = service.request(window).map(|(s, r)| (s, r.clone())) else {
            self.cleanup(cx.services);
            self.last_serial = None;
            return Some(window);
        };

        if self.last_serial != Some(serial) {
            self.cleanup(cx.services);
            self.open_path.clear();
            self.selection.clear();
            self.hover_panel = None;
            self.hover_row = None;
            self.clear_typeahead();
            self.last_serial = Some(serial);
        }

        self.rebuild_panels(cx, &request);
        Some(window)
    }

    fn rebuild_panels<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>, request: &ContextMenuRequest) {
        self.cleanup(cx.services);
        self.panels.clear();

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
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

    fn build_panel<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
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
                        label_text: None,
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
                        .is_none_or(|expr| expr.eval(&request.input_ctx));

                    let shortcut = keymap.and_then(|service| {
                        service
                            .keymap
                            .shortcut_for_command_sequence(&request.input_ctx, command)
                            .map(|seq| format_sequence(request.input_ctx.platform, &seq))
                    });

                    let (label, label_metrics) = cx.services.text().prepare(
                        title.as_ref(),
                        self.style.text_style,
                        constraints,
                    );
                    let label_w = label_metrics.size.width;

                    let (shortcut_blob, shortcut_metrics, shortcut_w) =
                        if let Some(s) = shortcut.as_deref() {
                            let (b, m) =
                                cx.services
                                    .text()
                                    .prepare(s, self.style.text_style, constraints);
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
                        label_text: Some(title.clone()),
                        label: Some(label),
                        label_metrics: Some(label_metrics),
                        shortcut: shortcut_blob,
                        shortcut_metrics,
                    });
                }
                MenuItem::Submenu { title, .. } => {
                    let label_text = format!("{title} ›");
                    let (label, label_metrics) =
                        cx.services
                            .text()
                            .prepare(&label_text, self.style.text_style, constraints);
                    max_content_w = Px(max_content_w.0.max(label_metrics.size.width.0));

                    rows.push(PreparedRow {
                        raw_index,
                        height: self.style.row_height,
                        kind: PreparedRowKind::Submenu,
                        enabled: true,
                        label_text: Some(title.clone()),
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

    fn menu_bar_bounds(menu_bar: &MenuBarContextMenu) -> Option<Rect> {
        let mut it = menu_bar.entries.iter();
        let first = it.next()?;

        let mut min_x = first.bounds.origin.x.0;
        let mut min_y = first.bounds.origin.y.0;
        let mut max_x = first.bounds.origin.x.0 + first.bounds.size.width.0;
        let mut max_y = first.bounds.origin.y.0 + first.bounds.size.height.0;

        for entry in it {
            min_x = min_x.min(entry.bounds.origin.x.0);
            min_y = min_y.min(entry.bounds.origin.y.0);
            max_x = max_x.max(entry.bounds.origin.x.0 + entry.bounds.size.width.0);
            max_y = max_y.max(entry.bounds.origin.y.0 + entry.bounds.size.height.0);
        }

        Some(Rect::new(
            Point::new(Px(min_x), Px(min_y)),
            Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
        ))
    }

    fn menu_bar_entry_at_or_nearest(menu_bar: &MenuBarContextMenu, pos: Point) -> Option<usize> {
        if let Some(entry) = menu_bar.entries.iter().find(|e| e.bounds.contains(pos)) {
            return Some(entry.index);
        }

        // Best-effort fallback: if the pointer is within the overall menu bar strip (with a small
        // vertical slop), pick the nearest entry by x. This avoids “click closes menu, second click
        // opens” when bounds drift slightly due to platform DPI rounding or async window metrics.
        let bar = Self::menu_bar_bounds(menu_bar)?;
        let slop_y = 6.0;
        let y0 = bar.origin.y.0 - slop_y;
        let y1 = bar.origin.y.0 + bar.size.height.0 + slop_y;
        if pos.y.0 < y0 || pos.y.0 > y1 {
            return None;
        }

        let mut best: Option<(usize, f32)> = None;
        for entry in &menu_bar.entries {
            let x0 = entry.bounds.origin.x.0;
            let x1 = entry.bounds.origin.x.0 + entry.bounds.size.width.0;
            let dx = if pos.x.0 < x0 {
                x0 - pos.x.0
            } else if pos.x.0 > x1 {
                pos.x.0 - x1
            } else {
                0.0
            };
            match best {
                None => best = Some((entry.index, dx)),
                Some((_, best_dx)) if dx < best_dx => best = Some((entry.index, dx)),
                _ => {}
            }
        }
        best.map(|(i, _)| i)
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

    fn close_menu<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, window: fret_core::AppWindowId) {
        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_pending_action(window, None);
            });
        self.cleanup(cx.services);
        cx.dispatch_command(CommandId::from("context_menu.close"));
        cx.stop_propagation();
    }

    fn activate_command<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        window: fret_core::AppWindowId,
        command: CommandId,
    ) {
        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_pending_action(window, Some(command));
            });
        self.cleanup(cx.services);
        cx.dispatch_command(CommandId::from("context_menu.close"));
        cx.stop_propagation();
    }

    fn switch_menu_bar_menu<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        window: fret_core::AppWindowId,
        request: &ContextMenuRequest,
        target_index: usize,
    ) -> bool {
        let Some(menu_bar) = request.menu_bar.as_ref() else {
            return false;
        };
        let Some(entry) = menu_bar.entries.iter().find(|e| e.index == target_index) else {
            return false;
        };

        let position = Point::new(
            entry.bounds.origin.x,
            Px(entry.bounds.origin.y.0 + entry.bounds.size.height.0 + 2.0),
        );

        let next = ContextMenuRequest {
            position,
            menu: entry.menu.clone(),
            input_ctx: request.input_ctx.clone(),
            menu_bar: Some(MenuBarContextMenu {
                open_index: target_index,
                entries: menu_bar.entries.clone(),
            }),
        };

        cx.app
            .with_global_mut(ContextMenuService::default, |service, _app| {
                service.set_request(window, next);
                service.set_pending_action(window, None);
            });

        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        true
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for ContextMenu {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.cleanup(services);
        self.last_serial = None;
        self.open_path.clear();
        self.selection.clear();
        self.hover_panel = None;
        self.hover_row = None;
        self.panels.clear();
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::Menu);
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        cx.available
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        let Some(window) = cx.window else {
            return;
        };

        let Some((_, request)) = cx
            .app
            .global::<ContextMenuService>()
            .and_then(|s| s.request(window))
            .map(|(serial, request)| (serial, request.clone()))
        else {
            return;
        };

        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if modifiers.ctrl || modifiers.meta || modifiers.alt {
                    return;
                }

                if let Some(c) = Self::typeahead_char(*key, modifiers) {
                    let depth = self.current_depth();
                    let query = self.update_typeahead(c);
                    let selected = self.selection_raw(depth);
                    let next = self
                        .panels
                        .get(depth)
                        .and_then(|p| Self::find_typeahead_match(&p.rows, selected, &query));
                    if let Some(raw) = next {
                        self.set_selection_raw(depth, Some(raw));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    return;
                }

                match key {
                    KeyCode::Escape => {
                        self.clear_typeahead();
                        self.close_menu(cx, window);
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                        self.clear_typeahead();
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
                        if let PreparedRowKind::Command(cmd) = &row.kind
                            && row.enabled
                        {
                            self.activate_command(cx, window, cmd.clone());
                        }
                    }
                    KeyCode::Home | KeyCode::End => {
                        self.clear_typeahead();
                        let depth = self.current_depth();
                        let Some(panel) = self.panels.get(depth) else {
                            return;
                        };
                        let selectable = Self::selectable_rows(panel);
                        if selectable.is_empty() {
                            return;
                        }
                        let next = match key {
                            KeyCode::Home => selectable[0],
                            KeyCode::End => selectable[selectable.len() - 1],
                            _ => selectable[0],
                        };
                        self.set_selection_raw(depth, Some(next));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::PageUp | KeyCode::PageDown => {
                        self.clear_typeahead();
                        let depth = self.current_depth();
                        let Some(panel) = self.panels.get(depth) else {
                            return;
                        };
                        let selectable = Self::selectable_rows(panel);
                        if selectable.is_empty() {
                            return;
                        }

                        let step = self.page_step(panel);
                        let cur = self
                            .selection_raw(depth)
                            .and_then(|raw| selectable.iter().position(|v| *v == raw));
                        let cur = cur.unwrap_or(0);
                        let next_i = match key {
                            KeyCode::PageDown => (cur + step).min(selectable.len() - 1),
                            KeyCode::PageUp => cur.saturating_sub(step),
                            _ => cur,
                        };
                        self.set_selection_raw(depth, Some(selectable[next_i]));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        self.clear_typeahead();
                        let depth = self.current_depth();
                        let Some(panel) = self.panels.get(depth) else {
                            return;
                        };
                        let selectable = Self::selectable_rows(panel);
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
                        self.clear_typeahead();
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
                        if matches!(row.kind, PreparedRowKind::Submenu)
                            && self.open_path.len() == depth
                        {
                            self.open_path.push(raw);
                            self.set_selection_raw(depth + 1, None);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    KeyCode::ArrowLeft => {
                        self.clear_typeahead();
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
                            self.clear_typeahead();
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
                    } else if let Some(menu_bar) = request.menu_bar.as_ref()
                        && let Some(index) = Self::menu_bar_entry_at_or_nearest(menu_bar, *position)
                        && index != menu_bar.open_index
                        && self.switch_menu_bar_menu(cx, window, &request, index)
                    {
                        cx.stop_propagation();
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

                    if let Some(menu_bar) = request.menu_bar.as_ref()
                        && let Some(index) = Self::menu_bar_entry_at_or_nearest(menu_bar, *position)
                    {
                        if index == menu_bar.open_index {
                            self.close_menu(cx, window);
                        } else if self.switch_menu_bar_menu(cx, window, &request, index) {
                            cx.stop_propagation();
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

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
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
            if let Some(shadow) = self.style.shadow {
                fret_ui::paint::paint_shadow(cx.scene, DrawOrder(0), panel.bounds, shadow);
            }
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
    _commands: &CommandRegistry,
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
                if visible_menu_indices(items, ctx, _commands)
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

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_components_ui::{PopoverItem, PopoverRequest, PopoverService, Size as ComponentSize};
use fret_core::{
    Color, Corners, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, NodeId, Point, Px,
    Rect, SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::CommandId;
use fret_ui::UiTree;
use fret_ui::{
    EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, Widget, widget::SemanticsCx,
};

#[derive(Debug, Clone)]
pub struct NavigationMenuLink {
    pub label: Arc<str>,
    pub command: Option<CommandId>,
    pub disabled: bool,
}

impl NavigationMenuLink {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            command: None,
            disabled: false,
        }
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct NavigationMenuItem {
    pub label: Arc<str>,
    pub content: Vec<NavigationMenuLink>,
    pub disabled: bool,
}

impl NavigationMenuItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            content: Vec::new(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn link(mut self, link: NavigationMenuLink) -> Self {
        self.content.push(link);
        self
    }
}

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedStyle {
    gap: Px,
    radius: Px,
    trigger_h: Px,
    padding_x: Px,
    text_style: TextStyle,
    fg: Color,
    fg_disabled: Color,
    bg_hover: Color,
    bg_open: Color,
}

impl Default for ResolvedStyle {
    fn default() -> Self {
        Self {
            gap: Px(6.0),
            radius: Px(8.0),
            trigger_h: Px(32.0),
            padding_x: Px(12.0),
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
                ..Default::default()
            },
            fg: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            fg_disabled: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.45,
            },
            bg_hover: Color::TRANSPARENT,
            bg_open: Color::TRANSPARENT,
        }
    }
}

/// shadcn/ui `NavigationMenu` (prototype).
///
/// This is a lightweight top-level navigation bar that opens list-like content via the standard
/// `Popover` overlay (installed through `WindowOverlays`).
///
/// Design notes:
/// - Uses `PopoverService` for anchored overlay rendering, so content is not clipped by parent
///   panels/docking.
/// - Content is currently list-like (labels + commands). Rich panels can be added later by
///   generalizing the popover overlay to host arbitrary content.
pub struct NavigationMenu {
    items: Vec<NavigationMenuItem>,
    size: ComponentSize,
    disabled: bool,
    a11y: Option<Rc<RefCell<NavigationMenuA11yState>>>,

    active_index: usize,
    open_index: Option<usize>,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,

    last_bounds: Rect,
    item_bounds: Vec<Rect>,

    prepared: Vec<Option<PreparedText>>,
    prepared_scale_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,

    last_theme_revision: Option<u64>,
    resolved: ResolvedStyle,
}

impl NavigationMenu {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: ComponentSize::Medium,
            disabled: false,
            a11y: None,
            active_index: 0,
            open_index: None,
            hovered_index: None,
            pressed_index: None,
            last_bounds: Rect::default(),
            item_bounds: Vec::new(),
            prepared: Vec::new(),
            prepared_scale_bits: None,
            prepared_theme_revision: None,
            last_theme_revision: None,
            resolved: ResolvedStyle::default(),
        }
    }

    pub fn with_size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self.last_theme_revision = None;
        self.prepared_theme_revision = None;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn with_a11y(mut self, a11y: Rc<RefCell<NavigationMenuA11yState>>) -> Self {
        self.a11y = Some(a11y);
        self
    }

    pub fn item(mut self, item: NavigationMenuItem) -> Self {
        self.items.push(item);
        self.prepared_theme_revision = None;
        self.prepared_scale_bits = None;
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let gap = theme
            .metric_by_key("component.navigation_menu.gap")
            .or_else(|| theme.metric_by_key("component.space.1p5"))
            .unwrap_or(Px(6.0));
        let radius = theme
            .metric_by_key("component.navigation_menu.radius")
            .or_else(|| theme.metric_by_key("component.radius.md"))
            .unwrap_or(theme.metrics.radius_md);
        let trigger_h = theme
            .metric_by_key("component.navigation_menu.trigger_h")
            .unwrap_or_else(|| self.size.button_h(theme));
        let padding_x = theme
            .metric_by_key("component.navigation_menu.px")
            .unwrap_or_else(|| self.size.button_px(theme));

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let fg_disabled = theme.colors.text_disabled;

        let accent = theme
            .color_by_key("accent")
            .or_else(|| theme.color_by_key("accent.background"))
            .unwrap_or(theme.colors.hover_background);
        let mut bg_hover = accent;
        bg_hover.a = (bg_hover.a * 0.6).clamp(0.0, 1.0);
        let bg_open = accent;

        let text_style = TextStyle {
            font: fret_core::FontId::default(),
            size: self.size.control_text_px(theme),
            ..Default::default()
        };

        self.resolved = ResolvedStyle {
            gap,
            radius,
            trigger_h,
            padding_x,
            text_style,
            fg,
            fg_disabled,
            bg_hover,
            bg_open,
        };

        self.prepared_theme_revision = None;
        self.prepared_scale_bits = None;
    }

    fn ensure_capacity(&mut self) {
        if self.item_bounds.len() != self.items.len() {
            self.item_bounds.clear();
            self.item_bounds.resize(self.items.len(), Rect::default());
        }
        if self.prepared.len() != self.items.len() {
            self.prepared.clear();
            self.prepared.resize_with(self.items.len(), || None);
        }
        if self.active_index >= self.items.len() {
            self.active_index = 0;
        }
    }

    fn cleanup_prepared(&mut self, services: &mut dyn fret_core::UiServices) {
        for slot in self.prepared.iter_mut() {
            if let Some(p) = slot.take() {
                services.text().release(p.blob);
            }
        }
        self.prepared_scale_bits = None;
        self.prepared_theme_revision = None;
    }

    fn item_at(&self, position: Point) -> Option<usize> {
        self.item_bounds.iter().position(|r| r.contains(position))
    }

    fn is_item_enabled(&self, idx: usize) -> bool {
        self.items
            .get(idx)
            .is_some_and(|it| !self.disabled && !it.disabled && !it.content.is_empty())
    }

    fn is_open<H: UiHost>(&self, cx: &EventCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        cx.app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .is_some_and(|(_, req)| req.owner == cx.node)
    }

    fn set_open<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, open: Option<usize>) {
        if open.is_some() {
            self.open_index = open;
            self.sync_popover_request(cx);
            cx.dispatch_command(CommandId::from("popover.open"));
        } else {
            self.open_index = None;
            cx.dispatch_command(CommandId::from("popover.close"));
        }
    }

    fn sync_popover_request<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        let Some(window) = cx.window else {
            return;
        };

        // Keep anchor rect up-to-date even if the tree uses fast-path translations.
        self.last_bounds = cx.bounds;

        let Some(open) = self.open_index else {
            return;
        };
        let Some(item) = self.items.get(open) else {
            return;
        };
        let anchor = self.item_bounds.get(open).copied().unwrap_or(cx.bounds);

        let popover_items: Vec<PopoverItem> = item
            .content
            .iter()
            .map(|it| {
                let mut pi = PopoverItem::new(it.label.clone());
                if it.disabled {
                    pi = pi.disabled();
                }
                pi
            })
            .collect();

        cx.app
            .with_global_mut(PopoverService::default, |service, _app| {
                service.set_request(
                    window,
                    PopoverRequest {
                        owner: cx.node,
                        anchor,
                        items: popover_items,
                        selected: None,
                        request_focus: true,
                    },
                );
            });
    }

    fn sync_popover_result<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let Some(open) = self.open_index else {
            return false;
        };

        let Some(selected_row) = cx
            .app
            .global_mut::<PopoverService>()
            .and_then(|s| s.take_result(window, cx.node))
        else {
            return false;
        };

        let Some(item) = self.items.get(open) else {
            return false;
        };
        let Some(link) = item.content.get(selected_row) else {
            return false;
        };
        let Some(command) = link.command.clone() else {
            return false;
        };

        cx.dispatch_command(command);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        true
    }
}

impl Default for NavigationMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for NavigationMenu {
    fn is_focusable(&self) -> bool {
        !self.disabled
    }

    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        !self.disabled && self.last_bounds.contains(position)
    }

    fn hit_test_children(&self, _bounds: Rect, position: Point) -> bool {
        !self.disabled && self.last_bounds.contains(position)
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::MenuBar);
        cx.set_disabled(self.disabled);
        cx.set_expanded(self.open_index.is_some());
        cx.set_focusable(!self.disabled);

        if let Some(a11y) = self.a11y.as_ref() {
            let mut state = a11y.borrow_mut();
            state.group_disabled = self.disabled;

            if state.items.len() != self.items.len() {
                state.items = self
                    .items
                    .iter()
                    .map(|it| NavigationMenuA11ySlot {
                        label: it.label.clone(),
                        disabled: it.disabled,
                        content: it.content.clone(),
                        node: NodeId::default(),
                    })
                    .collect();
            } else {
                for (slot, item) in state.items.iter_mut().zip(self.items.iter()) {
                    slot.label = item.label.clone();
                    slot.disabled = item.disabled;
                    slot.content = item.content.clone();
                }
            }
        }
    }

    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        self.cleanup_prepared(services);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.ensure_capacity();
        self.last_bounds = cx.bounds;

        self.sync_style_from_theme(cx.theme());

        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_scale_bits != Some(scale_bits)
            || self.prepared_theme_revision != Some(cx.theme().revision())
        {
            self.cleanup_prepared(cx.services);
            self.prepared_scale_bits = Some(scale_bits);
            self.prepared_theme_revision = Some(cx.theme().revision());
        }

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        let mut x = cx.bounds.origin.x;
        let y = cx.bounds.origin.y;
        let h = Px(self.resolved.trigger_h.0.min(cx.available.height.0));

        for (i, item) in self.items.iter().enumerate() {
            if self.prepared[i].is_none() {
                if let Some(prev) = self.prepared[i].take() {
                    cx.services.text().release(prev.blob);
                }
                let (blob, metrics) = cx.services.text().prepare(
                    item.label.as_ref(),
                    self.resolved.text_style,
                    constraints,
                );
                self.prepared[i] = Some(PreparedText { blob, metrics });
            }

            let text_w = self.prepared[i]
                .as_ref()
                .map(|p| p.metrics.size.width)
                .unwrap_or(Px(0.0));
            let w = Px((text_w.0 + self.resolved.padding_x.0 * 2.0).max(1.0));
            let rect = Rect::new(Point::new(x, y), Size::new(w, h));
            self.item_bounds[i] = rect;
            x = Px(x.0 + w.0 + self.resolved.gap.0);
        }

        if let Some(a11y) = self.a11y.as_ref() {
            let mut state = a11y.borrow_mut();
            state.group_disabled = self.disabled;

            if state.items.len() != self.items.len() {
                state.items = self
                    .items
                    .iter()
                    .map(|it| NavigationMenuA11ySlot {
                        label: it.label.clone(),
                        disabled: it.disabled,
                        content: it.content.clone(),
                        node: NodeId::default(),
                    })
                    .collect();
            } else {
                for (slot, item) in state.items.iter_mut().zip(self.items.iter()) {
                    slot.label = item.label.clone();
                    slot.disabled = item.disabled;
                    slot.content = item.content.clone();
                }
            }

            if let Some(focus) = cx.focus
                && let Some(idx) = cx.children.iter().position(|&id| id == focus)
                && self.is_item_enabled(idx)
            {
                self.active_index = idx;
            }

            for (idx, &child) in cx.children.iter().enumerate() {
                let rect = self.item_bounds.get(idx).copied().unwrap_or_default();
                let _ = cx.layout_in(child, rect);
            }
        }

        let desired_w = Px((x.0 - cx.bounds.origin.x.0).max(0.0));
        Size::new(
            Px(desired_w.0.min(cx.available.width.0.max(0.0))),
            Px(h.0.max(0.0)),
        )
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(_window) = cx.window else {
            return;
        };

        // Consume popover selections.
        let _ = self.sync_popover_result(cx);

        // If the popover was closed externally, clear local open state.
        if self.open_index.is_some() && !self.is_open(cx) {
            self.open_index = None;
        }

        if self.disabled {
            return;
        }

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let hovered = self.item_at(*position);
                    if hovered != self.hovered_index {
                        self.hovered_index = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }

                    if hovered.is_some() || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }

                    // Hover switching between menus when open (shadcn-like).
                    if let (Some(open), Some(hov)) = (self.open_index, hovered)
                        && open != hov
                        && self.is_item_enabled(hov)
                    {
                        self.active_index = hov;
                        self.set_open(cx, Some(hov));
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
                    let Some(idx) = self.item_at(*position) else {
                        return;
                    };
                    if !self.is_item_enabled(idx) {
                        return;
                    }
                    self.pressed_index = Some(idx);
                    self.active_index = idx;
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
                    let was_pressed = self.pressed_index.take();
                    let hovered = self.item_at(*position);
                    self.hovered_index = hovered;

                    if let Some(idx) = was_pressed
                        && Some(idx) == hovered
                    {
                        let next_open = if self.open_index == Some(idx) {
                            None
                        } else {
                            Some(idx)
                        };
                        self.set_open(cx, next_open);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    } else {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                _ => {}
            },
            Event::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return;
                }
                if cx.focus != Some(cx.node) {
                    return;
                }
                match key {
                    KeyCode::ArrowLeft => {
                        if self.items.is_empty() {
                            return;
                        }
                        self.active_index = self.active_index.saturating_sub(1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        if self.items.is_empty() {
                            return;
                        }
                        self.active_index = (self.active_index + 1).min(self.items.len() - 1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        if self.is_item_enabled(self.active_index) {
                            self.set_open(cx, Some(self.active_index));
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                        }
                    }
                    KeyCode::Escape => {
                        self.set_open(cx, None);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.ensure_capacity();
        self.last_bounds = cx.bounds;
        self.sync_style_from_theme(cx.theme());

        let focused = cx.focus == Some(cx.node);
        let focus_visible = focused && fret_ui::focus_visible::is_focus_visible(cx.app, cx.window);

        for (i, item) in self.items.iter().enumerate() {
            let rect = self.item_bounds.get(i).copied().unwrap_or(Rect::default());

            let enabled = !self.disabled && !item.disabled;

            let hovered = self.hovered_index == Some(i);
            let pressed = self.pressed_index == Some(i) && cx.focus == Some(cx.node);
            let open = self.open_index == Some(i);

            let bg = if !enabled {
                Color::TRANSPARENT
            } else if pressed || open {
                self.resolved.bg_open
            } else if hovered {
                self.resolved.bg_hover
            } else {
                Color::TRANSPARENT
            };

            if bg.a > 0.0 {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(self.resolved.radius),
                });
            }

            let fg = if enabled {
                self.resolved.fg
            } else {
                self.resolved.fg_disabled
            };

            if let Some(p) = self.prepared.get(i).and_then(|p| p.as_ref()) {
                let gx = rect.origin.x.0 + (rect.size.width.0 - p.metrics.size.width.0) * 0.5;
                let top = rect.origin.y.0 + (rect.size.height.0 - p.metrics.size.height.0) * 0.5;
                let gy = top + p.metrics.baseline.0;
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(1),
                    origin: Point::new(Px(gx), Px(gy)),
                    text: p.blob,
                    color: fg,
                });
            }
        }

        if focus_visible
            && !self.items.is_empty()
            && let Some(rect) = self.item_bounds.get(self.active_index).copied()
        {
            let ring = fret_components_ui::declarative::style::focus_ring(
                cx.theme(),
                self.resolved.radius,
            );
            fret_ui::paint::paint_focus_ring(cx.scene, DrawOrder(2), rect, ring);
        }
    }
}

#[derive(Clone)]
struct NavigationMenuA11ySlot {
    label: Arc<str>,
    disabled: bool,
    content: Vec<NavigationMenuLink>,
    node: NodeId,
}

struct NavigationMenuA11yState {
    group_disabled: bool,
    items: Vec<NavigationMenuA11ySlot>,
}

impl NavigationMenuA11yState {
    fn new(count: usize) -> Self {
        Self {
            group_disabled: false,
            items: (0..count)
                .map(|_| NavigationMenuA11ySlot {
                    label: Arc::from(""),
                    disabled: false,
                    content: Vec::new(),
                    node: NodeId::default(),
                })
                .collect(),
        }
    }

    fn set_node(&mut self, index: usize, node: NodeId) {
        if let Some(slot) = self.items.get_mut(index) {
            slot.node = node;
        }
    }

    fn is_item_enabled(&self, index: usize) -> bool {
        self.items.get(index).is_some_and(|it| {
            !self.group_disabled
                && !it.disabled
                && !it.content.is_empty()
                && it.node != NodeId::default()
        })
    }

    fn focus_delta_from(&self, from: usize, delta: i32) -> Option<NodeId> {
        if self.items.is_empty() {
            return None;
        }
        let len = self.items.len() as i32;
        let mut idx = from as i32;
        for _ in 0..len {
            idx = (idx + delta + len) % len;
            let u = idx as usize;
            if self.is_item_enabled(u) {
                return Some(self.items[u].node);
            }
        }
        None
    }

    fn focus_first(&self) -> Option<NodeId> {
        self.items
            .iter()
            .enumerate()
            .find(|(i, _)| self.is_item_enabled(*i))
            .map(|(_, it)| it.node)
    }

    fn focus_last(&self) -> Option<NodeId> {
        self.items
            .iter()
            .enumerate()
            .rev()
            .find(|(i, _)| self.is_item_enabled(*i))
            .map(|(_, it)| it.node)
    }
}

struct NavigationMenuA11yItem {
    index: usize,
    a11y: Rc<RefCell<NavigationMenuA11yState>>,
    pressed: bool,
}

impl NavigationMenuA11yItem {
    fn new(index: usize, a11y: Rc<RefCell<NavigationMenuA11yState>>) -> Self {
        Self {
            index,
            a11y,
            pressed: false,
        }
    }

    fn is_open<H: UiHost>(&self, cx: &SemanticsCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        cx.app
            .global::<PopoverService>()
            .and_then(|s| s.request(window))
            .is_some_and(|(_, req)| req.owner == cx.node)
    }

    fn sync_popover_result<H: UiHost>(&self, cx: &mut EventCx<'_, H>) -> bool {
        let Some(window) = cx.window else {
            return false;
        };
        let Some(selected) = cx
            .app
            .global_mut::<PopoverService>()
            .and_then(|s| s.take_result(window, cx.node))
        else {
            return false;
        };

        let link = self
            .a11y
            .borrow()
            .items
            .get(self.index)
            .and_then(|it| it.content.get(selected))
            .cloned();
        let Some(link) = link else {
            return false;
        };
        if link.disabled {
            return false;
        }
        let Some(command) = link.command else {
            return false;
        };

        cx.dispatch_command(command);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        true
    }
}

impl<H: UiHost> Widget<H> for NavigationMenuA11yItem {
    fn is_focusable(&self) -> bool {
        self.a11y.borrow().is_item_enabled(self.index)
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let state = self.a11y.borrow();
        let Some(slot) = state.items.get(self.index) else {
            cx.set_role(SemanticsRole::Generic);
            cx.set_disabled(true);
            return;
        };

        let enabled = state.is_item_enabled(self.index);
        cx.set_role(SemanticsRole::MenuItem);
        cx.set_label(slot.label.to_string());
        cx.set_disabled(!enabled);
        cx.set_focusable(enabled);
        cx.set_invokable(enabled);
        cx.set_expanded(self.is_open(cx));
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let _ = self.sync_popover_result(cx);

        if cx.focus != Some(cx.node) {
            self.pressed = false;
            return;
        }

        match event {
            Event::KeyDown { key, repeat, .. } => {
                if *repeat {
                    return;
                }
                match key {
                    KeyCode::ArrowLeft => {
                        if let Some(target) = self.a11y.borrow().focus_delta_from(self.index, -1) {
                            cx.request_focus(target);
                        }
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowRight => {
                        if let Some(target) = self.a11y.borrow().focus_delta_from(self.index, 1) {
                            cx.request_focus(target);
                        }
                        cx.stop_propagation();
                    }
                    KeyCode::Home => {
                        if let Some(target) = self.a11y.borrow().focus_first() {
                            cx.request_focus(target);
                        }
                        cx.stop_propagation();
                    }
                    KeyCode::End => {
                        if let Some(target) = self.a11y.borrow().focus_last() {
                            cx.request_focus(target);
                        }
                        cx.stop_propagation();
                    }
                    KeyCode::Escape => {
                        cx.dispatch_command(CommandId::from("popover.close"));
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        if self.a11y.borrow().is_item_enabled(self.index) {
                            self.pressed = true;
                        }
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            Event::KeyUp { key, .. } => {
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
                    return;
                }
                if !self.pressed {
                    return;
                }
                self.pressed = false;

                let Some(window) = cx.window else {
                    return;
                };

                let content = self
                    .a11y
                    .borrow()
                    .items
                    .get(self.index)
                    .map(|it| it.content.clone())
                    .unwrap_or_default();
                if content.is_empty() {
                    return;
                }

                let popover_items: Vec<PopoverItem> = content
                    .iter()
                    .map(|it| {
                        let mut pi = PopoverItem::new(it.label.clone());
                        if it.disabled {
                            pi = pi.disabled();
                        }
                        pi
                    })
                    .collect();

                cx.app
                    .with_global_mut(PopoverService::default, |service, _app| {
                        service.set_request(
                            window,
                            PopoverRequest {
                                owner: cx.node,
                                anchor: cx.bounds,
                                items: popover_items,
                                selected: None,
                                request_focus: true,
                            },
                        );
                    });

                cx.dispatch_command(CommandId::from("popover.open"));
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

pub fn install_navigation_menu<H: UiHost>(
    ui: &mut UiTree<H>,
    parent: NodeId,
    menu: NavigationMenu,
) -> NodeId {
    let count = menu.items.len();
    let a11y = Rc::new(RefCell::new(NavigationMenuA11yState::new(count)));

    let root = ui.create_node(menu.with_a11y(a11y.clone()));
    ui.add_child(parent, root);

    for index in 0..count {
        let node = ui.create_node(NavigationMenuA11yItem::new(index, a11y.clone()));
        ui.add_child(root, node);
        a11y.borrow_mut().set_node(index, node);
    }

    root
}

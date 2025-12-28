use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_components_ui::{MetricRef, Space};
use fret_core::{
    Color, CursorIcon, DrawOrder, Edges, Event, KeyCode, MouseButton, NodeId, Point, Px, Rect,
    SceneOp, SemanticsRole, Size, TextConstraints, TextMetrics, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, Theme, UiHost, UiTree, Widget};

#[derive(Debug, Clone)]
struct PreparedText {
    blob: fret_core::TextBlobId,
    metrics: TextMetrics,
}

#[derive(Debug, Clone)]
struct ResolvedRadioGroupStyle {
    row_gap: Px,
    icon_size: Px,
    indicator_size: Px,
    label_gap: Px,
    border_width: Px,
    ring_width: Px,
    text_style: TextStyle,
    fg: Color,
    fg_disabled: Color,
    border: Color,
    ring: Color,
    indicator: Color,
}

impl Default for ResolvedRadioGroupStyle {
    fn default() -> Self {
        Self {
            row_gap: Px(12.0),
            icon_size: Px(16.0),
            indicator_size: Px(8.0),
            label_gap: Px(8.0),
            border_width: Px(1.0),
            ring_width: Px(3.0),
            text_style: TextStyle::default(),
            fg: Color::TRANSPARENT,
            fg_disabled: Color::TRANSPARENT,
            border: Color::TRANSPARENT,
            ring: Color::TRANSPARENT,
            indicator: Color::TRANSPARENT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RadioGroupItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub disabled: bool,
}

impl RadioGroupItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct RadioGroup {
    model: Model<Option<Arc<str>>>,
    items: Vec<RadioGroupItem>,
    disabled: bool,
    a11y: Option<Rc<RefCell<RadioGroupA11yState>>>,
    hovered_index: Option<usize>,
    pressed_index: Option<usize>,
    active_index: usize,
    last_bounds: Rect,
    row_bounds: Vec<Rect>,
    icon_bounds: Vec<Rect>,
    prepared: Vec<Option<PreparedText>>,
    prepared_scale_factor_bits: Option<u32>,
    prepared_theme_revision: Option<u64>,
    last_theme_revision: Option<u64>,
    resolved: ResolvedRadioGroupStyle,
}

impl RadioGroup {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            disabled: false,
            a11y: None,
            hovered_index: None,
            pressed_index: None,
            active_index: 0,
            last_bounds: Rect::default(),
            row_bounds: Vec::new(),
            icon_bounds: Vec::new(),
            prepared: Vec::new(),
            prepared_scale_factor_bits: None,
            prepared_theme_revision: None,
            last_theme_revision: None,
            resolved: ResolvedRadioGroupStyle::default(),
        }
    }

    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.items.push(item);
        self.prepared_theme_revision = None;
        self.prepared_scale_factor_bits = None;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn with_a11y(mut self, a11y: Rc<RefCell<RadioGroupA11yState>>) -> Self {
        self.a11y = Some(a11y);
        self
    }

    fn sync_style_from_theme(&mut self, theme: &Theme) {
        if self.last_theme_revision == Some(theme.revision()) {
            return;
        }
        self.last_theme_revision = Some(theme.revision());

        let row_gap = theme
            .metric_by_key("component.radio_group.gap")
            .unwrap_or_else(|| MetricRef::space(Space::N3).resolve(theme));
        let label_gap = theme
            .metric_by_key("component.radio_group.label_gap")
            .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme));
        let icon_size = theme
            .metric_by_key("component.radio_group.icon_size_px")
            .unwrap_or(Px(16.0));
        let indicator_size = theme
            .metric_by_key("component.radio_group.indicator_size_px")
            .unwrap_or(Px(8.0));

        let text_px = theme
            .metric_by_key("component.radio_group.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.radio_group.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let fg_disabled = theme.colors.text_disabled;

        let border = theme
            .color_by_key("input")
            .or_else(|| theme.color_by_key("border"))
            .unwrap_or(theme.colors.panel_border);
        let ring = theme
            .color_by_key("ring")
            .unwrap_or(theme.colors.focus_ring);
        let indicator = theme
            .color_by_key("primary")
            .or_else(|| theme.color_by_key("accent"))
            .unwrap_or(theme.colors.accent);

        self.resolved = ResolvedRadioGroupStyle {
            row_gap,
            icon_size,
            indicator_size,
            label_gap,
            border_width: Px(1.0),
            ring_width: Px(3.0),
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: text_px,
                line_height: Some(line_height),
                ..Default::default()
            },
            fg,
            fg_disabled,
            border,
            ring,
            indicator,
        };

        self.prepared_theme_revision = None;
        self.prepared_scale_factor_bits = None;
    }

    fn prepare_texts<H: UiHost>(&mut self, cx: &mut LayoutCx<'_, H>) {
        let theme_rev = cx.theme().revision();
        let scale_bits = cx.scale_factor.to_bits();
        if self.prepared_theme_revision == Some(theme_rev)
            && self.prepared_scale_factor_bits == Some(scale_bits)
            && self.prepared.len() == self.items.len()
            && self.prepared.iter().all(|p| p.is_some())
        {
            return;
        }

        for p in self.prepared.drain(..).flatten() {
            cx.services.text().release(p.blob);
        }
        self.prepared.clear();

        let constraints = TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: cx.scale_factor,
        };

        for item in &self.items {
            let (blob, metrics) =
                cx.services
                    .text()
                    .prepare(&item.label, self.resolved.text_style, constraints);
            self.prepared.push(Some(PreparedText { blob, metrics }));
        }

        self.prepared_theme_revision = Some(theme_rev);
        self.prepared_scale_factor_bits = Some(scale_bits);
    }

    fn is_selected<H: UiHost>(&self, app: &H, value: &Arc<str>) -> bool {
        app.models().get(self.model).and_then(|v| v.as_ref()) == Some(value)
    }

    fn set_selected<H: UiHost>(&self, app: &mut H, value: Arc<str>) {
        let _ = app.models_mut().update(self.model, |v| *v = Some(value));
    }

    fn is_item_enabled(&self, index: usize) -> bool {
        self.items
            .get(index)
            .is_some_and(|it| !self.disabled && !it.disabled)
    }

    fn row_at(&self, position: Point) -> Option<usize> {
        self.row_bounds.iter().position(|r| r.contains(position))
    }

    fn hit_test_position(&self, position: Point) -> bool {
        if self.disabled {
            return false;
        }
        let Some(idx) = self.row_at(position) else {
            return false;
        };
        self.is_item_enabled(idx)
    }

    fn sync_active_index<H: UiHost>(&mut self, app: &H) {
        if self.items.is_empty() {
            self.active_index = 0;
            return;
        }
        if self.active_index >= self.items.len() {
            self.active_index = 0;
        }
        if self.is_item_enabled(self.active_index) {
            return;
        }

        if let Some((idx, it)) = self
            .items
            .iter()
            .enumerate()
            .find(|(i, it)| self.is_item_enabled(*i) && self.is_selected(app, &it.value))
        {
            let _ = it;
            self.active_index = idx;
            return;
        }

        if let Some((idx, _)) = self
            .items
            .iter()
            .enumerate()
            .find(|(i, _)| self.is_item_enabled(*i))
        {
            self.active_index = idx;
        }
    }

    fn move_active(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let len = self.items.len() as i32;
        let mut idx = self.active_index as i32;
        for _ in 0..len {
            idx = (idx + delta + len) % len;
            let u = idx as usize;
            if self.is_item_enabled(u) {
                self.active_index = u;
                return;
            }
        }
    }
}

impl<H: UiHost> Widget<H> for RadioGroup {
    fn cleanup_resources(&mut self, services: &mut dyn fret_core::UiServices) {
        for p in self.prepared.drain(..).flatten() {
            services.text().release(p.blob);
        }
        self.prepared_scale_factor_bits = None;
        self.prepared_theme_revision = None;
    }

    fn is_focusable(&self) -> bool {
        !self.disabled && self.items.iter().any(|it| !it.disabled)
    }

    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.hit_test_position(position)
    }

    fn hit_test_children(&self, _bounds: Rect, position: Point) -> bool {
        self.hit_test_position(position)
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        cx.set_role(SemanticsRole::List);
        cx.set_disabled(self.disabled);
        cx.set_focusable(!self.disabled && self.items.iter().any(|it| !it.disabled));

        if let Some(a11y) = self.a11y.as_ref() {
            let mut state = a11y.borrow_mut();
            state.group_disabled = self.disabled;
            state.model = self.model;

            for (slot, item) in state.items.iter_mut().zip(self.items.iter()) {
                slot.value = item.value.clone();
                slot.label = item.label.clone();
                slot.disabled = item.disabled;
                slot.selected = self.is_selected(cx.app, &item.value);
            }
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.sync_style_from_theme(cx.theme());
        self.last_bounds = cx.bounds;
        self.sync_active_index(cx.app);

        if self.disabled {
            return;
        }

        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Move { position, .. } => {
                    let mut hovered = self.row_at(*position);
                    if hovered.is_some_and(|i| !self.is_item_enabled(i)) {
                        hovered = None;
                    }
                    if hovered != self.hovered_index {
                        self.hovered_index = hovered;
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    if hovered.is_some() || cx.captured == Some(cx.node) {
                        cx.set_cursor_icon(CursorIcon::Pointer);
                    }
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    let Some(idx) = self.row_at(*position) else {
                        return;
                    };
                    if !self.is_item_enabled(idx) {
                        return;
                    }
                    self.active_index = idx;
                    self.pressed_index = Some(idx);
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
                    let pressed = self.pressed_index.take();
                    cx.release_pointer_capture();

                    let hovered = self.row_at(*position);
                    self.hovered_index = hovered.filter(|i| self.is_item_enabled(*i));

                    if let (Some(idx), Some(hov)) = (pressed, hovered)
                        && idx == hov
                        && let Some(item) = self.items.get(idx)
                    {
                        self.set_selected(cx.app, item.value.clone());
                    }

                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
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
                    KeyCode::ArrowUp | KeyCode::ArrowLeft => {
                        self.move_active(-1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowDown | KeyCode::ArrowRight => {
                        self.move_active(1);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Home => {
                        self.active_index = 0;
                        self.move_active(0);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::End => {
                        self.active_index = self.items.len().saturating_sub(1);
                        self.move_active(0);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        if self.pressed_index.is_none() && self.is_item_enabled(self.active_index) {
                            self.pressed_index = Some(self.active_index);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                        }
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
            Event::KeyUp { key, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if !matches!(key, KeyCode::Enter | KeyCode::Space) {
                    return;
                }
                let pressed = self.pressed_index.take();
                if pressed != Some(self.active_index) {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
                if let Some(item) = self.items.get(self.active_index)
                    && self.is_item_enabled(self.active_index)
                {
                    self.set_selected(cx.app, item.value.clone());
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;

        if let Some(a11y) = self.a11y.as_ref() {
            let mut state = a11y.borrow_mut();
            state.group_disabled = self.disabled;
            state.model = self.model;

            for (slot, item) in state.items.iter_mut().zip(self.items.iter()) {
                slot.value = item.value.clone();
                slot.label = item.label.clone();
                slot.disabled = item.disabled;
                slot.selected = self.is_selected(cx.app, &item.value);
            }
        }

        self.prepare_texts(cx);

        let n = self.items.len();
        if n == 0 {
            self.row_bounds.clear();
            self.icon_bounds.clear();
            return Size::new(Px(0.0), Px(0.0));
        }

        let row_gap = self.resolved.row_gap.0.max(0.0);
        let icon = self.resolved.icon_size.0.max(0.0);
        let label_gap = self.resolved.label_gap.0.max(0.0);

        let mut row_heights: Vec<f32> = Vec::with_capacity(n);
        let mut row_widths: Vec<f32> = Vec::with_capacity(n);
        for i in 0..n {
            let Some(Some(p)) = self.prepared.get(i) else {
                row_heights.push(icon);
                row_widths.push(icon);
                continue;
            };
            let h = icon.max(p.metrics.size.height.0);
            let w = icon + label_gap + p.metrics.size.width.0;
            row_heights.push(h.max(0.0));
            row_widths.push(w.max(0.0));
        }

        let desired_w = row_widths.iter().copied().fold(0.0_f32, |a, b| a.max(b));
        let desired_h = row_heights.iter().sum::<f32>() + row_gap * (n.saturating_sub(1) as f32);

        let w = desired_w.min(cx.available.width.0).max(0.0);
        let h = desired_h.min(cx.available.height.0).max(0.0);

        self.row_bounds.clear();
        self.icon_bounds.clear();
        self.row_bounds.reserve(n);
        self.icon_bounds.reserve(n);

        let mut y = cx.bounds.origin.y.0;
        let x = cx.bounds.origin.x.0;
        for (i, &h_i) in row_heights.iter().take(n).enumerate() {
            let row_h = h_i.min(h.max(0.0)).max(0.0);
            let row = Rect {
                origin: Point::new(Px(x), Px(y)),
                size: Size::new(Px(w), Px(row_h)),
            };
            self.row_bounds.push(row);

            let icon_y = y + ((row_h - icon) * 0.5).max(0.0);
            let icon_rect = Rect {
                origin: Point::new(Px(x), Px(icon_y)),
                size: Size::new(Px(icon), Px(icon)),
            };
            self.icon_bounds.push(icon_rect);

            y += row_h;
            if i + 1 < n {
                y += row_gap;
            }
        }

        if self.a11y.is_some() {
            if let Some(focus) = cx.focus
                && let Some(idx) = cx.children.iter().position(|&id| id == focus)
                && self.is_item_enabled(idx)
            {
                self.active_index = idx;
            }

            for (idx, &child) in cx.children.iter().enumerate() {
                let rect = self.row_bounds.get(idx).copied().unwrap_or_default();
                let _ = cx.layout_in(child, rect);
            }
        }

        Size::new(Px(w), Px(h))
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.sync_style_from_theme(cx.theme());
        cx.observe_model(self.model, Invalidation::Paint);
        self.last_bounds = cx.bounds;
        self.sync_active_index(cx.app);

        if self.disabled {
            self.hovered_index = None;
            self.pressed_index = None;
        }

        let n = self.items.len();
        if n == 0 {
            return;
        }

        let focus_visible = fret_ui::focus_visible::is_focus_visible(cx.app, cx.window)
            && (cx.focus == Some(cx.node) || cx.focus.is_some_and(|f| cx.children.contains(&f)));

        for i in 0..n {
            let Some(item) = self.items.get(i) else {
                continue;
            };
            let Some(row) = self.row_bounds.get(i).copied() else {
                continue;
            };
            let Some(icon) = self.icon_bounds.get(i).copied() else {
                continue;
            };
            let Some(Some(prepared)) = self.prepared.get(i) else {
                continue;
            };

            let enabled = !self.disabled && !item.disabled;
            let selected = self.is_selected(cx.app, &item.value);
            let hovered = self.hovered_index == Some(i);
            let pressed = self.pressed_index == Some(i);

            let mut border_color = self.resolved.border;
            if focus_visible && self.active_index == i {
                border_color = self.resolved.ring;
            } else if hovered || pressed {
                border_color = self.resolved.ring;
                border_color.a *= 0.5;
            }

            let mut fg = self.resolved.fg;
            if !enabled {
                fg = self.resolved.fg_disabled;
                fg.a *= 0.5;
                border_color.a *= 0.5;
            }

            let border_w = Px(self.resolved.border_width.0.max(0.0));
            let radius = Px(icon.size.width.0.max(0.0) * 0.5);

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: icon,
                background: Color::TRANSPARENT,
                border: Edges::all(border_w),
                border_color,
                corner_radii: fret_core::Corners::all(radius),
            });

            if selected {
                let inner = self.resolved.indicator_size.0.max(0.0);
                let inner_x = icon.origin.x.0 + ((icon.size.width.0 - inner) * 0.5).max(0.0);
                let inner_y = icon.origin.y.0 + ((icon.size.height.0 - inner) * 0.5).max(0.0);
                let inner_rect = Rect {
                    origin: Point::new(Px(inner_x), Px(inner_y)),
                    size: Size::new(Px(inner), Px(inner)),
                };
                let mut c = self.resolved.indicator;
                if !enabled {
                    c.a *= 0.5;
                }
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: inner_rect,
                    background: c,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: fret_core::Corners::all(Px(inner * 0.5)),
                });
            }

            if focus_visible && self.active_index == i {
                let ring_w = self.resolved.ring_width.0.max(0.0);
                if ring_w > 0.0 {
                    let mut ring = self.resolved.ring;
                    ring.a *= 0.5;
                    let ring_rect = Rect {
                        origin: Point::new(
                            Px(icon.origin.x.0 - ring_w),
                            Px(icon.origin.y.0 - ring_w),
                        ),
                        size: Size::new(
                            Px(icon.size.width.0 + ring_w * 2.0),
                            Px(icon.size.height.0 + ring_w * 2.0),
                        ),
                    };
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(2),
                        rect: ring_rect,
                        background: Color::TRANSPARENT,
                        border: Edges::all(Px(ring_w)),
                        border_color: ring,
                        corner_radii: fret_core::Corners::all(Px(icon.size.width.0 * 0.5 + ring_w)),
                    });
                }
            }

            let label_x = icon.origin.x.0 + icon.size.width.0 + self.resolved.label_gap.0.max(0.0);
            let text_top = row.origin.y.0
                + ((row.size.height.0 - prepared.metrics.size.height.0) * 0.5).max(0.0);
            let text_y = text_top + prepared.metrics.baseline.0;

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(3),
                origin: Point::new(Px(label_x), Px(text_y)),
                text: prepared.blob,
                color: fg,
            });
        }
    }
}

#[derive(Debug, Clone)]
struct RadioGroupA11ySlot {
    value: Arc<str>,
    label: Arc<str>,
    disabled: bool,
    selected: bool,
    node: NodeId,
}

struct RadioGroupA11yState {
    model: Model<Option<Arc<str>>>,
    group_disabled: bool,
    items: Vec<RadioGroupA11ySlot>,
}

impl RadioGroupA11yState {
    fn new(model: Model<Option<Arc<str>>>, count: usize) -> Self {
        Self {
            model,
            group_disabled: false,
            items: (0..count)
                .map(|_| RadioGroupA11ySlot {
                    value: Arc::from(""),
                    label: Arc::from(""),
                    disabled: false,
                    selected: false,
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
        self.items
            .get(index)
            .is_some_and(|it| !self.group_disabled && !it.disabled)
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

struct RadioGroupA11yItem {
    index: usize,
    a11y: Rc<RefCell<RadioGroupA11yState>>,
    pressed: bool,
}

impl RadioGroupA11yItem {
    fn new(index: usize, a11y: Rc<RefCell<RadioGroupA11yState>>) -> Self {
        Self {
            index,
            a11y,
            pressed: false,
        }
    }
}

impl<H: UiHost> Widget<H> for RadioGroupA11yItem {
    fn is_focusable(&self) -> bool {
        self.a11y.borrow().is_item_enabled(self.index)
    }

    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn hit_test_children(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn semantics(&mut self, cx: &mut fret_ui::widget::SemanticsCx<'_, H>) {
        let state = self.a11y.borrow();
        let Some(slot) = state.items.get(self.index) else {
            cx.set_role(SemanticsRole::Generic);
            cx.set_disabled(true);
            return;
        };

        let disabled = !state.is_item_enabled(self.index);
        cx.set_role(SemanticsRole::ListItem);
        cx.set_label(slot.label.to_string());
        cx.set_disabled(disabled);
        cx.set_selected(slot.selected);
        cx.set_checked(Some(slot.selected));
        cx.set_focusable(!disabled);
        cx.set_invokable(!disabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
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
                    KeyCode::ArrowUp | KeyCode::ArrowLeft => {
                        if let Some(target) = self.a11y.borrow().focus_delta_from(self.index, -1) {
                            cx.request_focus(target);
                        }
                        cx.stop_propagation();
                    }
                    KeyCode::ArrowDown | KeyCode::ArrowRight => {
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

                let (model, value) = {
                    let state = self.a11y.borrow();
                    if !state.is_item_enabled(self.index) {
                        return;
                    }
                    let Some(slot) = state.items.get(self.index) else {
                        return;
                    };
                    (state.model, slot.value.clone())
                };

                let _ = cx.app.models_mut().update(model, |v| *v = Some(value));
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

/// Installs a `RadioGroup` with per-item semantics nodes for assistive technologies.
pub fn install_radio_group<H: UiHost>(
    ui: &mut UiTree<H>,
    parent: NodeId,
    group: RadioGroup,
) -> NodeId {
    let count = group.items.len();
    let a11y = Rc::new(RefCell::new(RadioGroupA11yState::new(group.model, count)));

    let root = ui.create_node(group.with_a11y(a11y.clone()));
    ui.add_child(parent, root);

    for index in 0..count {
        let node = ui.create_node(RadioGroupA11yItem::new(index, a11y.clone()));
        ui.add_child(root, node);
        a11y.borrow_mut().set_node(index, node);
    }

    root
}

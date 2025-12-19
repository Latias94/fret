use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp,
    Size, TextConstraints, TextStyle, TextWrap,
};

#[derive(Debug, Clone)]
pub struct VirtualListStyle {
    pub padding_x: Px,
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
    pub row_hover: Color,
    pub row_selected: Color,
    pub text_color: Color,
    pub text_style: TextStyle,
}

impl Default for VirtualListStyle {
    fn default() -> Self {
        Self {
            padding_x: Px(8.0),
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
                a: 0.35,
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
            text_style: TextStyle {
                font: fret_core::FontId::default(),
                size: Px(13.0),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct VirtualListRow {
    pub text: String,
    pub indent_x: Px,
}

impl VirtualListRow {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            indent_x: Px(0.0),
        }
    }

    pub fn with_indent_x(mut self, indent_x: Px) -> Self {
        self.indent_x = indent_x;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VisibleRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct PreparedRow {
    index: usize,
    blob: fret_core::TextBlobId,
    metrics: fret_core::TextMetrics,
}

#[derive(Debug)]
pub struct VirtualList {
    rows: Vec<VirtualListRow>,
    row_height: Px,
    style: VirtualListStyle,

    offset_y: Px,
    dragging_thumb: bool,
    drag_pointer_start_y: Px,
    drag_offset_start_y: Px,

    hovered: Option<usize>,
    selected: Option<usize>,

    last_bounds: Rect,
    last_content_height: Px,
    last_viewport_height: Px,
    last_visible: VisibleRange,
    prepared: Vec<PreparedRow>,
    last_prepared_width: Px,
    prepared_dirty: bool,
}

impl VirtualList {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            rows: items.into_iter().map(VirtualListRow::new).collect(),
            row_height: Px(20.0),
            style: VirtualListStyle::default(),
            offset_y: Px(0.0),
            dragging_thumb: false,
            drag_pointer_start_y: Px(0.0),
            drag_offset_start_y: Px(0.0),
            hovered: None,
            selected: None,
            last_bounds: Rect::default(),
            last_content_height: Px(0.0),
            last_viewport_height: Px(0.0),
            last_visible: VisibleRange { start: 0, end: 0 },
            prepared: Vec::new(),
            last_prepared_width: Px(0.0),
            prepared_dirty: false,
        }
    }

    pub fn with_row_height(mut self, height: Px) -> Self {
        self.row_height = height;
        self
    }

    pub fn with_style(mut self, style: VirtualListStyle) -> Self {
        self.style = style;
        self
    }

    pub fn style(&self) -> &VirtualListStyle {
        &self.style
    }

    pub fn row_height(&self) -> Px {
        self.row_height
    }

    pub fn offset_y(&self) -> Px {
        self.offset_y
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn set_rows(&mut self, rows: Vec<VirtualListRow>) {
        self.rows = rows;
        self.hovered = None;
        self.prepared_dirty = true;

        if let Some(selected) = self.selected {
            if selected >= self.rows.len() {
                self.selected = None;
            }
        }
        self.clamp_offset();
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.set_rows(items.into_iter().map(VirtualListRow::new).collect());
    }

    fn max_offset(&self) -> Px {
        Px((self.last_content_height.0 - self.last_viewport_height.0).max(0.0))
    }

    fn clamp_offset(&mut self) {
        let max = self.max_offset();
        self.offset_y = Px(self.offset_y.0.clamp(0.0, max.0));
    }

    pub fn content_bounds(&self) -> Rect {
        const SCROLLBAR_W: Px = Px(10.0);
        if self.last_content_height.0 > self.last_viewport_height.0 {
            Rect::new(
                self.last_bounds.origin,
                Size::new(
                    Px((self.last_bounds.size.width.0 - SCROLLBAR_W.0).max(0.0)),
                    self.last_bounds.size.height,
                ),
            )
        } else {
            self.last_bounds
        }
    }

    fn row_index_from_y(&self, local_y: Px) -> Option<usize> {
        if self.rows.is_empty() || self.row_height.0 <= 0.0 {
            return None;
        }
        let y = (local_y.0 + self.offset_y.0).max(0.0);
        let idx = (y / self.row_height.0).floor() as isize;
        if idx < 0 {
            return None;
        }
        let idx = idx as usize;
        if idx >= self.rows.len() {
            return None;
        }
        Some(idx)
    }

    pub fn row_index_at(&self, position: fret_core::Point) -> Option<usize> {
        let content = self.content_bounds();
        if !content.contains(position) {
            return None;
        }
        let local_y = Px(position.y.0 - content.origin.y.0);
        self.row_index_from_y(local_y)
    }

    pub fn ensure_visible(&mut self, index: usize) {
        if self.row_height.0 <= 0.0 || self.last_viewport_height.0 <= 0.0 {
            return;
        }
        let row_top = index as f32 * self.row_height.0;
        let row_bottom = row_top + self.row_height.0;
        let viewport_top = self.offset_y.0;
        let viewport_bottom = self.offset_y.0 + self.last_viewport_height.0;

        if row_top < viewport_top {
            self.offset_y = Px(row_top);
        } else if row_bottom > viewport_bottom {
            self.offset_y = Px(row_bottom - self.last_viewport_height.0);
        }
        self.clamp_offset();
    }

    fn scrollbar_geometry(&self) -> Option<(Rect, Rect)> {
        let viewport_h = self.last_viewport_height;
        if viewport_h.0 <= 0.0 {
            return None;
        }

        let content_h = self.last_content_height;
        if content_h.0 <= viewport_h.0 {
            return None;
        }

        let w = Px(10.0);
        let track = Rect::new(
            fret_core::Point::new(
                Px(self.last_bounds.origin.x.0 + self.last_bounds.size.width.0 - w.0),
                self.last_bounds.origin.y,
            ),
            Size::new(w, self.last_bounds.size.height),
        );

        let ratio = (viewport_h.0 / content_h.0).clamp(0.0, 1.0);
        let min_thumb = 24.0;
        let thumb_h = Px((viewport_h.0 * ratio).max(min_thumb).min(viewport_h.0));

        let max_offset = self.max_offset().0;
        let t = if max_offset <= 0.0 {
            0.0
        } else {
            (self.offset_y.0 / max_offset).clamp(0.0, 1.0)
        };
        let travel = (viewport_h.0 - thumb_h.0).max(0.0);
        let thumb_y = Px(track.origin.y.0 + travel * t);

        let thumb = Rect::new(
            fret_core::Point::new(track.origin.x, thumb_y),
            Size::new(w, thumb_h),
        );

        Some((track, thumb))
    }

    fn set_offset_from_thumb_y(&mut self, thumb_top_y: Px) {
        let Some((track, thumb)) = self.scrollbar_geometry() else {
            return;
        };

        let viewport_h = self.last_viewport_height.0;
        let travel = (viewport_h - thumb.size.height.0).max(0.0);
        if travel <= 0.0 {
            self.offset_y = Px(0.0);
            return;
        }

        let t = ((thumb_top_y.0 - track.origin.y.0) / travel).clamp(0.0, 1.0);
        let max = self.max_offset().0;
        self.offset_y = Px(max * t);
    }

    fn compute_visible_range(&self) -> VisibleRange {
        if self.rows.is_empty() || self.row_height.0 <= 0.0 || self.last_viewport_height.0 <= 0.0 {
            return VisibleRange { start: 0, end: 0 };
        }

        let start = (self.offset_y.0 / self.row_height.0).floor().max(0.0) as usize;
        let viewport_rows = (self.last_viewport_height.0 / self.row_height.0).ceil() as usize;
        let overscan = 2usize;
        let start = start.saturating_sub(overscan);
        let end = (start + viewport_rows + overscan * 2).min(self.rows.len());
        VisibleRange { start, end }
    }

    fn rebuild_prepared_rows(&mut self, cx: &mut LayoutCx<'_>, width: Px) {
        for row in self.prepared.drain(..) {
            cx.text.release(row.blob);
        }

        let visible = self.compute_visible_range();
        self.last_visible = visible;
        self.last_prepared_width = width;

        if visible.start >= visible.end {
            return;
        }

        for i in visible.start..visible.end {
            let indent_x = self.rows[i].indent_x.0;
            let max_width = Px((width.0 - self.style.padding_x.0 * 2.0 - indent_x).max(0.0));
            let constraints = TextConstraints {
                max_width: Some(max_width),
                wrap: TextWrap::None,
                scale_factor: cx.scale_factor,
            };
            let (blob, metrics) =
                cx.text
                    .prepare(&self.rows[i].text, self.style.text_style, constraints);
            self.prepared.push(PreparedRow {
                index: i,
                blob,
                metrics,
            });
        }
    }

    fn update_hover(&mut self, content: Rect, position: fret_core::Point) -> bool {
        if !content.contains(position) {
            if self.hovered.take().is_some() {
                return true;
            }
            return false;
        }
        let local_y = Px(position.y.0 - content.origin.y.0);
        let next = self.row_index_from_y(local_y);
        if next != self.hovered {
            self.hovered = next;
            return true;
        }
        false
    }

    fn handle_keyboard_nav(&mut self, key: KeyCode, modifiers: Modifiers) -> bool {
        if modifiers.ctrl || modifiers.meta || modifiers.alt {
            return false;
        }

        if self.rows.is_empty() {
            return false;
        }

        let current = self
            .selected
            .unwrap_or(0)
            .min(self.rows.len().saturating_sub(1));
        let viewport_rows = if self.row_height.0 <= 0.0 {
            1
        } else {
            (self.last_viewport_height.0 / self.row_height.0)
                .floor()
                .max(1.0) as usize
        };

        let next = match key {
            KeyCode::ArrowUp => current.saturating_sub(1),
            KeyCode::ArrowDown => (current + 1).min(self.rows.len().saturating_sub(1)),
            KeyCode::Home => 0,
            KeyCode::End => self.rows.len().saturating_sub(1),
            KeyCode::PageUp => current.saturating_sub(viewport_rows),
            KeyCode::PageDown => (current + viewport_rows).min(self.rows.len().saturating_sub(1)),
            _ => return false,
        };

        self.selected = Some(next);
        self.ensure_visible(next);
        true
    }
}

impl Widget for VirtualList {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Wheel { delta, .. } => {
                    self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                    self.clamp_offset();
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Down {
                    position, button, ..
                } => {
                    if *button != MouseButton::Left {
                        return;
                    }

                    if let Some((track, thumb)) = self.scrollbar_geometry() {
                        if track.contains(*position) {
                            if thumb.contains(*position) {
                                self.dragging_thumb = true;
                                self.drag_pointer_start_y = position.y;
                                self.drag_offset_start_y = self.offset_y;
                                cx.capture_pointer(cx.node);
                            } else {
                                let centered = Px(position.y.0 - thumb.size.height.0 * 0.5);
                                self.set_offset_from_thumb_y(centered);
                                self.clamp_offset();
                            }

                            cx.invalidate_self(Invalidation::Layout);
                            cx.invalidate_self(Invalidation::Paint);
                            cx.request_redraw();
                            cx.stop_propagation();
                            return;
                        }
                    }

                    let content = self.content_bounds();
                    if !content.contains(*position) {
                        return;
                    }

                    cx.request_focus(cx.node);
                    let local_y = Px(position.y.0 - content.origin.y.0);
                    if let Some(idx) = self.row_index_from_y(local_y) {
                        self.selected = Some(idx);
                        self.ensure_visible(idx);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                    cx.stop_propagation();
                }
                fret_core::PointerEvent::Move { position, .. } => {
                    if self.dragging_thumb && cx.captured == Some(cx.node) {
                        let dy = position.y.0 - self.drag_pointer_start_y.0;
                        let Some((_, thumb)) = self.scrollbar_geometry() else {
                            return;
                        };

                        let max_offset = self.max_offset().0;
                        let travel = (self.last_viewport_height.0 - thumb.size.height.0).max(0.0);
                        if travel <= 0.0 || max_offset <= 0.0 {
                            return;
                        }

                        let offset_delta = dy / travel * max_offset;
                        self.offset_y = Px(self.drag_offset_start_y.0 + offset_delta);
                        self.clamp_offset();
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }

                    let content = self.content_bounds();
                    if self.update_hover(content, *position) {
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }
                }
                fret_core::PointerEvent::Up { button, .. } => {
                    if *button != MouseButton::Left {
                        return;
                    }
                    if self.dragging_thumb && cx.captured == Some(cx.node) {
                        self.dragging_thumb = false;
                        cx.release_pointer_capture();
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                        cx.stop_propagation();
                    }
                }
            },
            Event::KeyDown { key, modifiers, .. } => {
                if cx.focus != Some(cx.node) {
                    return;
                }
                if self.handle_keyboard_nav(*key, *modifiers) {
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        self.last_bounds = cx.bounds;

        if self.prepared_dirty {
            for row in self.prepared.drain(..) {
                cx.text.release(row.blob);
            }
            self.prepared_dirty = false;
            self.last_visible = VisibleRange { start: 0, end: 0 };
            self.last_prepared_width = Px(0.0);
        }

        let count = self.rows.len();
        self.last_content_height = Px(count as f32 * self.row_height.0);
        self.last_viewport_height = cx.available.height;
        self.clamp_offset();

        let content = self.content_bounds();
        let visible = self.compute_visible_range();
        if visible != self.last_visible || content.size.width != self.last_prepared_width {
            self.rebuild_prepared_rows(cx, content.size.width);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.last_bounds = cx.bounds;

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.style.background,
            border: self.style.border,
            border_color: self.style.border_color,
            corner_radii: self.style.corner_radii,
        });

        let content = self.content_bounds();
        cx.scene.push(SceneOp::PushClipRect { rect: content });

        let row_h = self.row_height;
        for row in &self.prepared {
            let y = content.origin.y.0 + row.index as f32 * row_h.0 - self.offset_y.0;
            let row_rect = Rect::new(
                fret_core::Point::new(content.origin.x, Px(y)),
                Size::new(content.size.width, row_h),
            );

            let is_selected = self.selected == Some(row.index);
            let is_hovered = self.hovered == Some(row.index);

            if is_selected || is_hovered {
                let bg = if is_selected {
                    self.style.row_selected
                } else {
                    self.style.row_hover
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(0),
                    rect: row_rect,
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(0.0)),
                });
            }

            let indent_x = self
                .rows
                .get(row.index)
                .map(|r| r.indent_x.0)
                .unwrap_or(0.0);
            let text_x = Px(row_rect.origin.x.0 + self.style.padding_x.0 + indent_x);
            let inner_y = row_rect.origin.y.0 + ((row_h.0 - row.metrics.size.height.0) * 0.5);
            let text_y = Px(inner_y + row.metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(0),
                origin: fret_core::Point::new(text_x, text_y),
                text: row.blob,
                color: self.style.text_color,
            });
        }

        cx.scene.push(SceneOp::PopClip);

        if let Some((track, thumb)) = self.scrollbar_geometry() {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(100),
                rect: track,
                background: Color {
                    r: 0.10,
                    g: 0.10,
                    b: 0.11,
                    a: 0.9,
                },
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(6.0)),
            });

            let thumb_bg = if self.dragging_thumb {
                Color {
                    r: 0.55,
                    g: 0.55,
                    b: 0.58,
                    a: 0.9,
                }
            } else {
                Color {
                    r: 0.42,
                    g: 0.42,
                    b: 0.45,
                    a: 0.9,
                }
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(101),
                rect: thumb,
                background: thumb_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(6.0)),
            });
        }
    }
}

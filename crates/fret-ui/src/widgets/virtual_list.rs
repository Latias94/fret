use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, KeyCode, Modifiers, MouseButton, Px, Rect, SceneOp,
    Size, TextConstraints, TextStyle, TextWrap,
};
use std::{borrow::Cow, hash::Hash};

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
pub struct VirtualListRow<'a> {
    pub text: Cow<'a, str>,
    pub indent_x: Px,
}

impl<'a> VirtualListRow<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
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

pub trait VirtualListDataSource {
    type Key: Copy + Eq + Hash;

    fn len(&self) -> usize;
    fn key_at(&self, index: usize) -> Self::Key;
    fn row_at(&self, index: usize) -> VirtualListRow<'_>;

    fn index_of_key(&self, key: Self::Key) -> Option<usize> {
        let len = self.len();
        for i in 0..len {
            if self.key_at(i) == key {
                return Some(i);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct VecStringDataSource {
    items: Vec<String>,
}

impl VecStringDataSource {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

impl VirtualListDataSource for VecStringDataSource {
    type Key = usize;

    fn len(&self) -> usize {
        self.items.len()
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        VirtualListRow::new(self.items[index].as_str())
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
    indent_x: Px,
    blob: fret_core::TextBlobId,
    metrics: fret_core::TextMetrics,
}

#[derive(Debug)]
pub struct VirtualList<D: VirtualListDataSource> {
    data: D,
    row_height: Px,
    style: VirtualListStyle,

    offset_y: Px,
    dragging_thumb: bool,
    drag_pointer_start_y: Px,
    drag_offset_start_y: Px,

    hovered: Option<usize>,
    selected_key: Option<D::Key>,
    selected_index: Option<usize>,

    last_bounds: Rect,
    last_content_height: Px,
    last_viewport_height: Px,
    last_visible: VisibleRange,
    prepared: Vec<PreparedRow>,
    last_prepared_width: Px,
    prepared_dirty: bool,
}

impl VirtualList<VecStringDataSource> {
    pub fn from_items(items: Vec<String>) -> Self {
        Self::new(VecStringDataSource::new(items))
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.set_data(VecStringDataSource::new(items));
    }
}

impl<D: VirtualListDataSource> VirtualList<D> {
    pub fn new(data: D) -> Self {
        Self {
            data,
            row_height: Px(20.0),
            style: VirtualListStyle::default(),
            offset_y: Px(0.0),
            dragging_thumb: false,
            drag_pointer_start_y: Px(0.0),
            drag_offset_start_y: Px(0.0),
            hovered: None,
            selected_key: None,
            selected_index: None,
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
        self.data.len()
    }

    pub fn selected_key(&self) -> Option<D::Key> {
        self.selected_key
    }

    pub fn set_selected_key(&mut self, key: Option<D::Key>) {
        self.selected_key = key;
        self.selected_index = self
            .selected_key
            .and_then(|selected_key| self.data.index_of_key(selected_key));
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn set_data(&mut self, data: D) {
        self.data = data;
        self.hovered = None;
        self.prepared_dirty = true;
        self.selected_index = self
            .selected_key
            .and_then(|selected_key| self.data.index_of_key(selected_key));
        if self.selected_key.is_some() && self.selected_index.is_none() {
            self.selected_key = None;
        }
        self.clamp_offset();
    }

    fn set_selected_index(&mut self, index: usize) {
        self.selected_index = Some(index);
        self.selected_key = Some(self.data.key_at(index));
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
        if self.data.len() == 0 || self.row_height.0 <= 0.0 {
            return None;
        }
        let y = (local_y.0 + self.offset_y.0).max(0.0);
        let idx = (y / self.row_height.0).floor() as isize;
        if idx < 0 {
            return None;
        }
        let idx = idx as usize;
        if idx >= self.data.len() {
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

    fn prepare_row(
        &mut self,
        text: &mut dyn fret_core::TextService,
        scale_factor: f32,
        width: Px,
        index: usize,
    ) {
        let row = self.data.row_at(index);
        let indent_x = row.indent_x.0;
        let max_width = Px((width.0 - self.style.padding_x.0 * 2.0 - indent_x).max(0.0));
        let constraints = TextConstraints {
            max_width: Some(max_width),
            wrap: TextWrap::None,
            scale_factor,
        };
        let (blob, metrics) = text.prepare(row.text.as_ref(), self.style.text_style, constraints);
        self.prepared.push(PreparedRow {
            index,
            indent_x: row.indent_x,
            blob,
            metrics,
        });
    }

    fn compute_visible_range(&self) -> VisibleRange {
        if self.data.len() == 0 || self.row_height.0 <= 0.0 || self.last_viewport_height.0 <= 0.0 {
            return VisibleRange { start: 0, end: 0 };
        }

        let start = (self.offset_y.0 / self.row_height.0).floor().max(0.0) as usize;
        let viewport_rows = (self.last_viewport_height.0 / self.row_height.0).ceil() as usize;
        let overscan = 2usize;
        let start = start.saturating_sub(overscan);
        let end = (start + viewport_rows + overscan * 2).min(self.data.len());
        VisibleRange { start, end }
    }

    fn ensure_prepared_for_visible(
        &mut self,
        text: &mut dyn fret_core::TextService,
        scale_factor: f32,
        width: Px,
        visible: VisibleRange,
        mut budget: usize,
    ) -> bool {
        if self.prepared_dirty || width != self.last_prepared_width {
            for row in self.prepared.drain(..) {
                text.release(row.blob);
            }
            self.prepared_dirty = false;
            self.last_visible = VisibleRange { start: 0, end: 0 };
            self.last_prepared_width = width;
        }

        if visible.start >= visible.end {
            self.last_visible = visible;
            self.last_prepared_width = width;
            return true;
        }

        self.prepared.retain_mut(|row| {
            if row.index >= visible.start && row.index < visible.end {
                true
            } else {
                text.release(row.blob);
                false
            }
        });

        let mut complete = true;
        for i in visible.start..visible.end {
            if self.prepared.iter().any(|r| r.index == i) {
                continue;
            }
            if budget == 0 {
                complete = false;
                continue;
            }
            self.prepare_row(text, scale_factor, width, i);
            budget -= 1;
        }
        self.prepared.sort_by_key(|r| r.index);

        self.last_visible = visible;
        self.last_prepared_width = width;
        complete
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

        if self.data.len() == 0 {
            return false;
        }

        let current = self
            .selected_index
            .unwrap_or(0)
            .min(self.data.len().saturating_sub(1));
        let viewport_rows = if self.row_height.0 <= 0.0 {
            1
        } else {
            (self.last_viewport_height.0 / self.row_height.0)
                .floor()
                .max(1.0) as usize
        };

        let next = match key {
            KeyCode::ArrowUp => current.saturating_sub(1),
            KeyCode::ArrowDown => (current + 1).min(self.data.len().saturating_sub(1)),
            KeyCode::Home => 0,
            KeyCode::End => self.data.len().saturating_sub(1),
            KeyCode::PageUp => current.saturating_sub(viewport_rows),
            KeyCode::PageDown => (current + viewport_rows).min(self.data.len().saturating_sub(1)),
            _ => return false,
        };

        self.set_selected_index(next);
        self.ensure_visible(next);
        true
    }
}

impl<D: VirtualListDataSource> Widget for VirtualList<D> {
    fn is_focusable(&self) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        match event {
            Event::Pointer(pe) => match pe {
                fret_core::PointerEvent::Wheel { delta, .. } => {
                    self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                    self.clamp_offset();
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
                        self.set_selected_index(idx);
                        self.ensure_visible(idx);
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

        let count = self.data.len();
        self.last_content_height = Px(count as f32 * self.row_height.0);
        self.last_viewport_height = cx.available.height;
        self.clamp_offset();

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.last_bounds = cx.bounds;
        self.last_viewport_height = cx.bounds.size.height;
        self.last_content_height = Px(self.data.len() as f32 * self.row_height.0);
        self.clamp_offset();

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

        let visible = self.compute_visible_range();
        let big_jump = {
            let old = self.last_visible;
            let overlap_start = old.start.max(visible.start);
            let overlap_end = old.end.min(visible.end);
            overlap_start >= overlap_end
        };
        let budget = if self.prepared.is_empty() {
            usize::MAX
        } else if self.dragging_thumb || big_jump {
            8
        } else {
            32
        };
        let complete = self.ensure_prepared_for_visible(
            cx.text,
            cx.scale_factor,
            content.size.width,
            visible,
            budget,
        );

        let row_h = self.row_height;
        for i in visible.start..visible.end {
            let y = content.origin.y.0 + i as f32 * row_h.0 - self.offset_y.0;
            let row_rect = Rect::new(
                fret_core::Point::new(content.origin.x, Px(y)),
                Size::new(content.size.width, row_h),
            );

            let key = self.data.key_at(i);
            let is_selected = self.selected_key == Some(key);
            let is_hovered = self.hovered == Some(i);

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
        }

        for row in &self.prepared {
            let y = content.origin.y.0 + row.index as f32 * row_h.0 - self.offset_y.0;
            let row_rect = Rect::new(
                fret_core::Point::new(content.origin.x, Px(y)),
                Size::new(content.size.width, row_h),
            );

            let text_x = Px(row_rect.origin.x.0 + self.style.padding_x.0 + row.indent_x.0);
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

        if !complete {
            if let Some(window) = cx.window {
                cx.app.request_redraw(window);
            }
        }

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

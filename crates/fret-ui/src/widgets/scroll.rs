use crate::{
    UiHost,
    widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget},
};
use fret_core::{
    Color, Corners, DrawOrder, Edges, Event, MouseButton, Point, PointerEvent, Px, Rect, SceneOp,
    Size,
};

pub struct Scroll {
    offset_y: Px,
    dragging_thumb: bool,
    drag_pointer_start_y: Px,
    drag_offset_start_y: Px,
    last_bounds: Rect,
    last_content_height: Px,
    last_viewport_height: Px,
    scrollbar_width: Px,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            offset_y: Px(0.0),
            dragging_thumb: false,
            drag_pointer_start_y: Px(0.0),
            drag_offset_start_y: Px(0.0),
            last_bounds: Rect::default(),
            last_content_height: Px(0.0),
            last_viewport_height: Px(0.0),
            scrollbar_width: Px(10.0),
        }
    }

    pub fn offset_y(&self) -> Px {
        self.offset_y
    }

    fn max_offset(&self) -> Px {
        Px((self.last_content_height.0 - self.last_viewport_height.0).max(0.0))
    }

    fn clamp_offset(&mut self, content_height: Px, viewport_height: Px) {
        let max = Px((content_height.0 - viewport_height.0).max(0.0));
        self.offset_y = Px(self.offset_y.0.clamp(0.0, max.0));
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

        let w = self.scrollbar_width;
        let track = Rect::new(
            Point::new(
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

        let thumb = Rect::new(Point::new(track.origin.x, thumb_y), Size::new(w, thumb_h));

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
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: UiHost> Widget<H> for Scroll {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        self.scrollbar_width = cx.theme().metrics.scrollbar_width;
        let Event::Pointer(pe) = event else {
            return;
        };

        match pe {
            PointerEvent::Wheel { delta, .. } => {
                self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
                self.clamp_offset(self.last_content_height, self.last_viewport_height);
                cx.invalidate_self(Invalidation::Layout);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            PointerEvent::Down {
                position, button, ..
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                let Some((track, thumb)) = self.scrollbar_geometry() else {
                    return;
                };
                if !track.contains(*position) {
                    return;
                }

                if thumb.contains(*position) {
                    self.dragging_thumb = true;
                    self.drag_pointer_start_y = position.y;
                    self.drag_offset_start_y = self.offset_y;
                    cx.capture_pointer(cx.node);
                } else {
                    let centered = Px(position.y.0 - thumb.size.height.0 * 0.5);
                    self.set_offset_from_thumb_y(centered);
                }

                cx.invalidate_self(Invalidation::Layout);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            PointerEvent::Move { position, .. } => {
                if !self.dragging_thumb {
                    return;
                }

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
                self.clamp_offset(self.last_content_height, self.last_viewport_height);

                cx.invalidate_self(Invalidation::Layout);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            PointerEvent::Up { button, .. } => {
                if *button != MouseButton::Left {
                    return;
                }
                if self.dragging_thumb {
                    self.dragging_thumb = false;
                    cx.release_pointer_capture();
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                }
            }
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.scrollbar_width = cx.theme().metrics.scrollbar_width;
        self.last_bounds = cx.bounds;
        let Some(&child) = cx.children.first() else {
            return cx.available;
        };

        let scrollbar_w = self.scrollbar_width;

        // Measure content with unconstrained height (very simple MVP).
        let mut content_width = cx.available.width;
        let mut content_size = cx.layout_in(
            child,
            Rect::new(cx.bounds.origin, Size::new(content_width, Px(1.0e9))),
        );

        self.last_content_height = content_size.height;
        self.last_viewport_height = cx.available.height;

        let show_scrollbar = content_size.height.0 > cx.available.height.0;
        if show_scrollbar {
            content_width = Px((cx.available.width.0 - scrollbar_w.0).max(0.0));
            content_size = cx.layout_in(
                child,
                Rect::new(cx.bounds.origin, Size::new(content_width, Px(1.0e9))),
            );
            self.last_content_height = content_size.height;
        }

        self.clamp_offset(content_size.height, cx.available.height);

        // Layout child at a translated origin to implement scrolling.
        let origin = Point::new(
            cx.bounds.origin.x,
            Px(cx.bounds.origin.y.0 - self.offset_y.0),
        );
        let child_bounds = Rect::new(origin, Size::new(content_width, content_size.height));
        let _ = cx.layout_in(child, child_bounds);

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.scrollbar_width = cx.theme().metrics.scrollbar_width;
        self.last_bounds = cx.bounds;
        let Some(&child) = cx.children.first() else {
            return;
        };

        cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });

        if let Some(bounds) = cx.child_bounds(child) {
            cx.paint(child, bounds);
        } else {
            cx.paint(child, cx.bounds);
        }

        cx.scene.push(SceneOp::PopClip);

        if let Some((track, thumb)) = self.scrollbar_geometry() {
            let (track_bg, thumb_bg, thumb_hover_bg, radius) = {
                let theme = cx.theme();
                (
                    theme.colors.scrollbar_track,
                    theme.colors.scrollbar_thumb,
                    theme.colors.scrollbar_thumb_hover,
                    theme.metrics.radius_sm,
                )
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(100),
                rect: track,
                background: track_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });

            let thumb_bg = if self.dragging_thumb {
                thumb_hover_bg
            } else {
                thumb_bg
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(101),
                rect: thumb,
                background: thumb_bg,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(radius),
            });
        }
    }
}

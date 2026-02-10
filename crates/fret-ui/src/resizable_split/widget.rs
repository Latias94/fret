use fret_core::{
    Axis, Color, Corners, DrawOrder, Edges, Event, MouseButton, Paint, Point, Px, Rect, Size,
};
use fret_runtime::Model;

use crate::resize_handle::ResizeHandle;
use crate::widget::{EventCx, LayoutCx, PaintCx, Widget};
use crate::{Invalidation, UiHost};

#[derive(Debug, Clone, Copy)]
struct DragState {
    grab_offset: f32,
}

fn split_log(_args: std::fmt::Arguments<'_>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::{
            io::Write,
            sync::{Mutex, OnceLock},
        };

        if std::env::var_os("FRET_RESIZABLE_SPLIT_LOG").is_none() {
            return;
        }

        static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
        let file = LOG_FILE.get_or_init(|| {
            let _ = std::fs::create_dir_all("target");
            let path = std::path::Path::new("target").join("fret-resizable-split.log");
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
                .expect("open fret-resizable-split.log");
            let _ = writeln!(
                file,
                "[session] pid={} time={:?}",
                std::process::id(),
                std::time::SystemTime::now()
            );
            Mutex::new(file)
        });

        let Ok(mut file) = file.lock() else {
            return;
        };
        let _ = writeln!(file, "{}", _args);
    }
}

/// A simple two-panel resizable split primitive.
///
/// This is a runtime substrate widget: it owns pointer capture, cursor affordances, and the split
/// fraction clamping rules. Higher-level panel groups and editor docking policies should live in
/// component/app layers.
pub struct ResizableSplit {
    axis: Axis,
    fraction: Model<f32>,
    min_px: Px,
    hit_thickness: Px,
    paint_device_px: f32,
    dragging: Option<DragState>,
    last_bounds: Rect,
    last_handle_rect: Rect,
}

impl ResizableSplit {
    pub fn new(axis: Axis, fraction: Model<f32>) -> Self {
        Self {
            axis,
            fraction,
            min_px: Px(120.0),
            hit_thickness: Px(6.0),
            paint_device_px: 1.0,
            dragging: None,
            last_bounds: Rect::default(),
            last_handle_rect: Rect::default(),
        }
    }

    pub fn with_min_px(mut self, min_px: Px) -> Self {
        self.min_px = min_px;
        self
    }

    pub fn with_hit_thickness(mut self, thickness: Px) -> Self {
        self.hit_thickness = thickness;
        self
    }

    pub fn with_paint_device_px(mut self, px: f32) -> Self {
        self.paint_device_px = px;
        self
    }

    fn axis_len(bounds: Rect, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => bounds.size.width.0,
            Axis::Vertical => bounds.size.height.0,
        }
    }

    fn axis_origin(bounds: Rect, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => bounds.origin.x.0,
            Axis::Vertical => bounds.origin.y.0,
        }
    }

    fn axis_pos(pos: Point, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => pos.x.0,
            Axis::Vertical => pos.y.0,
        }
    }

    fn clamp_fraction(&self, fraction: f32, bounds: Rect) -> f32 {
        let gap = self.hit_thickness.0.max(0.0);
        let axis_len = Self::axis_len(bounds, self.axis);
        let avail = (axis_len - gap).max(0.0);
        if avail <= 0.0 {
            return 0.5;
        }

        let min = self.min_px.0.max(0.0);
        if avail <= min * 2.0 {
            return 0.5;
        }

        let left = (avail * fraction.clamp(0.0, 1.0)).clamp(min, (avail - min).max(min));
        (left / avail).clamp(0.0, 1.0)
    }

    fn compute_layout(&self, bounds: Rect, fraction: f32) -> (Rect, Rect, Rect, f32) {
        let gap = self.hit_thickness.0.max(0.0);
        let axis_len = Self::axis_len(bounds, self.axis);
        let avail = (axis_len - gap).max(0.0);

        let f = self.clamp_fraction(fraction, bounds);
        let left_len = avail * f;
        let right_len = (avail - left_len).max(0.0);

        match self.axis {
            Axis::Horizontal => {
                let rect_a = Rect::new(bounds.origin, Size::new(Px(left_len), bounds.size.height));
                let handle_rect = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + left_len), bounds.origin.y),
                    Size::new(Px(gap), bounds.size.height),
                );
                let rect_b = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + left_len + gap), bounds.origin.y),
                    Size::new(Px(right_len), bounds.size.height),
                );
                let center = bounds.origin.x.0 + left_len + gap * 0.5;
                (rect_a, rect_b, handle_rect, center)
            }
            Axis::Vertical => {
                let rect_a = Rect::new(bounds.origin, Size::new(bounds.size.width, Px(left_len)));
                let handle_rect = Rect::new(
                    Point::new(bounds.origin.x, Px(bounds.origin.y.0 + left_len)),
                    Size::new(bounds.size.width, Px(gap)),
                );
                let rect_b = Rect::new(
                    Point::new(bounds.origin.x, Px(bounds.origin.y.0 + left_len + gap)),
                    Size::new(bounds.size.width, Px(right_len)),
                );
                let center = bounds.origin.y.0 + left_len + gap * 0.5;
                (rect_a, rect_b, handle_rect, center)
            }
        }
    }
}

impl<H: UiHost> Widget<H> for ResizableSplit {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Event::Pointer(pe) = event else {
            return;
        };

        self.last_bounds = cx.bounds;

        let fraction = cx.app.models().get_copied(&self.fraction).unwrap_or(0.5);
        let (_a, _b, handle_rect, handle_center) = self.compute_layout(cx.bounds, fraction);
        self.last_handle_rect = handle_rect;

        match pe {
            fret_core::PointerEvent::Move { position, .. } => {
                if let Some(drag) = self.dragging {
                    let gap = self.hit_thickness.0.max(0.0);
                    let axis_len = Self::axis_len(cx.bounds, self.axis);
                    let avail = (axis_len - gap).max(0.0);
                    if avail <= 0.0 {
                        return;
                    }

                    let min = self.min_px.0.max(0.0);
                    if avail <= min * 2.0 {
                        return;
                    }

                    let origin = Self::axis_origin(cx.bounds, self.axis);
                    let pos = Self::axis_pos(*position, self.axis);
                    let center = pos - drag.grab_offset;
                    let left = (center - origin - gap * 0.5).clamp(min, (avail - min).max(min));
                    let next = (left / avail).clamp(0.0, 1.0);

                    let _ = cx.app.models_mut().update(&self.fraction, |v| {
                        *v = next;
                    });

                    split_log(format_args!(
                        "move dragging=true node={:?} captured={:?} pos={:?} bounds={:?} center={:.2} next={:.4}",
                        cx.node, cx.captured, position, cx.bounds, handle_center, next
                    ));

                    cx.set_cursor_icon(
                        ResizeHandle {
                            axis: self.axis,
                            hit_thickness: self.hit_thickness,
                            paint_device_px: self.paint_device_px,
                        }
                        .cursor_icon(),
                    );
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if self.last_handle_rect.contains(*position) {
                    cx.set_cursor_icon(
                        ResizeHandle {
                            axis: self.axis,
                            hit_thickness: self.hit_thickness,
                            paint_device_px: self.paint_device_px,
                        }
                        .cursor_icon(),
                    );
                    split_log(format_args!(
                        "move hover=true node={:?} pos={:?} handle={:?} bounds={:?}",
                        cx.node, position, self.last_handle_rect, self.last_bounds
                    ));
                }
            }
            fret_core::PointerEvent::Down {
                position, button, ..
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                if !self.last_handle_rect.contains(*position) {
                    split_log(format_args!(
                        "down hit=false node={:?} pos={:?} handle={:?} bounds={:?}",
                        cx.node, position, self.last_handle_rect, self.last_bounds
                    ));
                    return;
                }

                cx.capture_pointer(cx.node);
                self.dragging = Some(DragState {
                    grab_offset: Self::axis_pos(*position, self.axis) - handle_center,
                });

                split_log(format_args!(
                    "down dragging=true node={:?} pos={:?} handle={:?} bounds={:?}",
                    cx.node, position, self.last_handle_rect, self.last_bounds
                ));

                cx.stop_propagation();
            }
            fret_core::PointerEvent::Up { button, .. } => {
                if *button != MouseButton::Left {
                    return;
                }
                if self.dragging.is_none() {
                    return;
                }

                cx.release_pointer_capture();
                self.dragging = None;
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;

        let fraction = cx.app.models().get_copied(&self.fraction).unwrap_or(0.5);
        let (rect_a, rect_b, handle_rect, _center) = self.compute_layout(cx.bounds, fraction);
        self.last_handle_rect = handle_rect;

        if cx.pass_kind == crate::layout_pass::LayoutPassKind::Final {
            if !cx.children.is_empty() {
                cx.solve_barrier_child_root_if_needed(cx.children[0], rect_a);
            }
            if cx.children.len() >= 2 {
                cx.solve_barrier_child_root_if_needed(cx.children[1], rect_b);
            }
        }

        if !cx.children.is_empty() {
            let _ = cx.layout_in(cx.children[0], rect_a);
        }
        if cx.children.len() >= 2 {
            let _ = cx.layout_in(cx.children[1], rect_b);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children.iter().take(2) {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }

        // Minimal handle affordance (debug-friendly): a thin quad centered in the handle rect.
        if self.hit_thickness.0 > 0.0 {
            let rect = self.last_handle_rect;
            let thickness = Px((self.paint_device_px.max(1.0)).recip().max(1.0));
            let line = match self.axis {
                Axis::Horizontal => Rect::new(
                    Point::new(
                        Px(rect.origin.x.0 + rect.size.width.0 * 0.5 - thickness.0 * 0.5),
                        rect.origin.y,
                    ),
                    Size::new(thickness, rect.size.height),
                ),
                Axis::Vertical => Rect::new(
                    Point::new(
                        rect.origin.x,
                        Px(rect.origin.y.0 + rect.size.height.0 * 0.5 - thickness.0 * 0.5),
                    ),
                    Size::new(rect.size.width, thickness),
                ),
            };

            cx.scene.push(fret_core::SceneOp::Quad {
                order: DrawOrder(10_000),
                rect: line,
                background: Paint::Solid(Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: if self.dragging.is_some() { 0.35 } else { 0.18 },
                }),
                border: Edges::all(Px(0.0)),
                border_paint: Paint::Solid(Color::TRANSPARENT),
                corner_radii: Corners::all(Px(0.0)),
            });
        }
    }
}

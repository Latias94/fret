use fret_core::{Axis, DrawOrder, Event, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::{EventCx, Invalidation, LayoutCx, PaintCx, ResizeHandle, UiHost, Widget};

#[derive(Debug, Clone, Copy)]
struct DragState {
    grab_offset: f32,
}

fn split_log(args: std::fmt::Arguments<'_>) {
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
        let _ = writeln!(file, "{}", args);
    }
}

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

        // Keep cached geometry in sync even if the tree uses "origin translation" fast paths
        // (i.e. layout is skipped and bounds are shifted in-place).
        self.last_bounds = cx.bounds;

        // Keep hit geometry stable even if layout isn't recomputed this frame.
        let fraction = cx.app.models().get(self.fraction).copied().unwrap_or(0.5);
        let (_a, _b, handle_rect, handle_center) = self.compute_layout(cx.bounds, fraction);
        self.last_handle_rect = handle_rect;

        match pe {
            fret_core::PointerEvent::Move {
                position,
                buttons: _buttons,
                ..
            } => {
                if self.dragging.is_some() {
                    let drag = self.dragging.expect("dragging implies DragState");

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

                    let _ = cx.app.models_mut().update(self.fraction, |v| {
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

                let gap = self.hit_thickness.0.max(0.0);
                let axis_len = Self::axis_len(cx.bounds, self.axis);
                let avail = (axis_len - gap).max(0.0);
                if avail <= 0.0 {
                    return;
                }

                let origin = Self::axis_origin(cx.bounds, self.axis);
                let center = if self.last_handle_rect.size.width.0 > 0.0
                    || self.last_handle_rect.size.height.0 > 0.0
                {
                    Self::axis_origin(self.last_handle_rect, self.axis) + gap * 0.5
                } else {
                    origin + avail * 0.5
                };
                let grab_offset = Self::axis_pos(*position, self.axis) - center;

                self.dragging = Some(DragState { grab_offset });
                cx.capture_pointer(cx.node);
                cx.set_cursor_icon(
                    ResizeHandle {
                        axis: self.axis,
                        hit_thickness: self.hit_thickness,
                        paint_device_px: self.paint_device_px,
                    }
                    .cursor_icon(),
                );
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                split_log(format_args!(
                    "down hit=true node={:?} captured(before)={:?} pos={:?} handle={:?} grab_offset={:.2} fraction={:.4}",
                    cx.node, cx.captured, position, self.last_handle_rect, grab_offset, fraction
                ));
                cx.stop_propagation();
            }
            fret_core::PointerEvent::Up { button, .. } => {
                if *button == MouseButton::Left && cx.captured == Some(cx.node) {
                    self.dragging = None;
                    cx.release_pointer_capture();
                    split_log(format_args!(
                        "up node={:?} captured={:?} dragging_cleared=true",
                        cx.node, cx.captured
                    ));
                    cx.stop_propagation();
                }
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        self.last_bounds = cx.bounds;
        cx.observe_model(self.fraction, Invalidation::Layout);

        let Some((&a, rest)) = cx.children.split_first() else {
            self.last_handle_rect = Rect::default();
            return cx.available;
        };
        let Some(&b) = rest.first() else {
            self.last_handle_rect = Rect::default();
            let _ = cx.layout_in(a, cx.bounds);
            return cx.available;
        };

        let fraction = cx.app.models().get(self.fraction).copied().unwrap_or(0.5);
        let (rect_a, rect_b, handle_rect, _center) = self.compute_layout(cx.bounds, fraction);
        self.last_handle_rect = handle_rect;

        let _ = cx.layout_in(a, rect_a);
        let _ = cx.layout_in(b, rect_b);
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some((&a, rest)) = cx.children.split_first() else {
            return;
        };
        let b = rest.first().copied();

        // Paint children first; handle is drawn after as chrome.
        if let Some(bounds) = cx.child_bounds(a) {
            cx.paint(a, bounds);
        } else {
            cx.paint(a, cx.bounds);
        }
        if let Some(b) = b {
            if let Some(bounds) = cx.child_bounds(b) {
                cx.paint(b, bounds);
            } else {
                cx.paint(b, cx.bounds);
            }
        }

        if self.last_handle_rect.size.width.0 <= 0.0 && self.last_handle_rect.size.height.0 <= 0.0 {
            return;
        }

        let theme = cx.theme().snapshot();
        let background = if self.dragging.is_some() {
            theme.colors.focus_ring
        } else {
            theme.colors.panel_border
        };

        let center = match self.axis {
            Axis::Horizontal => self.last_handle_rect.origin.x.0 + self.hit_thickness.0 * 0.5,
            Axis::Vertical => self.last_handle_rect.origin.y.0 + self.hit_thickness.0 * 0.5,
        };

        ResizeHandle {
            axis: self.axis,
            hit_thickness: self.hit_thickness,
            paint_device_px: self.paint_device_px,
        }
        .paint(
            cx.scene,
            // Paint the divider below focus rings (DrawOrder(1)) so it doesn't "cut" focus outlines.
            DrawOrder(0),
            cx.bounds,
            center,
            cx.scale_factor,
            background,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiTree;
    use crate::test_host::TestHost;
    use fret_core::{
        AppWindowId, Event, PlatformCapabilities, TextConstraints, TextMetrics, TextService,
        TextStyle,
    };
    use fret_runtime::Effect;

    #[derive(Default)]
    struct FakeUiServices {}

    impl TextService for FakeUiServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: TextStyle,
            _constraints: TextConstraints,
        ) -> (fret_core::TextBlobId, TextMetrics) {
            (
                fret_core::TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: fret_core::TextBlobId) {}
    }

    impl fret_core::PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[fret_core::PathCommand],
            _style: fret_core::PathStyle,
            _constraints: fret_core::PathConstraints,
        ) -> (fret_core::PathId, fret_core::PathMetrics) {
            (
                fret_core::PathId::default(),
                fret_core::PathMetrics::default(),
            )
        }

        fn release(&mut self, _path: fret_core::PathId) {}
    }

    impl fret_core::SvgService for FakeUiServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
            fret_core::SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
            false
        }
    }

    struct Leaf;

    impl<H: UiHost> Widget<H> for Leaf {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    #[test]
    fn resizable_split_hover_sets_resize_cursor() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let fraction = app.models_mut().insert(0.5f32);

        let root = ui.create_node(ResizableSplit::new(Axis::Horizontal, fraction));
        let a = ui.create_node(Leaf);
        let b = ui.create_node(Leaf);
        ui.add_child(root, a);
        ui.add_child(root, b);
        ui.set_root(root);

        let mut services = FakeUiServices::default();
        let size = Size::new(Px(400.0), Px(120.0));
        let _ = ui.layout(&mut app, &mut services, root, size, 1.0);
        let _ = app.take_effects();

        // With a fraction of 0.5 and gap=6, handle center should be near x=200.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(200.0), Px(10.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let effects = app.take_effects();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::CursorSetIcon { window: w, icon }
                    if *w == window && *icon == fret_core::CursorIcon::ColResize
            )),
            "expected a resize cursor effect when hovering the split handle"
        );
    }

    #[test]
    fn resizable_split_drag_updates_fraction_model() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        let fraction = app.models_mut().insert(0.5f32);

        let root = ui.create_node(ResizableSplit::new(Axis::Horizontal, fraction));
        let a = ui.create_node(Leaf);
        let b = ui.create_node(Leaf);
        ui.add_child(root, a);
        ui.add_child(root, b);
        ui.set_root(root);

        let mut services = FakeUiServices::default();
        let size = Size::new(Px(400.0), Px(120.0));
        let _ = ui.layout(&mut app, &mut services, root, size, 1.0);

        // Start drag on the handle (near x=200 for a 0.5 split).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(200.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        // Drag right.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(280.0), Px(10.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let updated = app.models().get(fraction).copied().unwrap_or(0.0);
        assert!(
            updated > 0.5,
            "expected drag to increase split fraction, got {updated}"
        );
    }
}

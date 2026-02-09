use fret_core::{Axis, Color, DrawOrder, Event, MouseButton, Px, Rect, Size};
use fret_runtime::{Model, ModelId};

use super::{
    ResizablePanelGroupStyle, apply_handle_delta, compute_resizable_panel_group_layout,
    fractions_from_sizes, layout,
};
use crate::resize_handle::ResizeHandle;
use crate::widget::{EventCx, LayoutCx, PaintCx, SemanticsCx, Widget};
use crate::{Invalidation, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone, Copy)]
struct DragState {
    handle_ix: usize,
    grab_offset: f32,
}

pub struct BoundResizablePanelGroup {
    axis: Axis,
    model: Model<Vec<f32>>,
    enabled: bool,
    min_px: Vec<Px>,
    style: ResizablePanelGroupStyle,
    dragging: Option<DragState>,
    hovered_handle_ix: Option<usize>,
    last_bounds: Rect,
    last_sizes: Vec<f32>,
    last_handle_rects: Vec<Rect>,
    last_handle_centers: Vec<f32>,
}

impl BoundResizablePanelGroup {
    pub fn new(axis: Axis, model: Model<Vec<f32>>) -> Self {
        Self {
            axis,
            model,
            enabled: true,
            min_px: Vec::new(),
            style: ResizablePanelGroupStyle::default(),
            dragging: None,
            hovered_handle_ix: None,
            last_bounds: Rect::default(),
            last_sizes: Vec::new(),
            last_handle_rects: Vec::new(),
            last_handle_centers: Vec::new(),
        }
    }

    pub fn model_id(&self) -> ModelId {
        self.model.id()
    }

    pub fn set_model(&mut self, model: Model<Vec<f32>>) {
        if self.model.id() == model.id() {
            return;
        }
        self.model = model;
        self.dragging = None;
    }

    pub fn set_axis(&mut self, axis: Axis) {
        self.axis = axis;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.dragging = None;
            self.hovered_handle_ix = None;
        }
    }

    pub fn set_min_px(&mut self, min_px: Vec<Px>) {
        self.min_px = min_px;
    }

    pub fn set_style(&mut self, style: ResizablePanelGroupStyle) {
        self.style = style;
    }

    pub fn cleanup_resources(&mut self, _services: &mut dyn fret_core::UiServices) {}

    fn effective_min_px(&self, count: usize, avail: f32) -> Vec<f32> {
        layout::effective_min_px_static(count, avail, &self.min_px)
    }

    fn compute_layout<H: UiHost>(
        &mut self,
        app: &H,
        bounds: Rect,
        children_len: usize,
    ) -> (Vec<Rect>, Vec<Rect>, Vec<f32>, Vec<f32>, f32) {
        let raw = app.models().get_cloned(&self.model).unwrap_or_default();
        let layout = compute_resizable_panel_group_layout(
            self.axis,
            bounds,
            children_len,
            raw,
            self.style.gap,
            self.style.hit_thickness,
            &self.min_px,
        );
        (
            layout.panel_rects,
            layout.handle_hit_rects,
            layout.handle_centers,
            layout.sizes,
            layout.avail,
        )
    }

    fn update_model_sizes<H: UiHost>(&self, app: &mut H, sizes: &[f32], avail: f32) {
        let next = fractions_from_sizes(sizes, avail);
        let _ = app.models_mut().update(&self.model, |v| *v = next);
    }
}

impl<H: UiHost> Widget<H> for BoundResizablePanelGroup {
    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        cx.set_disabled(!self.enabled);
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Event::Pointer(pe) = event else {
            return;
        };

        self.last_bounds = cx.bounds;
        let children_len = cx.children.len();
        if children_len < 2 || self.style.hit_thickness.0 <= 0.0 {
            return;
        }

        let (_panel_rects, handle_rects, handle_centers, sizes, avail) =
            self.compute_layout(cx.app, cx.bounds, children_len);
        self.last_handle_rects = handle_rects;
        self.last_handle_centers = handle_centers;
        self.last_sizes = sizes;

        let mins = self.effective_min_px(children_len, avail);

        match pe {
            fret_core::PointerEvent::Move { position, .. } => {
                if let Some(drag) = self.dragging {
                    let Some(&old_center) = self.last_handle_centers.get(drag.handle_ix) else {
                        return;
                    };
                    let desired_center = layout::axis_pos(*position, self.axis) - drag.grab_offset;
                    let desired_delta = desired_center - old_center;

                    let mut sizes = self.last_sizes.clone();
                    let actual =
                        apply_handle_delta(drag.handle_ix, desired_delta, &mut sizes, &mins);
                    if actual.abs() > 1.0e-6 {
                        self.update_model_sizes(cx.app, &sizes, avail);
                        cx.invalidate_self(Invalidation::Layout);
                        cx.invalidate_self(Invalidation::Paint);
                        cx.request_redraw();
                    }

                    cx.set_cursor_icon(
                        ResizeHandle {
                            axis: self.axis,
                            hit_thickness: self.style.hit_thickness,
                            paint_device_px: self.style.paint_device_px,
                        }
                        .cursor_icon(),
                    );
                    cx.stop_propagation();
                    return;
                }

                if !self.enabled {
                    return;
                }

                let mut hovered = None;
                for (i, rect) in self.last_handle_rects.iter().enumerate() {
                    if rect.contains(*position) {
                        hovered = Some(i);
                        break;
                    }
                }
                if hovered != self.hovered_handle_ix {
                    self.hovered_handle_ix = hovered;
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                if self.hovered_handle_ix.is_some() {
                    cx.set_cursor_icon(
                        ResizeHandle {
                            axis: self.axis,
                            hit_thickness: self.style.hit_thickness,
                            paint_device_px: self.style.paint_device_px,
                        }
                        .cursor_icon(),
                    );
                }
            }
            fret_core::PointerEvent::Down {
                position,
                button: MouseButton::Left,
                ..
            } => {
                if !self.enabled {
                    return;
                }
                let mut picked = None;
                for (i, rect) in self.last_handle_rects.iter().enumerate() {
                    if rect.contains(*position) {
                        picked = Some(i);
                        break;
                    }
                }
                let Some(handle_ix) = picked else {
                    return;
                };
                let Some(&center) = self.last_handle_centers.get(handle_ix) else {
                    return;
                };

                cx.capture_pointer(cx.node);
                self.dragging = Some(DragState {
                    handle_ix,
                    grab_offset: layout::axis_pos(*position, self.axis) - center,
                });
                self.hovered_handle_ix = Some(handle_ix);
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            fret_core::PointerEvent::Up {
                button: MouseButton::Left,
                ..
            } => {
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
        cx.observe_model(&self.model, Invalidation::Layout);

        self.last_bounds = cx.bounds;
        let children_len = cx.children.len();
        let (panel_rects, handle_rects, handle_centers, sizes, _avail) =
            self.compute_layout(cx.app, cx.bounds, children_len);
        self.last_handle_rects = handle_rects;
        self.last_handle_centers = handle_centers;
        self.last_sizes = sizes;

        // Avoid registering viewport roots during "probe" layout passes (important for
        // scroll/virtualized descendants).
        if cx.pass_kind == crate::layout_pass::LayoutPassKind::Final {
            for (&child, &rect) in cx.children.iter().zip(panel_rects.iter()) {
                let _ = cx.layout_viewport_root(child, rect);
            }
            return cx.available;
        }

        for (&child, &rect) in cx.children.iter().zip(panel_rects.iter()) {
            let _ = cx.layout_in(child, rect);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }

        let theme = Theme::global(&*cx.app).snapshot();
        if self.style.handle_color.a <= 0.0 {
            self.style.handle_color = theme
                .color_by_key("border")
                .or_else(|| theme.color_by_key("input"))
                .unwrap_or(theme.colors.panel_border);
        }

        let handle = ResizeHandle {
            axis: self.axis,
            hit_thickness: self.style.hit_thickness,
            paint_device_px: self.style.paint_device_px,
        };

        for (i, center) in self.last_handle_centers.iter().copied().enumerate() {
            let alpha = if let Some(drag) = self.dragging {
                if drag.handle_ix == i {
                    self.style.handle_drag_alpha
                } else {
                    self.style.handle_alpha
                }
            } else if self.hovered_handle_ix == Some(i) {
                self.style.handle_hover_alpha
            } else {
                self.style.handle_alpha
            };
            let color = alpha_mul(self.style.handle_color, alpha);
            handle.paint(
                cx.scene,
                DrawOrder(10_000),
                cx.bounds,
                center,
                cx.scale_factor,
                color,
            );
        }
    }
}

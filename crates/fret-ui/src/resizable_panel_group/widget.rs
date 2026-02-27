use fret_core::{
    Axis, Color, Corners, DrawOrder, Event, KeyCode, MouseButton, Point, Px, Rect, Size,
};
use fret_runtime::{Model, ModelId};

use super::{
    ResizablePanelGroupStyle, apply_handle_delta, compute_resizable_panel_group_layout,
    fractions_from_sizes, layout,
};
use crate::element::{RingPlacement, RingStyle};
use crate::resize_handle::ResizeHandle;
use crate::widget::{EventCx, LayoutCx, PaintCx, SemanticsCx, Widget};
use crate::{Invalidation, Theme, UiHost};

type LayoutResult = (usize, bool, Vec<Rect>, Vec<Rect>, Vec<f32>, Vec<f32>, f32);

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
    last_panel_count: usize,
    last_has_handle_children: bool,
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
            last_panel_count: 0,
            last_has_handle_children: false,
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
    ) -> LayoutResult {
        let raw = match app.models().try_get_cloned(&self.model) {
            Ok(Some(v)) => v,
            Ok(None) => {
                #[cfg(debug_assertions)]
                tracing::warn!(
                    model_id = ?self.model.id(),
                    children_len,
                    "resizable_panel_group: fractions model not found"
                );
                Vec::new()
            }
            Err(err) => {
                #[cfg(debug_assertions)]
                tracing::warn!(
                    ?err,
                    model_id = ?self.model.id(),
                    children_len,
                    "resizable_panel_group: failed to read fractions model"
                );
                Vec::new()
            }
        };

        // `ResizablePanelGroup` can optionally include per-handle children (for semantics or
        // additional decorations) in the pattern:
        // `[panel0, handle0, panel1, handle1, ..., panelN]`.
        //
        // Infer the intended panel count from the fractions model when available; this avoids
        // ambiguity when `children_len == 3` (either 3 panels, or 2 panels + 1 handle child).
        let mut panel_count = if raw.len() >= 2 {
            raw.len()
        } else {
            children_len
        };
        let has_handle_children = children_len == panel_count.saturating_mul(2).saturating_sub(1);
        if !has_handle_children && children_len != panel_count {
            panel_count = children_len;
        }

        let layout = compute_resizable_panel_group_layout(
            self.axis,
            bounds,
            panel_count,
            raw,
            self.style.gap,
            self.style.hit_thickness,
            &self.min_px,
        );
        (
            panel_count,
            has_handle_children,
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
        match event {
            Event::KeyDown { key, modifiers, .. } => {
                if !self.enabled {
                    return;
                }

                let step = if modifiers.shift {
                    32.0
                } else if modifiers.alt {
                    1.0
                } else {
                    8.0
                };

                let desired_delta = match (self.axis, *key) {
                    (Axis::Horizontal, KeyCode::ArrowLeft) => Some(-step),
                    (Axis::Horizontal, KeyCode::ArrowRight) => Some(step),
                    (Axis::Vertical, KeyCode::ArrowUp) => Some(-step),
                    (Axis::Vertical, KeyCode::ArrowDown) => Some(step),
                    _ => None,
                };
                let Some(desired_delta) = desired_delta else {
                    return;
                };

                self.last_bounds = cx.bounds;
                let children_len = cx.children.len();
                if children_len < 2 || self.style.hit_thickness.0 <= 0.0 {
                    return;
                }

                let (
                    panel_count,
                    has_handle_children,
                    _panel_rects,
                    handle_rects,
                    handle_centers,
                    sizes,
                    avail,
                ) = self.compute_layout(cx.app, cx.bounds, children_len);
                self.last_panel_count = panel_count;
                self.last_has_handle_children = has_handle_children;
                self.last_handle_rects = handle_rects;
                self.last_handle_centers = handle_centers;
                self.last_sizes = sizes;

                if !has_handle_children || panel_count < 2 {
                    return;
                }

                let Some(focus) = cx.focus else {
                    return;
                };

                let mut focused_handle_ix = None;
                for handle_ix in 0..panel_count.saturating_sub(1) {
                    let child_ix = handle_ix.saturating_mul(2).saturating_add(1);
                    if cx.children.get(child_ix).copied() == Some(focus) {
                        focused_handle_ix = Some(handle_ix);
                        break;
                    }
                }
                let Some(handle_ix) = focused_handle_ix else {
                    return;
                };

                let mins = self.effective_min_px(panel_count, avail);
                let mut sizes = self.last_sizes.clone();
                let actual = apply_handle_delta(handle_ix, desired_delta, &mut sizes, &mins);
                if actual.abs() > 1.0e-6 {
                    self.update_model_sizes(cx.app, &sizes, avail);
                    cx.invalidate_self(Invalidation::Layout);
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                } else {
                    cx.invalidate_self(Invalidation::Paint);
                    cx.request_redraw();
                }

                cx.stop_propagation();
            }
            Event::Pointer(pe) => {
                self.last_bounds = cx.bounds;
                let children_len = cx.children.len();
                if children_len < 2 || self.style.hit_thickness.0 <= 0.0 {
                    return;
                }

                let (
                    panel_count,
                    has_handle_children,
                    _panel_rects,
                    handle_rects,
                    handle_centers,
                    sizes,
                    avail,
                ) = self.compute_layout(cx.app, cx.bounds, children_len);
                self.last_panel_count = panel_count;
                self.last_has_handle_children = has_handle_children;
                self.last_handle_rects = handle_rects;
                self.last_handle_centers = handle_centers;
                self.last_sizes = sizes;

                if panel_count < 2 {
                    return;
                }

                let mins = self.effective_min_px(panel_count, avail);

                match pe {
                    fret_core::PointerEvent::Move { position, .. } => {
                        if let Some(drag) = self.dragging {
                            let Some(&old_center) = self.last_handle_centers.get(drag.handle_ix)
                            else {
                                return;
                            };
                            let desired_center =
                                layout::axis_pos(*position, self.axis) - drag.grab_offset;
                            let desired_delta = desired_center - old_center;

                            let mut sizes = self.last_sizes.clone();
                            let actual = apply_handle_delta(
                                drag.handle_ix,
                                desired_delta,
                                &mut sizes,
                                &mins,
                            );
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

                        if has_handle_children {
                            let child_ix = handle_ix.saturating_mul(2).saturating_add(1);
                            if let Some(&child) = cx.children.get(child_ix) {
                                cx.request_focus(child);
                            }
                        }

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
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.model, Invalidation::Layout);

        self.last_bounds = cx.bounds;
        let children_len = cx.children.len();
        let (
            panel_count,
            has_handle_children,
            panel_rects,
            handle_rects,
            handle_centers,
            sizes,
            _avail,
        ) = self.compute_layout(cx.app, cx.bounds, children_len);
        self.last_panel_count = panel_count;
        self.last_has_handle_children = has_handle_children;
        self.last_handle_rects = handle_rects;
        self.last_handle_centers = handle_centers;
        self.last_sizes = sizes;

        if panel_count < 1 {
            return cx.available;
        }

        // Avoid registering viewport roots during "probe" layout passes (important for
        // scroll/virtualized descendants).
        if cx.pass_kind == crate::layout_pass::LayoutPassKind::Final {
            for (panel_ix, &rect) in panel_rects.iter().enumerate() {
                let child_ix = if has_handle_children {
                    panel_ix.saturating_mul(2)
                } else {
                    panel_ix
                };
                let Some(&child) = cx.children.get(child_ix) else {
                    continue;
                };
                let _ = cx.layout_viewport_root(child, rect);
            }

            if has_handle_children {
                for (handle_ix, &rect) in self.last_handle_rects.iter().enumerate() {
                    let child_ix = handle_ix.saturating_mul(2).saturating_add(1);
                    let Some(&child) = cx.children.get(child_ix) else {
                        continue;
                    };
                    let _ = cx.layout_in(child, rect);
                }
            }
            return cx.available;
        }

        for (panel_ix, &rect) in panel_rects.iter().enumerate() {
            let child_ix = if has_handle_children {
                panel_ix.saturating_mul(2)
            } else {
                panel_ix
            };
            let Some(&child) = cx.children.get(child_ix) else {
                continue;
            };
            let _ = cx.layout_in(child, rect);
        }

        if has_handle_children {
            for (handle_ix, &rect) in self.last_handle_rects.iter().enumerate() {
                let child_ix = handle_ix.saturating_mul(2).saturating_add(1);
                let Some(&child) = cx.children.get(child_ix) else {
                    continue;
                };
                let _ = cx.layout_in(child, rect);
            }
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        if self.last_has_handle_children {
            for panel_ix in 0..self.last_panel_count {
                let child_ix = panel_ix.saturating_mul(2);
                let Some(&child) = cx.children.get(child_ix) else {
                    continue;
                };
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint(child, bounds);
                } else {
                    cx.paint(child, cx.bounds);
                }
            }
        } else {
            for &child in cx.children {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint(child, bounds);
                } else {
                    cx.paint(child, cx.bounds);
                }
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

        let focus_visible = crate::focus_visible::is_focus_visible(cx.app, cx.window);
        for (i, center) in self.last_handle_centers.iter().copied().enumerate() {
            if focus_visible
                && self.last_has_handle_children
                && cx.focus.is_some_and(|focus| {
                    let child_ix = i.saturating_mul(2).saturating_add(1);
                    cx.children.get(child_ix).copied() == Some(focus)
                })
            {
                let ring = RingStyle {
                    placement: RingPlacement::Outset,
                    width: Px(1.0),
                    offset: Px(1.0),
                    color: theme
                        .color_by_key("ring")
                        .unwrap_or(theme.colors.focus_ring),
                    offset_color: Some(
                        theme
                            .color_by_key("ring-offset-background")
                            .unwrap_or_else(|| theme.color_token("background")),
                    ),
                    corner_radii: Corners::all(Px(0.0)),
                };
                let paint_rect = handle.paint_rect(cx.bounds, center, cx.scale_factor);
                let paint_thickness = match self.axis {
                    Axis::Horizontal => paint_rect.size.width,
                    Axis::Vertical => paint_rect.size.height,
                };
                let min_thickness = self.style.hit_thickness.min(Px(4.0));
                let thickness = Px(paint_thickness.0.max(min_thickness.0));
                let ring_rect = match self.axis {
                    Axis::Horizontal => Rect {
                        origin: Point::new(Px(center - thickness.0 * 0.5), cx.bounds.origin.y),
                        size: Size::new(thickness, cx.bounds.size.height),
                    },
                    Axis::Vertical => Rect {
                        origin: Point::new(cx.bounds.origin.x, Px(center - thickness.0 * 0.5)),
                        size: Size::new(cx.bounds.size.width, thickness),
                    },
                };
                crate::paint::paint_focus_ring(cx.scene, DrawOrder(9_990), ring_rect, ring);
            }

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

        if self.last_has_handle_children {
            // Ensure handle children (e.g. shadcn `withHandle` grip chrome) paint over the handle
            // line, mirroring upstream `z-10` behavior.
            let handle_count = self.last_panel_count.saturating_sub(1);
            for handle_ix in 0..handle_count {
                let child_ix = handle_ix.saturating_mul(2).saturating_add(1);
                let Some(&child) = cx.children.get(child_ix) else {
                    continue;
                };
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint(child, bounds);
                } else {
                    cx.paint(child, cx.bounds);
                }
            }
        }
    }
}

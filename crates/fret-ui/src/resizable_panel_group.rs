use fret_core::{Axis, Color, DrawOrder, Event, MouseButton, Px, Rect, Size};
use fret_runtime::{Model, ModelId};

use crate::resize_handle::ResizeHandle;
use crate::widget::{EventCx, LayoutCx, PaintCx, SemanticsCx, Widget};
use crate::{Invalidation, Theme, UiHost};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

#[derive(Debug, Clone)]
pub struct ResizablePanelGroupStyle {
    /// Layout gap between panels in logical px.
    ///
    /// This does **not** need to match `hit_thickness`: it is common to keep the visual/layout gap
    /// small (or zero) while using a larger hit area for usability.
    pub gap: Px,
    /// Thickness of the handle region in logical px.
    ///
    /// This region is used for hit-testing (and can be larger than `gap`).
    pub hit_thickness: Px,
    /// Visual thickness in *device* pixels (converted using the current scale factor).
    pub paint_device_px: f32,
    pub handle_color: Color,
    pub handle_alpha: f32,
    pub handle_hover_alpha: f32,
    pub handle_drag_alpha: f32,
}

impl Default for ResizablePanelGroupStyle {
    fn default() -> Self {
        Self {
            gap: Px(0.0),
            hit_thickness: Px(6.0),
            paint_device_px: 1.0,
            handle_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            handle_alpha: 0.18,
            handle_hover_alpha: 0.28,
            handle_drag_alpha: 0.35,
        }
    }
}

impl ResizablePanelGroupStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        let handle_color = theme
            .color_by_key("border")
            .or_else(|| theme.color_by_key("input"))
            .unwrap_or(theme.snapshot().colors.panel_border);

        Self {
            gap: theme
                .metric_by_key("component.resizable.gap")
                .unwrap_or(Px(0.0)),
            hit_thickness: theme
                .metric_by_key("component.resizable.hit_thickness")
                .unwrap_or(Px(6.0)),
            paint_device_px: theme
                .metric_by_key("component.resizable.paint_device_px")
                .map(|p| p.0.max(1.0))
                .unwrap_or(1.0),
            handle_color,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResizablePanelGroupLayout {
    pub panel_rects: Vec<Rect>,
    pub handle_hit_rects: Vec<Rect>,
    pub handle_centers: Vec<f32>,
    pub sizes: Vec<f32>,
    pub mins: Vec<f32>,
    pub avail: f32,
}

fn handle_hit_rect(axis: Axis, bounds: Rect, center: f32, thickness: f32) -> Rect {
    if thickness <= 0.0 || !thickness.is_finite() {
        return Rect::default();
    }

    let axis_origin = match axis {
        Axis::Horizontal => bounds.origin.x.0,
        Axis::Vertical => bounds.origin.y.0,
    };
    let axis_len = match axis {
        Axis::Horizontal => bounds.size.width.0,
        Axis::Vertical => bounds.size.height.0,
    }
    .max(0.0);

    let t = thickness.min(axis_len);
    let max_origin = (axis_origin + axis_len - t).max(axis_origin);
    let origin_axis = (center - t * 0.5).clamp(axis_origin, max_origin);

    match axis {
        Axis::Horizontal => Rect::new(
            fret_core::Point::new(Px(origin_axis), bounds.origin.y),
            Size::new(Px(t), bounds.size.height),
        ),
        Axis::Vertical => Rect::new(
            fret_core::Point::new(bounds.origin.x, Px(origin_axis)),
            Size::new(bounds.size.width, Px(t)),
        ),
    }
}

pub(crate) fn fractions_from_sizes(sizes: &[f32], avail: f32) -> Vec<f32> {
    if avail <= 0.0 {
        return Vec::new();
    }
    let mut next: Vec<f32> = sizes.iter().map(|s| (*s / avail).clamp(0.0, 1.0)).collect();
    next = BoundResizablePanelGroup::sanitize_fractions(next, sizes.len());
    next
}

pub(crate) fn apply_handle_delta(
    handle_ix: usize,
    mut delta: f32,
    sizes: &mut [f32],
    mins: &[f32],
) -> f32 {
    if sizes.len() < 2 || handle_ix + 1 >= sizes.len() {
        return 0.0;
    }
    if mins.len() != sizes.len() {
        return 0.0;
    }

    if delta > 0.0 {
        let mut reducible = 0.0;
        for k in (handle_ix + 1)..sizes.len() {
            reducible += (sizes[k] - mins[k]).max(0.0);
        }
        if reducible <= 1.0e-6 {
            return 0.0;
        }
        delta = delta.min(reducible);
        sizes[handle_ix] += delta;

        let mut remaining = delta;
        for k in (handle_ix + 1)..sizes.len() {
            if remaining <= 1.0e-6 {
                break;
            }
            let available = (sizes[k] - mins[k]).max(0.0);
            let take = remaining.min(available);
            sizes[k] -= take;
            remaining -= take;
        }
        delta - remaining
    } else if delta < 0.0 {
        let shrinkable = (sizes[handle_ix] - mins[handle_ix]).max(0.0);
        if shrinkable <= 1.0e-6 {
            return 0.0;
        }
        delta = delta.max(-shrinkable);
        sizes[handle_ix] += delta;
        sizes[handle_ix + 1] -= delta;
        delta
    } else {
        0.0
    }
}

pub(crate) fn compute_resizable_panel_group_layout(
    axis: Axis,
    bounds: Rect,
    children_len: usize,
    fractions: Vec<f32>,
    gap: Px,
    hit_thickness: Px,
    min_px: &[Px],
) -> ResizablePanelGroupLayout {
    let gap = gap.0.max(0.0);
    let hit = hit_thickness.0.max(0.0).max(gap);

    let axis_len = BoundResizablePanelGroup::axis_len(bounds, axis).max(0.0);
    let total_gap = gap * (children_len.saturating_sub(1) as f32);
    let avail = (axis_len - total_gap).max(0.0);

    let mins = BoundResizablePanelGroup::effective_min_px_static(children_len, avail, min_px);
    let fractions = BoundResizablePanelGroup::sanitize_fractions(fractions, children_len);
    let sizes = BoundResizablePanelGroup::apply_min_constraints(
        BoundResizablePanelGroup::sizes_from_fractions(&fractions, avail),
        &mins,
        avail,
    );

    let mut panel_rects = Vec::with_capacity(children_len);
    let mut handle_hit_rects = Vec::with_capacity(children_len.saturating_sub(1));
    let mut handle_centers = Vec::with_capacity(children_len.saturating_sub(1));

    let mut cursor = BoundResizablePanelGroup::axis_origin(bounds, axis);
    for i in 0..children_len {
        let len = sizes.get(i).copied().unwrap_or(0.0).max(0.0);
        match axis {
            Axis::Horizontal => {
                panel_rects.push(Rect::new(
                    fret_core::Point::new(Px(cursor), bounds.origin.y),
                    Size::new(Px(len), bounds.size.height),
                ));
            }
            Axis::Vertical => {
                panel_rects.push(Rect::new(
                    fret_core::Point::new(bounds.origin.x, Px(cursor)),
                    Size::new(bounds.size.width, Px(len)),
                ));
            }
        }
        cursor += len;

        if i + 1 < children_len {
            let center = cursor + gap * 0.5;
            handle_centers.push(center);
            handle_hit_rects.push(handle_hit_rect(axis, bounds, center, hit));
            cursor += gap;
        }
    }

    ResizablePanelGroupLayout {
        panel_rects,
        handle_hit_rects,
        handle_centers,
        sizes,
        mins,
        avail,
    }
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

    fn axis_pos(pos: fret_core::Point, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => pos.x.0,
            Axis::Vertical => pos.y.0,
        }
    }

    pub(crate) fn effective_min_px_static(count: usize, avail: f32, min_px: &[Px]) -> Vec<f32> {
        let default = Px(120.0);
        if count == 0 {
            return Vec::new();
        }

        let mut mins: Vec<f32> = if min_px.is_empty() {
            vec![default.0; count]
        } else if min_px.len() == 1 {
            vec![min_px[0].0.max(0.0); count]
        } else if min_px.len() == count {
            min_px.iter().map(|p| p.0.max(0.0)).collect()
        } else {
            vec![min_px[0].0.max(0.0); count]
        };

        let sum: f32 = mins.iter().copied().sum();
        if !sum.is_finite() || sum <= 0.0 {
            return mins;
        }
        if avail > 0.0 && avail < sum {
            let scale = (avail / sum).clamp(0.0, 1.0);
            for m in &mut mins {
                *m = (*m * scale).max(0.0);
            }
        }
        mins
    }

    fn effective_min_px(&self, count: usize, avail: f32) -> Vec<f32> {
        Self::effective_min_px_static(count, avail, &self.min_px)
    }

    fn sanitize_fractions(mut v: Vec<f32>, count: usize) -> Vec<f32> {
        if count == 0 {
            return Vec::new();
        }
        if v.len() != count {
            return vec![1.0 / count as f32; count];
        }
        for x in &mut v {
            if !x.is_finite() {
                *x = 0.0;
            }
            *x = (*x).max(0.0);
        }
        let sum: f32 = v.iter().sum();
        if !sum.is_finite() || sum <= f32::EPSILON {
            return vec![1.0 / count as f32; count];
        }
        for x in &mut v {
            *x /= sum;
        }
        v
    }

    fn sizes_from_fractions(fractions: &[f32], avail: f32) -> Vec<f32> {
        let mut sizes: Vec<f32> = fractions
            .iter()
            .copied()
            .map(|f| (f.clamp(0.0, 1.0) * avail).max(0.0))
            .collect();
        let sum: f32 = sizes.iter().sum();
        let diff = avail - sum;
        if sizes.is_empty() {
            return sizes;
        }
        let last = sizes.len() - 1;
        sizes[last] = (sizes[last] + diff).max(0.0);
        sizes
    }

    fn apply_min_constraints(mut sizes: Vec<f32>, mins: &[f32], avail: f32) -> Vec<f32> {
        if sizes.is_empty() {
            return sizes;
        }
        if mins.len() != sizes.len() {
            return sizes;
        }

        let sum_min: f32 = mins.iter().copied().sum();
        if avail <= 0.0 {
            return vec![0.0; sizes.len()];
        }
        if sum_min.is_finite() && sum_min > 0.0 && avail < sum_min {
            let scale = (avail / sum_min).clamp(0.0, 1.0);
            for (s, m) in sizes.iter_mut().zip(mins.iter().copied()) {
                *s = (m * scale).max(0.0);
            }
            return sizes;
        }

        for (s, m) in sizes.iter_mut().zip(mins.iter().copied()) {
            if *s < m {
                *s = m;
            }
        }

        let mut sum: f32 = sizes.iter().sum();
        if sum <= avail + 1.0e-3 {
            let last = sizes.len() - 1;
            sizes[last] = (sizes[last] + (avail - sum)).max(mins[last]);
            return sizes;
        }

        let mut excess = sum - avail;
        for _ in 0..4 {
            if excess <= 1.0e-3 {
                break;
            }
            let mut adjustable_total = 0.0;
            for (s, m) in sizes.iter().zip(mins.iter().copied()) {
                adjustable_total += (*s - m).max(0.0);
            }
            if adjustable_total <= 1.0e-6 {
                break;
            }
            for (s, m) in sizes.iter_mut().zip(mins.iter().copied()) {
                let room = (*s - m).max(0.0);
                if room <= 0.0 {
                    continue;
                }
                let take = (excess * (room / adjustable_total)).min(room);
                *s -= take;
                excess -= take;
                if excess <= 1.0e-3 {
                    break;
                }
            }
        }

        sum = sizes.iter().sum();
        let last = sizes.len() - 1;
        sizes[last] = (sizes[last] + (avail - sum)).max(mins[last]);
        sizes
    }

    fn compute_layout<H: UiHost>(
        &mut self,
        app: &H,
        bounds: Rect,
        children_len: usize,
    ) -> (Vec<Rect>, Vec<Rect>, Vec<f32>, Vec<f32>, f32) {
        let raw = app.models().get(self.model).cloned().unwrap_or_default();
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
        let _ = app.models_mut().update(self.model, |v| *v = next);
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
                    let desired_center = Self::axis_pos(*position, self.axis) - drag.grab_offset;
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
                    grab_offset: Self::axis_pos(*position, self.axis) - center,
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
        cx.observe_model(self.model, Invalidation::Layout);

        self.last_bounds = cx.bounds;
        let children_len = cx.children.len();
        let (panel_rects, handle_rects, handle_centers, sizes, _avail) =
            self.compute_layout(cx.app, cx.bounds, children_len);
        self.last_handle_rects = handle_rects;
        self.last_handle_centers = handle_centers;
        self.last_sizes = sizes;

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

        let theme = Theme::global(&*cx.app).clone();
        if self.style.handle_color.a <= 0.0 {
            self.style.handle_color = theme
                .color_by_key("border")
                .or_else(|| theme.color_by_key("input"))
                .unwrap_or(theme.snapshot().colors.panel_border);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use crate::tree::UiTree;
    use fret_core::{
        AppWindowId, Event, PathCommand, PathConstraints, PathMetrics, PathService, PathStyle,
        PlatformCapabilities, Point, Px, Size, TextConstraints, TextMetrics, TextService,
        TextStyle,
    };
    use fret_runtime::Effect;

    #[derive(Default)]
    struct FakeUiServices;

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

    impl PathService for FakeUiServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (fret_core::PathId, PathMetrics) {
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

    struct Dummy;
    impl<H: UiHost> Widget<H> for Dummy {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(0.0), Px(0.0))
        }
    }

    #[test]
    fn resizable_panel_group_drag_updates_fractions_model() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let fractions = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
        let mut group = BoundResizablePanelGroup::new(Axis::Horizontal, fractions);
        group.set_style(ResizablePanelGroupStyle {
            gap: Px(0.0),
            hit_thickness: Px(10.0),
            ..Default::default()
        });

        let root_id = ui.create_node(group);
        let a = ui.create_node(Dummy);
        let b = ui.create_node(Dummy);
        let c = ui.create_node(Dummy);
        ui.add_child(root_id, a);
        ui.add_child(root_id, b);
        ui.add_child(root_id, c);
        ui.set_root(root_id);

        let mut services = FakeUiServices;
        let size = Size::new(Px(600.0), Px(40.0));
        let _ = ui.layout(&mut app, &mut services, root_id, size, 1.0);
        let _ = app.take_effects();

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
        let fractions_now = app.models().get(fractions).cloned().unwrap_or_default();
        let layout = compute_resizable_panel_group_layout(
            Axis::Horizontal,
            bounds,
            3,
            fractions_now,
            Px(0.0),
            Px(10.0),
            &[],
        );
        let center = layout.handle_centers.first().copied().unwrap_or(0.0);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(center), Px(20.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(ui.captured(), Some(root_id), "expected pointer capture");
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(center + 30.0), Px(20.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(center + 30.0), Px(20.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let v = app.models().get(fractions).cloned().unwrap_or_default();
        assert_eq!(v.len(), 3);
        assert!(v[0] > 0.33, "expected left panel to grow, got {v:?}");
        let effects = app.take_effects();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::CursorSetIcon { window: w, icon }
                    if *w == window && *icon == fret_core::CursorIcon::ColResize
            )),
            "expected resize cursor effects during interaction"
        );
    }

    #[test]
    fn resizable_panel_group_pushes_growth_through_following_panels() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let fractions = app.models_mut().insert(vec![0.33, 0.34, 0.33]);
        let mut group = BoundResizablePanelGroup::new(Axis::Horizontal, fractions);
        group.set_style(ResizablePanelGroupStyle {
            gap: Px(0.0),
            hit_thickness: Px(10.0),
            ..Default::default()
        });
        group.set_min_px(vec![Px(100.0), Px(100.0), Px(100.0)]);

        let root_id = ui.create_node(group);
        let a = ui.create_node(Dummy);
        let b = ui.create_node(Dummy);
        let c = ui.create_node(Dummy);
        ui.add_child(root_id, a);
        ui.add_child(root_id, b);
        ui.add_child(root_id, c);
        ui.set_root(root_id);

        let mut services = FakeUiServices;
        let size = Size::new(Px(600.0), Px(40.0));
        let _ = ui.layout(&mut app, &mut services, root_id, size, 1.0);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
        let before = app.models().get(fractions).cloned().unwrap_or_default();
        let layout_before = compute_resizable_panel_group_layout(
            Axis::Horizontal,
            bounds,
            3,
            before,
            Px(0.0),
            Px(10.0),
            &[Px(100.0), Px(100.0), Px(100.0)],
        );
        let center = layout_before.handle_centers.first().copied().unwrap_or(0.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(center), Px(20.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(ui.captured(), Some(root_id), "expected pointer capture");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(center + 250.0), Px(20.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(center + 250.0), Px(20.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let after = app.models().get(fractions).cloned().unwrap_or_default();
        let layout_after = compute_resizable_panel_group_layout(
            Axis::Horizontal,
            bounds,
            3,
            after,
            Px(0.0),
            Px(10.0),
            &[Px(100.0), Px(100.0), Px(100.0)],
        );

        assert_eq!(layout_after.sizes.len(), 3);
        assert!(
            (layout_after.sizes[0] - 400.0).abs() < 0.01,
            "{layout_after:?}"
        );
        assert!(
            (layout_after.sizes[1] - 100.0).abs() < 0.01,
            "{layout_after:?}"
        );
        assert!(
            (layout_after.sizes[2] - 100.0).abs() < 0.01,
            "{layout_after:?}"
        );
    }

    #[test]
    fn resizable_panel_group_shrink_clamps_to_min_px() {
        let window = AppWindowId::default();

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let fractions = app.models_mut().insert(vec![0.5, 0.25, 0.25]);
        let mut group = BoundResizablePanelGroup::new(Axis::Horizontal, fractions);
        group.set_style(ResizablePanelGroupStyle {
            gap: Px(0.0),
            hit_thickness: Px(10.0),
            ..Default::default()
        });
        group.set_min_px(vec![Px(100.0), Px(100.0), Px(100.0)]);

        let root_id = ui.create_node(group);
        let a = ui.create_node(Dummy);
        let b = ui.create_node(Dummy);
        let c = ui.create_node(Dummy);
        ui.add_child(root_id, a);
        ui.add_child(root_id, b);
        ui.add_child(root_id, c);
        ui.set_root(root_id);

        let mut services = FakeUiServices;
        let size = Size::new(Px(600.0), Px(40.0));
        let _ = ui.layout(&mut app, &mut services, root_id, size, 1.0);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), size);
        let before = app.models().get(fractions).cloned().unwrap_or_default();
        let layout_before = compute_resizable_panel_group_layout(
            Axis::Horizontal,
            bounds,
            3,
            before,
            Px(0.0),
            Px(10.0),
            &[Px(100.0), Px(100.0), Px(100.0)],
        );
        let center = layout_before.handle_centers.first().copied().unwrap_or(0.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(center), Px(20.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(ui.captured(), Some(root_id), "expected pointer capture");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Move {
                position: Point::new(Px(center - 250.0), Px(20.0)),
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(center - 250.0), Px(20.0)),
                button: MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );

        let after = app.models().get(fractions).cloned().unwrap_or_default();
        let layout_after = compute_resizable_panel_group_layout(
            Axis::Horizontal,
            bounds,
            3,
            after,
            Px(0.0),
            Px(10.0),
            &[Px(100.0), Px(100.0), Px(100.0)],
        );

        assert_eq!(layout_after.sizes.len(), 3);
        assert!(
            (layout_after.sizes[0] - 100.0).abs() < 0.01,
            "{layout_after:?}"
        );
        assert!(
            (layout_after.sizes[1] - 350.0).abs() < 0.01,
            "{layout_after:?}"
        );
        assert!(
            (layout_after.sizes[2] - 150.0).abs() < 0.01,
            "{layout_after:?}"
        );
    }
}

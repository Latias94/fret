use crate::{AppWindowId, PointerId, RenderTargetId, ViewportFit, ViewportMapping};

use super::{Modifiers, MouseButton, MouseButtons, PointerCancelReason, PointerType};
use crate::geometry::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportInputGeometry {
    /// The viewport widget bounds in window-local logical pixels (ADR 0017).
    pub content_rect_px: Rect,
    /// The mapped draw rect in window-local logical pixels after applying the viewport `fit`.
    pub draw_rect_px: Rect,
    /// The backing render target size in physical pixels.
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    /// Pixels-per-point (a.k.a. window scale factor) used to convert logical px → physical px.
    pub pixels_per_point: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportInputEvent {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub pointer_id: PointerId,
    pub pointer_type: PointerType,
    pub geometry: ViewportInputGeometry,
    /// Cursor position in window-local logical pixels (ADR 0017).
    pub cursor_px: Point,
    pub uv: (f32, f32),
    pub target_px: (u32, u32),
    pub kind: ViewportInputKind,
}

impl ViewportInputEvent {
    /// Returns the scale from window-local logical pixels ("screen px") to render-target pixels.
    ///
    /// This is derived from `self.geometry.draw_rect_px` (logical pixels) and the backing render
    /// target size `self.geometry.target_px_size` (physical pixels).
    ///
    /// For `ViewportFit::Contain`/`Cover` this is uniform; for `ViewportFit::Stretch` the mapping
    /// is non-uniform, so this returns the smaller axis scale as a conservative approximation for
    /// isotropic thresholds (hit radii, click distances).
    pub fn target_px_per_screen_px(&self) -> Option<f32> {
        let (tw, th) = self.geometry.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let rect = self.geometry.draw_rect_px;
        let dw = rect.size.width.0.max(0.0);
        let dh = rect.size.height.0.max(0.0);
        if dw <= 0.0 || dh <= 0.0 || !dw.is_finite() || !dh.is_finite() {
            return None;
        }

        let sx = tw / dw;
        let sy = th / dh;
        let s = sx.min(sy);
        (s.is_finite() && s > 0.0).then_some(s)
    }

    /// Computes the cursor position in the viewport render target's pixel space (float).
    ///
    /// - Input `self.cursor_px` is in window-local logical pixels (ADR 0017).
    /// - The mapping uses `self.geometry.draw_rect_px` (logical pixels) as the area that maps to
    ///   the full render target.
    /// - Output is expressed in physical target pixels (`self.geometry.target_px_size`).
    ///
    /// This is useful for editor tooling that operates directly on render-target pixel buffers.
    /// Prefer this over reconstructing target coordinates from `uv * target_px_size` because `uv`
    /// and `target_px` may be clamped when pointer capture is active.
    pub fn cursor_target_px_f32(&self) -> Option<(f32, f32)> {
        let (tw, th) = self.geometry.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let rect = self.geometry.draw_rect_px;
        let dw = rect.size.width.0.max(0.0);
        let dh = rect.size.height.0.max(0.0);
        if dw <= 0.0 || dh <= 0.0 || !dw.is_finite() || !dh.is_finite() {
            return None;
        }

        let uv_x = (self.cursor_px.x.0 - rect.origin.x.0) / dw;
        let uv_y = (self.cursor_px.y.0 - rect.origin.y.0) / dh;
        Some((uv_x * tw, uv_y * th))
    }

    /// Like [`Self::cursor_target_px_f32`], but clamps the resulting coordinates to the render
    /// target bounds.
    pub fn cursor_target_px_f32_clamped(&self) -> (f32, f32) {
        let (tw, th) = self.geometry.target_px_size;
        let tw = tw.max(1) as f32;
        let th = th.max(1) as f32;

        let Some((x, y)) = self.cursor_target_px_f32() else {
            return (self.target_px.0 as f32, self.target_px.1 as f32);
        };
        (x.clamp(0.0, tw), y.clamp(0.0, th))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_mapping_window_point(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        pixels_per_point: f32,
        pointer_id: PointerId,
        pointer_type: PointerType,
        position: Point,
        kind: ViewportInputKind,
    ) -> Option<Self> {
        let mapped = mapping.map();
        let uv = mapping.window_point_to_uv(position)?;
        let target_px = mapping.window_point_to_target_px(position)?;
        Some(Self {
            window,
            target,
            pointer_id,
            pointer_type,
            geometry: ViewportInputGeometry {
                content_rect_px: mapping.content_rect,
                draw_rect_px: mapped.draw_rect,
                target_px_size: mapping.target_px_size,
                fit: mapping.fit,
                pixels_per_point,
            },
            cursor_px: position,
            uv,
            target_px,
            kind,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_mapping_window_point_clamped(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        pixels_per_point: f32,
        pointer_id: PointerId,
        pointer_type: PointerType,
        position: Point,
        kind: ViewportInputKind,
    ) -> Self {
        let mapped = mapping.map();
        let uv = mapping.window_point_to_uv_clamped(position);
        let target_px = mapping.window_point_to_target_px_clamped(position);
        Self {
            window,
            target,
            pointer_id,
            pointer_type,
            geometry: ViewportInputGeometry {
                content_rect_px: mapping.content_rect,
                draw_rect_px: mapped.draw_rect,
                target_px_size: mapping.target_px_size,
                fit: mapping.fit,
                pixels_per_point,
            },
            cursor_px: position,
            uv,
            target_px,
            kind,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_mapping_window_point_maybe_clamped(
        window: AppWindowId,
        target: RenderTargetId,
        mapping: &ViewportMapping,
        pixels_per_point: f32,
        pointer_id: PointerId,
        pointer_type: PointerType,
        position: Point,
        kind: ViewportInputKind,
        clamped: bool,
    ) -> Option<Self> {
        if clamped {
            Some(Self::from_mapping_window_point_clamped(
                window,
                target,
                mapping,
                pixels_per_point,
                pointer_id,
                pointer_type,
                position,
                kind,
            ))
        } else {
            Self::from_mapping_window_point(
                window,
                target,
                mapping,
                pixels_per_point,
                pointer_id,
                pointer_type,
                position,
                kind,
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportInputKind {
    PointerMove {
        buttons: MouseButtons,
        modifiers: Modifiers,
    },
    PointerDown {
        button: MouseButton,
        modifiers: Modifiers,
        /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
        click_count: u8,
    },
    PointerUp {
        button: MouseButton,
        modifiers: Modifiers,
        /// Whether this pointer-up completes a "true click".
        ///
        /// See `PointerEvent::Up.is_click` for normalization rules.
        is_click: bool,
        /// See `PointerEvent::{Down,Up}.click_count` for normalization rules.
        click_count: u8,
    },
    PointerCancel {
        buttons: MouseButtons,
        modifiers: Modifiers,
        reason: PointerCancelReason,
    },
    Wheel {
        delta: Point,
        modifiers: Modifiers,
    },
}

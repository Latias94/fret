use super::{
    WindowLogicalSize, WindowPhysicalPosition, WindowPosition, WinitAppDriver, WinitRunner,
};
use fret_core::{Point, Px};
use winit::dpi::PhysicalPosition;

#[derive(Debug, Clone, Copy)]
pub(super) struct WindowClientOriginDiagnostics {
    pub(super) client_origin_screen: PhysicalPosition<f64>,
    pub(super) client_origin_source_platform: bool,
    pub(super) outer_pos_physical: Option<PhysicalPosition<i32>>,
    pub(super) decoration_offset_physical: PhysicalPosition<i32>,
    pub(super) scale_factor: f64,
}

pub(super) fn client_origin_screen(
    outer: winit::dpi::PhysicalPosition<i32>,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
) -> winit::dpi::PhysicalPosition<f64> {
    winit::dpi::PhysicalPosition::new(
        outer.x as f64 + decoration_offset.x as f64,
        outer.y as f64 + decoration_offset.y as f64,
    )
}

pub(super) fn screen_pos_in_client(
    client_origin: winit::dpi::PhysicalPosition<f64>,
    client_size: winit::dpi::PhysicalSize<u32>,
    screen_pos: winit::dpi::PhysicalPosition<f64>,
) -> bool {
    let left = client_origin.x;
    let top = client_origin.y;
    let right = left + client_size.width as f64;
    let bottom = top + client_size.height as f64;
    screen_pos.x >= left && screen_pos.x < right && screen_pos.y >= top && screen_pos.y < bottom
}

pub(super) fn local_pos_for_screen_pos(
    client_origin: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
    screen_pos: winit::dpi::PhysicalPosition<f64>,
) -> Point {
    let local_physical = winit::dpi::PhysicalPosition::new(
        screen_pos.x - client_origin.x,
        screen_pos.y - client_origin.y,
    );
    let local_logical: winit::dpi::LogicalPosition<f32> = local_physical.to_logical(scale_factor);
    Point::new(Px(local_logical.x), Px(local_logical.y))
}

pub(super) fn outer_pos_for_cursor_grab(
    screen_pos: PhysicalPosition<f64>,
    grab_offset_logical: Point,
    scale_factor: f64,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
    max_client_logical: Option<winit::dpi::LogicalSize<f32>>,
) -> Option<(f64, f64)> {
    if !grab_offset_logical.x.0.is_finite()
        || !grab_offset_logical.y.0.is_finite()
        || grab_offset_logical.x.0 < 0.0
        || grab_offset_logical.y.0 < 0.0
    {
        return None;
    }

    let mut grab_x = grab_offset_logical.x.0;
    let mut grab_y = grab_offset_logical.y.0;
    if let Some(max) = max_client_logical {
        if max.width.is_finite() && max.width > 0.0 {
            grab_x = grab_x.min(max.width).max(0.0);
        } else {
            grab_x = 0.0;
        }
        if max.height.is_finite() && max.height > 0.0 {
            grab_y = grab_y.min(max.height).max(0.0);
        } else {
            grab_y = 0.0;
        }
    }

    // Match ImGui's platform contract:
    // - viewport pos is client/inner screen position (logical)
    // - winit expects outer position
    // - therefore: outer = desired_client - decoration_offset(window)
    // See `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs:winit_set_window_pos`.
    let grab_client_x = grab_x as f64 * scale_factor;
    let grab_client_y = grab_y as f64 * scale_factor;
    let grab_outer_x = decoration_offset.x as f64 + grab_client_x;
    let grab_outer_y = decoration_offset.y as f64 + grab_client_y;

    let x = screen_pos.x - grab_outer_x;
    let y = screen_pos.y - grab_outer_y;
    Some((x, y))
}

pub(super) fn scale_decoration_offset_for_target_scale(
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
    source_scale_factor: f64,
    target_scale_factor: f64,
) -> winit::dpi::PhysicalPosition<i32> {
    if !source_scale_factor.is_finite()
        || source_scale_factor <= 0.0
        || !target_scale_factor.is_finite()
        || target_scale_factor <= 0.0
    {
        return decoration_offset;
    }

    let ratio = target_scale_factor / source_scale_factor;
    if !ratio.is_finite() || ratio <= 0.0 || (ratio - 1.0).abs() <= f64::EPSILON {
        return decoration_offset;
    }

    let scaled_x = (decoration_offset.x as f64 * ratio)
        .round()
        .clamp(i32::MIN as f64, i32::MAX as f64) as i32;
    let scaled_y = (decoration_offset.y as f64 * ratio)
        .round()
        .clamp(i32::MIN as f64, i32::MAX as f64) as i32;

    winit::dpi::PhysicalPosition::new(scaled_x, scaled_y)
}

pub(super) fn estimated_outer_pos_for_cursor_grab(
    screen_pos: PhysicalPosition<f64>,
    grab_offset_logical: Point,
    source_scale_factor: f64,
    target_scale_factor: f64,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
    max_client_logical: Option<winit::dpi::LogicalSize<f32>>,
) -> Option<(f64, f64)> {
    let scale_factor = if target_scale_factor.is_finite() && target_scale_factor > 0.0 {
        target_scale_factor
    } else {
        source_scale_factor
    };
    let decoration_offset = scale_decoration_offset_for_target_scale(
        decoration_offset,
        source_scale_factor,
        scale_factor,
    );
    outer_pos_for_cursor_grab(
        screen_pos,
        grab_offset_logical,
        scale_factor,
        decoration_offset,
        max_client_logical,
    )
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn compute_window_position_from_anchor(
        &self,
        anchor: fret_core::WindowAnchor,
    ) -> Option<WindowPosition> {
        let anchor_state = self.windows.get(anchor.window)?;
        // `WindowAnchor::position` is in surface-local logical coordinates (matching pointer
        // events), so start from the surface origin in desktop coordinates.
        let outer = anchor_state.window.outer_position().ok()?;
        let surface = anchor_state.window.surface_position();
        let scale = anchor_state.window.scale_factor();

        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = outer.x as f64 + surface.x as f64 + anchor.position.x.0 as f64 * scale + ox;
        let mut y = outer.y as f64 + surface.y as f64 + anchor.position.y.0 as f64 * scale + oy;

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        if let Some(monitor) = anchor_state.window.current_monitor()
            && let (Some(pos), Some(mode)) = (monitor.position(), monitor.current_video_mode())
        {
            let size = mode.size();
            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            // Leave a small margin so the window stays reachable even if its size is larger
            // than the monitor work area.
            let max_x = min_x + size.width as f64 - 40.0;
            let max_y = min_y + size.height as f64 - 40.0;

            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn compute_window_position_from_cursor(
        &self,
        reference_window: fret_core::AppWindowId,
    ) -> Option<WindowPosition> {
        let screen_pos = self.cursor_screen_pos?;
        let ref_state = self.windows.get(reference_window)?;
        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = screen_pos.x + ox;
        let mut y = screen_pos.y + oy;

        if let Some(monitor) = ref_state.window.current_monitor()
            && let (Some(pos), Some(mode)) = (monitor.position(), monitor.current_video_mode())
        {
            let size = mode.size();
            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            let max_x = min_x + size.width as f64 - 40.0;
            let max_y = min_y + size.height as f64 - 40.0;

            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn compute_window_position_from_cursor_grab_estimate(
        &self,
        reference_window: fret_core::AppWindowId,
        new_window_inner_size: WindowLogicalSize,
        grab_offset_logical: Point,
    ) -> Option<WindowPosition> {
        let screen_pos = self.cursor_screen_pos?;
        let state = self.windows.get(reference_window)?;
        let source_scale = state.window.scale_factor();

        #[cfg(target_os = "windows")]
        let target_scale = Self::monitor_scale_factor_for_point(state.window.as_ref(), screen_pos)
            .unwrap_or(source_scale);
        #[cfg(not(target_os = "windows"))]
        let target_scale = source_scale;

        let max_client = winit::dpi::LogicalSize::new(
            new_window_inner_size.width as f32,
            new_window_inner_size.height as f32,
        );

        let mut x = screen_pos.x;
        let mut y = screen_pos.y;

        #[cfg(target_os = "windows")]
        let decoration_offset = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::decoration_offset_for_hwnd)
            .unwrap_or_else(|| state.window.surface_position());
        #[cfg(not(target_os = "windows"))]
        let decoration_offset = state.window.surface_position();

        if let Some((ox, oy)) = estimated_outer_pos_for_cursor_grab(
            screen_pos,
            grab_offset_logical,
            source_scale,
            target_scale,
            decoration_offset,
            Some(max_client),
        ) {
            x = ox;
            y = oy;
        }

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        let outer_size =
            winit::dpi::LogicalSize::new(new_window_inner_size.width, new_window_inner_size.height)
                .to_physical::<u32>(target_scale);

        #[cfg(target_os = "windows")]
        if let Some(work) = super::win32::monitor_work_area_for_point(screen_pos) {
            (x, y) = Self::clamp_window_outer_pos_to_monitor(
                x,
                y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn compute_window_outer_position_from_cursor_grab(
        &self,
        target_window: fret_core::AppWindowId,
        grab_offset_logical: Point,
    ) -> Option<WindowPosition> {
        let screen_pos = self.cursor_screen_pos?;
        let state = self.windows.get(target_window)?;
        let scale = state.window.scale_factor();

        // Clamp the grab point to the target window's current client size. During tear-off, the
        // grab offset comes from the source window's client coordinates; if the new floating
        // window is smaller, keeping the original offset would place the cursor outside the new
        // window (visible as a fixed offset between cursor and window).
        let target_inner = state.window.surface_size();
        let target_inner_logical: winit::dpi::LogicalSize<f32> = target_inner.to_logical(scale);

        #[cfg(target_os = "windows")]
        let decoration_offset = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::decoration_offset_for_hwnd)
            .unwrap_or_else(|| state.window.surface_position());
        #[cfg(not(target_os = "windows"))]
        let decoration_offset = state.window.surface_position();

        let (mut x, mut y) = outer_pos_for_cursor_grab(
            screen_pos,
            grab_offset_logical,
            scale,
            decoration_offset,
            Some(target_inner_logical),
        )?;

        // Align with ImGui docking/multi-viewport behavior:
        // - platform backend sets the window pos as requested
        // - visibility/reachability constraints are based on the *target monitor*, not the window's
        //   current monitor (which can pin the window at monitor edges).
        let outer_size = state.window.outer_size();

        #[cfg(target_os = "windows")]
        if let Some(work) = super::win32::monitor_work_area_for_point(screen_pos) {
            (x, y) = Self::clamp_window_outer_pos_to_monitor(
                x,
                y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
        } else {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            } else if let Some(monitor) = Self::virtual_desktop_bounds(state.window.as_ref()) {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            } else if let Some(monitor) = Self::virtual_desktop_bounds(state.window.as_ref()) {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn cursor_screen_pos_fallback_for_window(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<PhysicalPosition<f64>> {
        let state = self.windows.get(window)?;
        // `Window::surface_position()` is defined as the decoration offset from the outer
        // window position to the client/surface origin (ImGui-style multi-viewport contract).
        // Convert it to a screen-space client origin before adding a local cursor position.
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            })?;
        #[cfg(not(target_os = "windows"))]
        let origin = {
            let outer = state.window.outer_position().ok()?;
            let deco = state.window.surface_position();
            client_origin_screen(outer, deco)
        };
        let scale = state.window.scale_factor();
        let x = origin.x + state.platform.input.cursor_pos.x.0 as f64 * scale;
        let y = origin.y + state.platform.input.cursor_pos.y.0 as f64 * scale;
        Some(PhysicalPosition::new(x, y))
    }

    pub(super) fn screen_pos_in_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> bool {
        let Some(state) = self.windows.get(window) else {
            return false;
        };
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            });
        #[cfg(not(target_os = "windows"))]
        let origin = state
            .window
            .outer_position()
            .ok()
            .map(|outer| client_origin_screen(outer, state.window.surface_position()));
        let size = state.window.surface_size();
        origin.is_some_and(|origin| screen_pos_in_client(origin, size, screen_pos))
    }

    pub(super) fn local_pos_for_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> Option<Point> {
        let diag = self.client_origin_screen_diagnostics_for_window(window)?;
        Some(local_pos_for_screen_pos(
            diag.client_origin_screen,
            diag.scale_factor,
            screen_pos,
        ))
    }

    pub(super) fn client_origin_screen_diagnostics_for_window(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<WindowClientOriginDiagnostics> {
        let state = self.windows.get(window)?;

        let outer = state.window.outer_position().ok();
        #[cfg(target_os = "windows")]
        let decoration_offset = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::decoration_offset_for_hwnd)
            .unwrap_or_else(|| state.window.surface_position());
        #[cfg(not(target_os = "windows"))]
        let decoration_offset = state.window.surface_position();

        let fallback = outer.map(|outer| client_origin_screen(outer, decoration_offset));
        #[cfg(target_os = "windows")]
        let platform = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd);
        #[cfg(not(target_os = "windows"))]
        let platform: Option<PhysicalPosition<f64>> = None;

        let client_origin_screen = platform.or(fallback)?;
        let client_origin_source_platform = platform.is_some();

        Some(WindowClientOriginDiagnostics {
            client_origin_screen,
            client_origin_source_platform,
            outer_pos_physical: outer,
            decoration_offset_physical: decoration_offset,
            scale_factor: state.window.scale_factor(),
        })
    }

    pub(super) fn window_client_rect_screen(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<(
        winit::dpi::PhysicalPosition<f64>,
        winit::dpi::PhysicalSize<u32>,
    )> {
        let state = self.windows.get(window)?;
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            })?;
        #[cfg(not(target_os = "windows"))]
        let origin = {
            let outer = state.window.outer_position().ok()?;
            let deco = state.window.surface_position();
            client_origin_screen(outer, deco)
        };
        let size = state.window.surface_size();
        Some((origin, size))
    }

    pub(super) fn clamp_screen_pos_to_window_client(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> Option<PhysicalPosition<f64>> {
        let (origin, size) = self.window_client_rect_screen(window)?;
        if size.width == 0 || size.height == 0 {
            return None;
        }
        // Clamp to the inclusive interior to avoid points right on the boundary (which can be
        // sensitive to rounding and platform hit-test behavior).
        let min_x = origin.x + 1.0;
        let min_y = origin.y + 1.0;
        let max_x = origin.x + (size.width as f64) - 1.0;
        let max_y = origin.y + (size.height as f64) - 1.0;
        Some(PhysicalPosition::new(
            screen_pos.x.clamp(min_x, max_x),
            screen_pos.y.clamp(min_y, max_y),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    fn outer_pos_for_cursor_grab_accounts_for_decorations_and_scale() {
        let cursor = PhysicalPosition::new(1000.0, 500.0);
        let grab = Point::new(Px(20.0), Px(40.0));
        let scale = 1.5;
        let deco = winit::dpi::PhysicalPosition::new(10, 30);
        let max_client = winit::dpi::LogicalSize::new(200.0f32, 200.0f32);

        let (x, y) = outer_pos_for_cursor_grab(cursor, grab, scale, deco, Some(max_client))
            .expect("expected outer pos");
        assert_eq!(x, 960.0);
        assert_eq!(y, 410.0);
    }

    #[test]
    fn outer_pos_for_cursor_grab_clamps_to_client_size() {
        let cursor = PhysicalPosition::new(1000.0, 500.0);
        let grab = Point::new(Px(9999.0), Px(9999.0));
        let scale = 2.0;
        let deco = winit::dpi::PhysicalPosition::new(0, 0);
        let max_client = winit::dpi::LogicalSize::new(100.0f32, 100.0f32);

        let (x, y) = outer_pos_for_cursor_grab(cursor, grab, scale, deco, Some(max_client))
            .expect("expected outer pos");
        assert_eq!(x, 800.0);
        assert_eq!(y, 300.0);
    }

    #[test]
    fn scale_decoration_offset_for_target_scale_applies_monitor_ratio() {
        let deco = winit::dpi::PhysicalPosition::new(12, 36);
        let scaled = scale_decoration_offset_for_target_scale(deco, 1.0, 1.5);
        assert_eq!(scaled, winit::dpi::PhysicalPosition::new(18, 54));
    }

    #[test]
    fn estimated_outer_pos_for_cursor_grab_prefers_target_monitor_scale() {
        let cursor = PhysicalPosition::new(1500.0, 900.0);
        let grab = Point::new(Px(20.0), Px(40.0));
        let source_scale = 1.0;
        let target_scale = 1.5;
        let deco = winit::dpi::PhysicalPosition::new(10, 30);
        let max_client = winit::dpi::LogicalSize::new(200.0f32, 200.0f32);

        let (x, y) = estimated_outer_pos_for_cursor_grab(
            cursor,
            grab,
            source_scale,
            target_scale,
            deco,
            Some(max_client),
        )
        .expect("expected outer pos");
        assert_eq!(x, 1455.0);
        assert_eq!(y, 795.0);
    }

    #[test]
    fn client_origin_screen_adds_decoration_offset() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(12, 34);
        let origin = client_origin_screen(outer, deco);
        assert_eq!(origin, PhysicalPosition::new(112.0, 234.0));
    }

    #[test]
    fn screen_pos_in_client_uses_half_open_bounds() {
        let origin = PhysicalPosition::new(10.0, 20.0);
        let size = PhysicalSize::new(100u32, 50u32);

        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(10.0, 20.0)
        ));
        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(109.9, 69.9)
        ));

        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(110.0, 20.0)
        ));
        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(10.0, 70.0)
        ));
    }

    #[test]
    fn local_pos_for_screen_pos_respects_scale_factor() {
        let origin = PhysicalPosition::new(100.0, 200.0);
        let scale = 2.0;
        let screen_pos = PhysicalPosition::new(120.0, 240.0);
        let local = local_pos_for_screen_pos(origin, scale, screen_pos);
        assert_eq!(local, Point::new(Px(10.0), Px(20.0)));
    }

    #[test]
    fn screen_pos_in_client_respects_outer_plus_decoration_offset() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(12, 34);
        let origin = client_origin_screen(outer, deco);
        let size = PhysicalSize::new(100u32, 50u32);

        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(112.0, 234.0)
        ));
        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(111.9, 234.0)
        ));
    }

    #[test]
    fn local_pos_for_screen_pos_roundtrips_with_outer_plus_decoration_and_scale() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(10, 30);
        let origin = client_origin_screen(outer, deco);
        let scale = 1.5;

        let desired_local = Point::new(Px(20.0), Px(40.0));
        let screen_pos = PhysicalPosition::new(
            origin.x + desired_local.x.0 as f64 * scale,
            origin.y + desired_local.y.0 as f64 * scale,
        );

        let local = local_pos_for_screen_pos(origin, scale, screen_pos);
        assert_eq!(local, desired_local);
    }
}

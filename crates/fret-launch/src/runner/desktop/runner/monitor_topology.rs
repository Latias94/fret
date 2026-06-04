use winit::dpi::PhysicalPosition;
use winit::event_loop::ActiveEventLoop;

use super::{Window, WinitAppDriver, WinitRunner};

#[derive(Clone, Copy, Debug)]
pub(super) struct MonitorRectF64 {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RectF64 {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

fn collect_runner_monitor_topology_snapshot(
    event_loop: &dyn ActiveEventLoop,
) -> fret_runtime::RunnerMonitorTopologySnapshotV1 {
    let mut monitors = event_loop
        .available_monitors()
        .filter_map(|monitor| {
            let pos = monitor.position()?;
            let size = monitor.current_video_mode()?.size();
            let scale_factor = monitor.scale_factor() as f32;
            let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
                scale_factor
            } else {
                1.0
            };
            Some(fret_runtime::RunnerMonitorInfoV1 {
                bounds_physical: fret_runtime::RunnerMonitorRectPhysicalV1 {
                    x: pos.x,
                    y: pos.y,
                    width: size.width,
                    height: size.height,
                },
                scale_factor,
            })
        })
        .collect::<Vec<_>>();

    monitors.sort_by(|a, b| {
        a.bounds_physical
            .x
            .cmp(&b.bounds_physical.x)
            .then_with(|| a.bounds_physical.y.cmp(&b.bounds_physical.y))
            .then_with(|| a.bounds_physical.width.cmp(&b.bounds_physical.width))
            .then_with(|| a.bounds_physical.height.cmp(&b.bounds_physical.height))
            .then_with(|| a.scale_factor.to_bits().cmp(&b.scale_factor.to_bits()))
    });

    let virtual_desktop_bounds_physical = (!monitors.is_empty()).then(|| {
        let min_x = monitors
            .iter()
            .map(|monitor| monitor.bounds_physical.x as i64)
            .min()
            .unwrap_or(0);
        let min_y = monitors
            .iter()
            .map(|monitor| monitor.bounds_physical.y as i64)
            .min()
            .unwrap_or(0);
        let max_x = monitors
            .iter()
            .map(|monitor| {
                monitor.bounds_physical.x as i64 + i64::from(monitor.bounds_physical.width)
            })
            .max()
            .unwrap_or(min_x);
        let max_y = monitors
            .iter()
            .map(|monitor| {
                monitor.bounds_physical.y as i64 + i64::from(monitor.bounds_physical.height)
            })
            .max()
            .unwrap_or(min_y);

        fret_runtime::RunnerMonitorRectPhysicalV1 {
            x: min_x.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
            y: min_y.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
            width: (max_x - min_x).clamp(0, i64::from(u32::MAX)) as u32,
            height: (max_y - min_y).clamp(0, i64::from(u32::MAX)) as u32,
        }
    });

    fret_runtime::RunnerMonitorTopologySnapshotV1 {
        virtual_desktop_bounds_physical,
        monitors,
    }
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) const WINDOW_VISIBILITY_PADDING_PX: f64 = 40.0;

    pub(super) fn refresh_runner_monitor_topology_diagnostics(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let snapshot = collect_runner_monitor_topology_snapshot(event_loop);
        let _ = fret_runtime::update_runner_monitor_topology_diagnostics(&mut self.app, snapshot);
    }

    pub(super) fn virtual_desktop_bounds(window: &dyn Window) -> Option<MonitorRectF64> {
        let mut monitors = window.available_monitors();
        let first = monitors.next()?;

        let first_pos = first.position()?;
        let first_size = first.current_video_mode()?.size();
        let mut min_x = first_pos.x as f64;
        let mut min_y = first_pos.y as f64;
        let mut max_x = first_pos.x as f64 + first_size.width as f64;
        let mut max_y = first_pos.y as f64 + first_size.height as f64;

        for monitor in monitors {
            let Some(pos) = monitor.position() else {
                continue;
            };
            let Some(mode) = monitor.current_video_mode() else {
                continue;
            };
            let size = mode.size();
            min_x = min_x.min(pos.x as f64);
            min_y = min_y.min(pos.y as f64);
            max_x = max_x.max(pos.x as f64 + size.width as f64);
            max_y = max_y.max(pos.y as f64 + size.height as f64);
        }

        Some(MonitorRectF64 {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    pub(super) fn monitor_rects_physical(window: &dyn Window) -> Vec<MonitorRectF64> {
        window
            .available_monitors()
            .filter_map(|m| {
                let pos = m.position()?;
                let size = m.current_video_mode()?.size();
                Some(MonitorRectF64 {
                    min_x: pos.x as f64,
                    min_y: pos.y as f64,
                    max_x: pos.x as f64 + size.width as f64,
                    max_y: pos.y as f64 + size.height as f64,
                })
            })
            .collect()
    }

    #[cfg(target_os = "windows")]
    pub(super) fn monitor_scale_factor_for_point(
        window: &dyn Window,
        point: PhysicalPosition<f64>,
    ) -> Option<f64> {
        let mut best_scale = None;
        let mut best_dist2 = f64::INFINITY;

        for monitor in window.available_monitors() {
            let Some(pos) = monitor.position() else {
                continue;
            };
            let Some(mode) = monitor.current_video_mode() else {
                continue;
            };
            let size = mode.size();
            let rect = MonitorRectF64 {
                min_x: pos.x as f64,
                min_y: pos.y as f64,
                max_x: pos.x as f64 + size.width as f64,
                max_y: pos.y as f64 + size.height as f64,
            };

            let dx = if point.x < rect.min_x {
                rect.min_x - point.x
            } else if point.x > rect.max_x {
                point.x - rect.max_x
            } else {
                0.0
            };
            let dy = if point.y < rect.min_y {
                rect.min_y - point.y
            } else if point.y > rect.max_y {
                point.y - rect.max_y
            } else {
                0.0
            };
            let dist2 = dx * dx + dy * dy;
            if dist2 < best_dist2 {
                best_dist2 = dist2;
                best_scale = Some(monitor.scale_factor());
            }
            if dist2 == 0.0 {
                return Some(monitor.scale_factor());
            }
        }

        best_scale
    }

    pub(super) fn find_monitor_for_point(
        monitors: &[MonitorRectF64],
        point: PhysicalPosition<f64>,
    ) -> Option<usize> {
        if monitors.is_empty() {
            return None;
        }

        let mut best = 0usize;
        let mut best_dist2 = f64::INFINITY;
        for (i, m) in monitors.iter().enumerate() {
            let dx = if point.x < m.min_x {
                m.min_x - point.x
            } else if point.x > m.max_x {
                point.x - m.max_x
            } else {
                0.0
            };
            let dy = if point.y < m.min_y {
                m.min_y - point.y
            } else if point.y > m.max_y {
                point.y - m.max_y
            } else {
                0.0
            };
            let dist2 = dx * dx + dy * dy;
            if dist2 < best_dist2 {
                best_dist2 = dist2;
                best = i;
            }
            if dist2 == 0.0 {
                return Some(i);
            }
        }

        Some(best)
    }

    pub(super) fn find_monitor_for_rect(
        monitors: &[MonitorRectF64],
        rect: RectF64,
    ) -> Option<usize> {
        if monitors.is_empty() {
            return None;
        }
        if monitors.len() == 1 {
            return Some(0);
        }

        let mut best = 0usize;
        let mut best_area = -1.0f64;
        for (i, m) in monitors.iter().enumerate() {
            let ix0 = rect.min_x.max(m.min_x);
            let iy0 = rect.min_y.max(m.min_y);
            let ix1 = rect.max_x.min(m.max_x);
            let iy1 = rect.max_y.min(m.max_y);
            let iw = (ix1 - ix0).max(0.0);
            let ih = (iy1 - iy0).max(0.0);
            let area = iw * ih;
            if area > best_area {
                best_area = area;
                best = i;
            }
        }
        Some(best)
    }

    pub(super) fn clamp_window_outer_pos_to_monitor(
        desired_outer_x: f64,
        desired_outer_y: f64,
        outer_size: winit::dpi::PhysicalSize<u32>,
        monitor: MonitorRectF64,
        padding: f64,
    ) -> (f64, f64) {
        let w = outer_size.width as f64;
        let h = outer_size.height as f64;

        let pad_x = padding.min(w).max(0.0);
        let pad_y = padding.min(h).max(0.0);

        // Keep at least `pad` pixels of the window visible within the monitor bounds.
        let min_x = monitor.min_x - (w - pad_x);
        let max_x = monitor.max_x - pad_x;
        let min_y = monitor.min_y - (h - pad_y);
        let max_y = monitor.max_y - pad_y;

        let clamped_x = desired_outer_x.clamp(min_x, max_x.max(min_x));
        let clamped_y = desired_outer_y.clamp(min_y, max_y.max(min_y));
        (clamped_x, clamped_y)
    }

    pub(super) fn settle_window_outer_position(
        &self,
        window: &dyn Window,
        cursor_screen_pos: Option<PhysicalPosition<f64>>,
    ) -> Option<PhysicalPosition<i32>> {
        let outer_pos = window.outer_position().ok()?;
        let outer_size = window.outer_size();

        let desired_x = outer_pos.x as f64;
        let desired_y = outer_pos.y as f64;

        #[cfg(target_os = "windows")]
        if let Some(cursor) = cursor_screen_pos
            && let Some(work) = super::win32::monitor_work_area_for_point(cursor)
        {
            let (x, y) = Self::clamp_window_outer_pos_to_monitor(
                desired_x,
                desired_y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
            let target = PhysicalPosition::new(x.round() as i32, y.round() as i32);
            return (target != outer_pos).then_some(target);
        }

        let monitors = Self::monitor_rects_physical(window);
        let monitor = if let Some(cursor) = cursor_screen_pos
            && let Some(idx) = Self::find_monitor_for_point(&monitors, cursor)
            && let Some(m) = monitors.get(idx).copied()
        {
            Some(m)
        } else {
            let rect = RectF64 {
                min_x: desired_x,
                min_y: desired_y,
                max_x: desired_x + outer_size.width as f64,
                max_y: desired_y + outer_size.height as f64,
            };
            let idx = Self::find_monitor_for_rect(&monitors, rect);
            idx.and_then(|i| monitors.get(i).copied())
        };

        let monitor = monitor.or_else(|| Self::virtual_desktop_bounds(window));
        let monitor = monitor?;

        let (x, y) = Self::clamp_window_outer_pos_to_monitor(
            desired_x,
            desired_y,
            outer_size,
            monitor,
            Self::WINDOW_VISIBILITY_PADDING_PX,
        );

        let target = PhysicalPosition::new(x.round() as i32, y.round() as i32);
        if target == outer_pos {
            None
        } else {
            Some(target)
        }
    }
}

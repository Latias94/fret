use winit::event_loop::ActiveEventLoop;

use super::{WinitAppDriver, WinitRunner};

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
    pub(super) fn refresh_runner_monitor_topology_diagnostics(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let snapshot = collect_runner_monitor_topology_snapshot(event_loop);
        let _ = fret_runtime::update_runner_monitor_topology_diagnostics(&mut self.app, snapshot);
    }
}

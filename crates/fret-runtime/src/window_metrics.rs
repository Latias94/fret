use fret_core::{AppWindowId, Event, WindowMetricsService};

use crate::GlobalsHost;

pub fn apply_window_metrics_event(host: &mut impl GlobalsHost, window: AppWindowId, event: &Event) {
    host.with_global_mut(WindowMetricsService::default, |svc, _host| {
        svc.apply_event(window, event);
    });
}

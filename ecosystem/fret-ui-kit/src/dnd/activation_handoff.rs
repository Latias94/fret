use fret_core::{AppWindowId, Point, PointerId};
use fret_runtime::{DragHost, ModelHost, TickId, UiHost};
use fret_ui::action::{UiActionHost, UiDragActionHost, UiPointerActionHost};

use super::{DndActivationProbe, SensorOutput};

/// Clears probe tracking and upgrades a runtime drag session started via a `UiHost`.
pub fn complete_cross_window_drag_activation<H, F>(
    host: &mut H,
    activation_probe: &DndActivationProbe,
    window: AppWindowId,
    pointer_id: PointerId,
    begin_drag: F,
) where
    H: UiHost + ?Sized,
    F: FnOnce(&mut H),
{
    begin_drag(host);
    activation_probe.clear(ModelHost::models_mut(host), window, pointer_id);
    if let Some(drag) = DragHost::drag_mut(host, pointer_id) {
        drag.dragging = true;
    }
}

/// Clears probe tracking and upgrades a runtime drag session started via an object-safe action host.
pub fn complete_cross_window_drag_activation_for_action_host<F>(
    host: &mut dyn UiPointerActionHost,
    activation_probe: &DndActivationProbe,
    window: AppWindowId,
    pointer_id: PointerId,
    begin_drag: F,
) where
    F: FnOnce(&mut dyn UiPointerActionHost),
{
    begin_drag(host);
    activation_probe.clear(UiActionHost::models_mut(host), window, pointer_id);
    if let Some(drag) = UiDragActionHost::drag_mut(host, pointer_id) {
        drag.dragging = true;
    }
}

/// Probes activation and, once the sensor crosses its threshold, starts the runtime drag session.
pub fn try_begin_cross_window_drag_on_activation<H, F>(
    host: &mut H,
    activation_probe: &DndActivationProbe,
    window: AppWindowId,
    pointer_id: PointerId,
    start_tick: TickId,
    start_position: Point,
    position: Point,
    tick_id: TickId,
    begin_drag: F,
) -> SensorOutput
where
    H: UiHost + ?Sized,
    F: FnOnce(&mut H),
{
    let sensor = activation_probe.move_or_init(
        ModelHost::models_mut(host),
        window,
        pointer_id,
        start_tick,
        start_position,
        position,
        tick_id,
    );
    if matches!(sensor, SensorOutput::DragStart { .. }) {
        complete_cross_window_drag_activation(
            host,
            activation_probe,
            window,
            pointer_id,
            begin_drag,
        );
    }
    sensor
}

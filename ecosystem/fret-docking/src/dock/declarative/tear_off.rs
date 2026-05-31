use std::collections::HashMap;

use fret_core::{AppWindowId, PanelKey, Rect, Size};
use fret_runtime::Effect;
use fret_ui::UiHost;

use super::super::manager::DockManager;
use super::super::services::DockingPolicyService;
use super::super::types::{DockPanelDragPayload, DockTabsDragPayload};

pub(super) fn declarative_default_floating_rect_for_panel(
    panel: &PanelKey,
    cursor: fret_core::Point,
    tab_grab_offset: fret_core::Point,
    window_bounds: Rect,
    panel_last_sizes: &HashMap<PanelKey, Size>,
) -> Rect {
    let content = panel_last_sizes
        .get(panel)
        .copied()
        .unwrap_or(Size::new(fret_core::Px(360.0), fret_core::Px(240.0)));

    let inner_w = content.width.0.max(160.0);
    let inner_h = (content.height.0 + super::super::consts::DOCK_TAB_H.0).max(120.0);

    let border = super::super::consts::DOCK_FLOATING_BORDER.0.max(0.0);
    let title_h = super::super::consts::DOCK_FLOATING_TITLE_H.0.max(0.0);
    let outer_w = inner_w + border * 2.0;
    let outer_h = inner_h + border * 2.0 + title_h;

    let inner_origin = fret_core::Point::new(
        fret_core::Px(cursor.x.0 - tab_grab_offset.x.0),
        fret_core::Px(cursor.y.0 - tab_grab_offset.y.0),
    );
    let outer_origin = fret_core::Point::new(
        fret_core::Px(inner_origin.x.0 - border),
        fret_core::Px(inner_origin.y.0 - border - title_h),
    );

    clamp_declarative_floating_rect_to_bounds(
        Rect::new(
            outer_origin,
            Size::new(fret_core::Px(outer_w), fret_core::Px(outer_h)),
        ),
        window_bounds,
    )
}

pub(super) fn declarative_allow_tear_off_for_panel<H: UiHost>(
    app: &H,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
    source_window: AppWindowId,
    panel: &PanelKey,
) -> bool {
    if !allow_tear_off {
        return false;
    }
    let Some(dock) = app.global::<DockManager>() else {
        return false;
    };

    if crate::runtime::is_dock_floating_os_window(app, source_window)
        && dock.graph.collect_panels_in_window(source_window).len() == 1
    {
        return false;
    }

    if dock.graph.windows().len() > 1
        && dock.graph.collect_panels_in_window(source_window).len() == 1
    {
        return false;
    }

    let info = dock.panels.get(panel);
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    if policy
        .as_deref()
        .is_some_and(|policy| !policy.allow_tear_off(source_window, panel, info))
    {
        return false;
    }

    if dock.graph.windows().len() <= 1 || allow_multi_window_tear_off {
        return true;
    }

    policy
        .as_deref()
        .is_some_and(|policy| policy.allow_multi_window_tear_off(source_window, panel, info))
}

fn declarative_is_outside_bounds_with_margin(
    bounds: Rect,
    position: fret_core::Point,
    margin: fret_core::Px,
) -> bool {
    position.x.0 < bounds.origin.x.0 - margin.0
        || position.y.0 < bounds.origin.y.0 - margin.0
        || position.x.0 > bounds.origin.x.0 + bounds.size.width.0 + margin.0
        || position.y.0 > bounds.origin.y.0 + bounds.size.height.0 + margin.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarativeTearOffRetryTarget {
    Panel,
    Tabs,
}

#[derive(Default)]
pub(super) struct DeclarativeTearOffHoverResult {
    pub(super) effects: Vec<Effect>,
    pub(super) requested_tear_off: bool,
}

pub(super) fn declarative_resolve_tear_off_hover<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    bounds: Rect,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
) -> DeclarativeTearOffHoverResult {
    let now_frame = app.frame_id();
    let now_tick = app.tick_id();
    let Some(drag) = app.drag(pointer_id) else {
        return DeclarativeTearOffHoverResult::default();
    };
    let source_window = drag.source_window;
    let panel_payload = drag.payload::<DockPanelDragPayload>().cloned();
    let tabs_payload = drag.payload::<DockTabsDragPayload>().cloned();
    if panel_payload.is_none() && tabs_payload.is_none() {
        return DeclarativeTearOffHoverResult::default();
    }

    let oob = declarative_is_outside_bounds_with_margin(bounds, position, fret_core::Px(10.0));
    let mut set_tear_off_oob_start_frame: Option<Option<fret_runtime::FrameId>> = None;
    let mut mark_tear_off_requested = false;
    let mut effects = Vec::new();

    if let Some(payload) = panel_payload.as_ref() {
        if allow_tear_off && source_window == window {
            match (oob, payload.tear_off_oob_start_frame) {
                (true, None) => {
                    set_tear_off_oob_start_frame = Some(Some(now_frame));
                }
                (false, Some(_)) => {
                    set_tear_off_oob_start_frame = Some(None);
                }
                _ => {}
            }
        }

        let stable_oob = oob
            && payload
                .tear_off_oob_start_frame
                .is_some_and(|frame| frame != now_frame);
        let disallow_chained_tear_off = app.global::<DockManager>().is_some_and(|dock| {
            dock.graph.windows().len() > 1 && dock.graph.collect_panels_in_window(window).len() == 1
        });
        let allow_panel_tear_off = declarative_allow_tear_off_for_panel(
            app,
            allow_tear_off,
            allow_multi_window_tear_off,
            source_window,
            &payload.panel,
        );
        let requested_tear_off = allow_panel_tear_off
            && source_window == window
            && stable_oob
            && !disallow_chained_tear_off
            && !payload.tear_off_requested;

        if requested_tear_off {
            mark_tear_off_requested = true;
            effects.push(Effect::Dock(
                fret_core::DockOp::RequestFloatPanelToNewWindow {
                    source_window,
                    panel: payload.panel.clone(),
                    anchor: Some(fret_core::WindowAnchor {
                        window,
                        position: payload.grab_offset,
                    }),
                },
            ));
        }
    } else if let Some(payload) = tabs_payload.as_ref() {
        if allow_tear_off && source_window == window {
            match (oob, payload.tear_off_oob_start_frame) {
                (true, None) => {
                    set_tear_off_oob_start_frame = Some(Some(now_frame));
                }
                (false, Some(_)) => {
                    set_tear_off_oob_start_frame = Some(None);
                }
                _ => {}
            }
        }

        let stable_oob = oob
            && payload
                .tear_off_oob_start_frame
                .is_some_and(|frame| frame != now_frame);
        let panel = payload
            .tabs
            .get(payload.active)
            .or_else(|| payload.tabs.first());
        let allow_tabs_tear_off = panel.is_some_and(|panel| {
            declarative_allow_tear_off_for_panel(
                app,
                allow_tear_off,
                allow_multi_window_tear_off,
                source_window,
                panel,
            )
        });
        let requested_tear_off = allow_tabs_tear_off
            && source_window == window
            && stable_oob
            && !payload.tear_off_requested;

        if requested_tear_off && let Some(panel) = panel {
            mark_tear_off_requested = true;
            effects.push(Effect::Dock(
                fret_core::DockOp::RequestFloatTabsToNewWindow {
                    source_window,
                    source_tabs: payload.source_tabs,
                    panel: panel.clone(),
                    anchor: Some(fret_core::WindowAnchor {
                        window,
                        position: payload.grab_offset,
                    }),
                },
            ));
        }
    }

    let retry_target = (!mark_tear_off_requested
        && !bounds.contains(position)
        && source_window == window)
        .then(|| {
            if let Some(payload) = panel_payload.as_ref() {
                let requested_at = payload.tear_off_requested_at_tick?;
                if !payload.tear_off_requested || now_tick.0.saturating_sub(requested_at.0) <= 600 {
                    return None;
                }
                let dock = app.global::<DockManager>()?;
                dock.graph
                    .find_panel_in_window(source_window, &payload.panel)
                    .is_some()
                    .then_some(DeclarativeTearOffRetryTarget::Panel)
            } else if let Some(payload) = tabs_payload.as_ref() {
                let requested_at = payload.tear_off_requested_at_tick?;
                let panel = payload
                    .tabs
                    .get(payload.active)
                    .or_else(|| payload.tabs.first())?;
                if !payload.tear_off_requested || now_tick.0.saturating_sub(requested_at.0) <= 600 {
                    return None;
                }
                let dock = app.global::<DockManager>()?;
                dock.graph
                    .find_panel_in_window(source_window, panel)
                    .is_some()
                    .then_some(DeclarativeTearOffRetryTarget::Tabs)
            } else {
                None
            }
        })
        .flatten();

    if let Some(drag) = app.drag_mut(pointer_id) {
        drag.position = position;
        drag.dragging = true;
        if let Some(payload) = drag.payload_mut::<DockPanelDragPayload>() {
            if retry_target == Some(DeclarativeTearOffRetryTarget::Panel) {
                payload.tear_off_requested = false;
                payload.tear_off_requested_at_tick = None;
                payload.tear_off_oob_start_frame = None;
            }
            if mark_tear_off_requested {
                payload.tear_off_requested = true;
                payload.tear_off_requested_at_tick = Some(now_tick);
                payload.tear_off_oob_start_frame = None;
            }
            if let Some(next) = set_tear_off_oob_start_frame {
                payload.tear_off_oob_start_frame = next;
            }
        } else if let Some(payload) = drag.payload_mut::<DockTabsDragPayload>() {
            if retry_target == Some(DeclarativeTearOffRetryTarget::Tabs) {
                payload.tear_off_requested = false;
                payload.tear_off_requested_at_tick = None;
                payload.tear_off_oob_start_frame = None;
            }
            if mark_tear_off_requested {
                payload.tear_off_requested = true;
                payload.tear_off_requested_at_tick = Some(now_tick);
                payload.tear_off_oob_start_frame = None;
            }
            if let Some(next) = set_tear_off_oob_start_frame {
                payload.tear_off_oob_start_frame = next;
            }
        }
    }

    DeclarativeTearOffHoverResult {
        effects,
        requested_tear_off: mark_tear_off_requested,
    }
}

pub(super) fn clamp_declarative_floating_rect_to_bounds(rect: Rect, bounds: Rect) -> Rect {
    let mut out = rect;
    if bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0 {
        let min_x = bounds.origin.x.0;
        let min_y = bounds.origin.y.0;
        let max_x = bounds.origin.x.0 + (bounds.size.width.0 - out.size.width.0).max(0.0);
        let max_y = bounds.origin.y.0 + (bounds.size.height.0 - out.size.height.0).max(0.0);
        out.origin.x = fret_core::Px(out.origin.x.0.clamp(min_x, max_x.max(min_x)));
        out.origin.y = fret_core::Px(out.origin.y.0.clamp(min_y, max_y.max(min_y)));
    }
    out
}

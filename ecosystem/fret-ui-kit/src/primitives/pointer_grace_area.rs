//! Pointer grace area helpers (Radix Menu submenu safe-hover outcomes).
//!
//! Radix menus use a “pointer grace area” concept to avoid closing submenus while the pointer
//! moves diagonally from a trigger into the submenu panel.
//!
//! This module provides a small, deterministic policy helper:
//! - compute whether the pointer is inside a geometry-only corridor,
//! - cancel a pending close-delay timer when safe,
//! - arm a close-delay timer when unsafe.
//!
//! This primitive intentionally does not decide *what* to close when the timer fires; that
//! remains component policy.

use std::time::Duration;

use fret_core::{Point, Px, Rect};
use fret_runtime::{Effect, Model, TimerToken};
use fret_ui::action::{ActionCx, PointerMoveCx, UiActionHost};

use crate::headless::safe_hover::safe_hover_contains;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerGraceGeometry {
    pub reference: Rect,
    pub floating: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerGraceConfig {
    pub buffer: Px,
    pub close_delay: Duration,
}

impl PointerGraceConfig {
    pub fn new(buffer: Px, close_delay: Duration) -> Self {
        Self {
            buffer,
            close_delay,
        }
    }
}

/// Update a close-delay timer for a submenu safe-hover corridor.
///
/// Behavior:
/// - When the pointer is inside the grace corridor, any pending close timer is cancelled.
/// - When the pointer is outside the corridor, a close timer is armed if none is pending.
///
/// Returns `true` when it changes the timer state.
pub fn update_close_timer_on_pointer_move(
    host: &mut dyn UiActionHost,
    acx: ActionCx,
    mv: PointerMoveCx,
    geometry: Option<PointerGraceGeometry>,
    config: PointerGraceConfig,
    close_timer: &Model<Option<TimerToken>>,
) -> bool {
    let Some(geometry) = geometry else {
        return false;
    };

    let safe = safe_hover_contains(
        mv.position,
        geometry.reference,
        geometry.floating,
        config.buffer,
    );

    let pending = host.models_mut().read(close_timer, |v| *v).ok().flatten();
    if safe {
        let Some(token) = pending else {
            return false;
        };
        host.push_effect(Effect::CancelTimer { token });
        let _ = host.models_mut().update(close_timer, |v| *v = None);
        host.request_redraw(acx.window);
        return true;
    }

    if pending.is_some() {
        return false;
    }

    let token = host.next_timer_token();
    host.push_effect(Effect::SetTimer {
        window: Some(acx.window),
        token,
        after: config.close_delay,
        repeat: None,
    });
    let _ = host.models_mut().update(close_timer, |v| *v = Some(token));
    host.request_redraw(acx.window);
    true
}

pub fn last_pointer_is_safe(pointer: Point, geometry: PointerGraceGeometry, buffer: Px) -> bool {
    safe_hover_contains(pointer, geometry.reference, geometry.floating, buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    #[test]
    fn last_pointer_is_safe_matches_geometry_corridor() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(2.0)), Size::new(Px(10.0), Px(10.0)));
        let geometry = PointerGraceGeometry {
            reference,
            floating,
        };

        assert!(last_pointer_is_safe(
            Point::new(Px(12.0), Px(5.0)),
            geometry,
            Px(0.0)
        ));
        assert!(!last_pointer_is_safe(
            Point::new(Px(12.0), Px(30.0)),
            geometry,
            Px(0.0)
        ));
    }
}


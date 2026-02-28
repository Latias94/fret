use crate::embla::drag_release::{
    DragReleaseConfig, DragReleaseResult, PointerKind, compute_release,
};
use crate::embla::limit::Limit;
use crate::embla::scroll_body::ScrollBody;
use crate::embla::scroll_bounds::{ScrollBounds, ScrollBoundsConfig};
use crate::embla::scroll_limit::scroll_limit;
use crate::embla::scroll_target::{ScrollTarget, Target};
use crate::embla::utils::{DIRECTION_NONE, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectEvent {
    pub target_snap: usize,
    pub source_snap: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineConfig {
    pub loop_enabled: bool,
    pub drag_free: bool,
    pub skip_snaps: bool,
    pub duration: f32,
    pub base_friction: f32,
    pub view_size: f32,
    pub start_snap: usize,
}

/// Minimal Embla-style engine state for headless parity work.
///
/// This is intentionally not a full port of `Engine.ts`. It is a small, composable core that
/// supports:
/// - scroll target selection (`ScrollTarget`)
/// - scroll integration (`ScrollBody`)
/// - drag release shaping (`DragHandler.up` math)
/// - select event emission (index changes)
///
/// Upstream references:
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/Engine.ts`
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollTo.ts`
/// - `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollAnimator.ts`
#[derive(Debug, Clone, PartialEq)]
pub struct Engine {
    pub scroll_body: ScrollBody,
    pub scroll_target: ScrollTarget,
    pub index_current: usize,
    pub index_previous: usize,

    pub scroll_bounds: ScrollBounds,

    config: EngineConfig,
    content_size: f32,
    limit: Limit,
}

impl Engine {
    pub fn new(scroll_snaps: Vec<f32>, content_size: f32, config: EngineConfig) -> Self {
        let max_index = scroll_snaps.len().saturating_sub(1);
        let start_snap = config.start_snap.min(max_index);
        let start_location = scroll_snaps.get(start_snap).copied().unwrap_or_default();

        let limit = scroll_limit(content_size, &scroll_snaps, config.loop_enabled);
        let scroll_target = ScrollTarget::new(
            config.loop_enabled,
            scroll_snaps,
            content_size,
            limit,
            start_location,
        );
        let mut scroll_body =
            ScrollBody::new(start_location, config.duration, config.base_friction);
        scroll_body.set_target(start_location);
        let mut scroll_bounds = ScrollBounds::new(ScrollBoundsConfig {
            view_size: config.view_size.max(0.0),
        });
        scroll_bounds.toggle_active(!config.loop_enabled);

        Self {
            scroll_body,
            scroll_target,
            index_current: start_snap,
            index_previous: start_snap,
            scroll_bounds,
            config,
            content_size,
            limit,
        }
    }

    pub fn set_options(
        &mut self,
        loop_enabled: bool,
        drag_free: bool,
        skip_snaps: bool,
        duration: f32,
    ) {
        self.config.loop_enabled = loop_enabled;
        self.config.drag_free = drag_free;
        self.config.skip_snaps = skip_snaps;

        let duration = duration.max(0.0);
        if (self.config.duration - duration).abs() > 0.0001 {
            self.config.duration = duration;
            self.scroll_body.set_base_duration(duration);
        }
    }

    pub fn reinit(
        &mut self,
        scroll_snaps: Vec<f32>,
        content_size: f32,
        view_size: f32,
    ) -> Option<SelectEvent> {
        let content_size = content_size.max(0.0);
        let view_size = view_size.max(0.0);

        self.config.view_size = view_size;

        let mut scroll_snaps = scroll_snaps;
        if scroll_snaps.is_empty() {
            scroll_snaps.push(0.0);
        }

        let limit = scroll_limit(content_size, &scroll_snaps, self.config.loop_enabled);

        // A geometry-driven re-init should keep motion state but ensure the integrator stays within
        // the updated limits. Otherwise the recipe can end up displaying a clamped offset while the
        // engine continues integrating from an out-of-bounds location/target.
        if self.config.loop_enabled {
            if limit.length != 0.0 {
                self.scroll_body
                    .set_location(limit.remove_offset(self.scroll_body.location()));
                self.scroll_body
                    .set_target(limit.remove_offset(self.scroll_body.target()));
            }
        } else {
            self.scroll_body
                .set_location(limit.clamp(self.scroll_body.location()));
            self.scroll_body
                .set_target(limit.clamp(self.scroll_body.target()));
        }

        let scroll_target = ScrollTarget::new(
            self.config.loop_enabled,
            scroll_snaps,
            content_size,
            limit,
            self.scroll_body.target(),
        );

        self.limit = limit;
        self.content_size = content_size;
        self.scroll_target = scroll_target;
        self.scroll_bounds = ScrollBounds::new(ScrollBoundsConfig { view_size });
        self.scroll_bounds.toggle_active(!self.config.loop_enabled);

        self.sync_target_vector();

        let next = self
            .scroll_target
            .by_distance(0.0, true)
            .index
            .min(self.scroll_target.max_index());
        if next != self.index_current {
            let source_snap = self.index_current;
            self.index_previous = source_snap;
            self.index_current = next;
            Some(SelectEvent {
                target_snap: next,
                source_snap,
            })
        } else {
            None
        }
    }

    #[inline]
    fn sync_target_vector(&mut self) {
        self.scroll_target
            .set_target_vector(self.scroll_body.target());
    }

    pub fn constrain_bounds(&mut self, pointer_down: bool) {
        self.scroll_bounds
            .constrain(self.limit, &mut self.scroll_body, pointer_down);
    }

    fn apply_target(&mut self, target: Target) -> Option<SelectEvent> {
        let source_snap = self.index_current;
        if target.distance != 0.0 {
            self.scroll_body.add_target(target.distance);
        }
        self.sync_target_vector();

        if target.index != source_snap {
            self.index_previous = source_snap;
            self.index_current = target.index;
            Some(SelectEvent {
                target_snap: target.index,
                source_snap,
            })
        } else {
            None
        }
    }

    /// One engine step (typically one rendered frame).
    pub fn tick(&mut self, pointer_down: bool) {
        self.scroll_body.seek();
        self.constrain_bounds(pointer_down);
        self.normalize_loop_entities();
        self.sync_target_vector();
    }

    /// Keeps loop-enabled motion values within the scroll limit by applying Embla-style loop distances.
    ///
    /// Unlike `ScrollBounds`, this is only active for `loop=true` and does not clamp; it wraps the
    /// location/target by the loop length while keeping integrator velocity intact.
    pub fn normalize_loop_entities(&mut self) {
        if !self.config.loop_enabled {
            return;
        }
        if self.limit.length == 0.0 {
            return;
        }

        let location = self.scroll_body.location();
        let wrapped = self.limit.remove_offset(location);
        let delta = wrapped - location;
        if delta == 0.0 {
            return;
        }

        self.scroll_body.add_loop_distance(delta);
        self.sync_target_vector();
    }

    pub fn scroll_to_distance(
        &mut self,
        distance: f32,
        snap_to_closest: bool,
    ) -> Option<SelectEvent> {
        self.sync_target_vector();
        let target = self.scroll_target.by_distance(distance, snap_to_closest);
        self.apply_target(target)
    }

    pub fn scroll_to_index(&mut self, index: usize, direction: Direction) -> Option<SelectEvent> {
        self.sync_target_vector();
        let target = self.scroll_target.by_index(index, direction);
        self.apply_target(target)
    }

    pub fn scroll_to_next(&mut self) -> Option<SelectEvent> {
        let max = self.scroll_target.max_index();
        let next = if self.config.loop_enabled {
            (self.index_current + 1) % (max + 1).max(1)
        } else {
            (self.index_current + 1).min(max)
        };
        self.scroll_to_index(next, DIRECTION_NONE)
    }

    pub fn scroll_to_prev(&mut self) -> Option<SelectEvent> {
        let max = self.scroll_target.max_index();
        let prev = if self.config.loop_enabled {
            if max == 0 {
                0
            } else if self.index_current == 0 {
                max
            } else {
                self.index_current - 1
            }
        } else {
            self.index_current.saturating_sub(1)
        };
        self.scroll_to_index(prev, DIRECTION_NONE)
    }

    /// Apply Embla-style drag release shaping and scroll to the resulting target.
    ///
    /// `pointer_delta` is the signed drag delta in the main axis.
    /// `direction` is the axis direction function (Embla `axis.direction`).
    pub fn on_drag_release(
        &mut self,
        pointer_kind: PointerKind,
        pointer_delta: f32,
        direction: impl Fn(f32) -> f32,
    ) -> (DragReleaseResult, Option<SelectEvent>) {
        self.sync_target_vector();
        let cfg = DragReleaseConfig {
            drag_free: self.config.drag_free,
            skip_snaps: self.config.skip_snaps,
            view_size: self.config.view_size,
            base_friction: self.config.base_friction,
        };

        let out = compute_release(
            cfg,
            pointer_kind,
            &self.scroll_target,
            self.index_current,
            pointer_delta,
            direction,
        );

        self.scroll_body
            .use_duration(out.duration)
            .use_friction(out.friction);
        let ev = self.scroll_to_distance(out.force, !self.config.drag_free);
        (out, ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embla::drag_release::PointerKind;

    #[test]
    fn scroll_to_distance_emits_select_when_index_changes() {
        let snaps = vec![0.0, -100.0, -200.0, -300.0];
        let mut engine = Engine::new(
            snaps,
            300.0,
            EngineConfig {
                loop_enabled: false,
                drag_free: false,
                skip_snaps: false,
                duration: 25.0,
                base_friction: 0.68,
                view_size: 320.0,
                start_snap: 0,
            },
        );

        let ev = engine
            .scroll_to_distance(-130.0, true)
            .expect("select event");
        assert_eq!(ev.source_snap, 0);
        assert_eq!(ev.target_snap, 1);
        assert_eq!(engine.index_current, 1);
    }

    #[test]
    fn drag_release_shapes_duration_and_friction() {
        let snaps = vec![0.0, -100.0, -200.0, -300.0];
        let mut engine = Engine::new(
            snaps,
            300.0,
            EngineConfig {
                loop_enabled: false,
                drag_free: false,
                skip_snaps: false,
                duration: 25.0,
                base_friction: 0.68,
                view_size: 320.0,
                start_snap: 0,
            },
        );

        let (release, _ev) = engine.on_drag_release(PointerKind::Mouse, -0.25, |v| v);
        assert!(release.duration <= 25.0);
        assert!(release.friction >= 0.68);
    }

    #[test]
    fn reinit_updates_limit_and_keeps_location() {
        let snaps = vec![0.0, -100.0, -200.0, -300.0];
        let mut engine = Engine::new(
            snaps,
            300.0,
            EngineConfig {
                loop_enabled: false,
                drag_free: false,
                skip_snaps: false,
                duration: 25.0,
                base_friction: 0.68,
                view_size: 320.0,
                start_snap: 0,
            },
        );

        engine.scroll_body.set_location(-250.0);
        engine.scroll_body.set_target(-250.0);

        let ev = engine.reinit(vec![0.0, -120.0, -240.0], 240.0, 320.0);
        assert!(ev.is_some());
        assert_eq!(engine.config.view_size, 320.0);
        assert_eq!(engine.scroll_body.location(), -240.0);
        assert_eq!(engine.scroll_body.target(), -240.0);
        assert_eq!(engine.index_current, 2);
    }

    #[test]
    fn loop_normalization_wraps_location_without_resetting_motion() {
        // 5 uniform slides, view size 100 => content size 500, scroll snaps 0..-400.
        let snaps = vec![0.0, -100.0, -200.0, -300.0, -400.0];
        let mut engine = Engine::new(
            snaps,
            500.0,
            EngineConfig {
                loop_enabled: true,
                drag_free: false,
                skip_snaps: false,
                duration: 25.0,
                base_friction: 0.9,
                view_size: 100.0,
                start_snap: 0,
            },
        );

        // Simulate a drag past max bound (location > 0).
        engine.scroll_body.set_location(20.0);
        engine.scroll_body.set_target(20.0);
        engine.scroll_target.set_target_vector(20.0);

        engine.normalize_loop_entities();

        assert!(
            engine.scroll_body.location() <= engine.limit.max,
            "expected wrapped location within max bound; loc={}",
            engine.scroll_body.location()
        );
        assert!(
            engine.scroll_body.location() >= engine.limit.min,
            "expected wrapped location within min bound; loc={}",
            engine.scroll_body.location()
        );
    }

    #[test]
    fn loop_scroll_to_next_wraps_selection_index() {
        let snaps = vec![0.0, -100.0, -200.0, -300.0, -400.0];
        let mut engine = Engine::new(
            snaps,
            500.0,
            EngineConfig {
                loop_enabled: true,
                drag_free: false,
                skip_snaps: false,
                duration: 0.0,
                base_friction: 0.9,
                view_size: 100.0,
                start_snap: 0,
            },
        );

        for _ in 0..5 {
            let _ = engine.scroll_to_next();
        }

        assert_eq!(engine.index_current, 0);
    }
}

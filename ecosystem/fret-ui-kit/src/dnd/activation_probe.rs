use fret_core::{AppWindowId, Point, PointerId};
use fret_runtime::{DragKindId, ModelStore, TickId};

use super::{
    ActivationConstraint, DND_SCOPE_DEFAULT, DndScopeId, DndServiceModel, clear_pointer_in_scope,
    handle_sensor_move_or_init_in_scope,
};

#[derive(Clone)]
pub struct DndActivationProbeConfig {
    pub kind: DragKindId,
    pub scope: DndScopeId,
    pub activation_constraint: ActivationConstraint,
}

impl DndActivationProbeConfig {
    pub fn for_kind(kind: DragKindId) -> Self {
        Self {
            kind,
            scope: DND_SCOPE_DEFAULT,
            activation_constraint: ActivationConstraint::Distance { px: 2.0 },
        }
    }

    pub fn scope(mut self, scope: DndScopeId) -> Self {
        self.scope = scope;
        self
    }

    pub fn activation_constraint(mut self, constraint: ActivationConstraint) -> Self {
        self.activation_constraint = constraint;
        self
    }
}

#[derive(Clone)]
pub struct DndActivationProbe {
    svc: DndServiceModel,
    cfg: DndActivationProbeConfig,
}

impl DndActivationProbe {
    pub fn new(svc: DndServiceModel, cfg: DndActivationProbeConfig) -> Self {
        Self { svc, cfg }
    }

    pub fn move_or_init(
        &self,
        models: &mut ModelStore,
        window: AppWindowId,
        pointer_id: PointerId,
        start_tick: TickId,
        start_position: Point,
        position: Point,
        tick_id: TickId,
    ) -> fret_dnd::SensorOutput {
        handle_sensor_move_or_init_in_scope(
            models,
            &self.svc,
            window,
            self.cfg.kind,
            self.cfg.scope,
            pointer_id,
            start_tick,
            start_position,
            position,
            tick_id,
            self.cfg.activation_constraint,
        )
    }

    pub fn clear(&self, models: &mut ModelStore, window: AppWindowId, pointer_id: PointerId) {
        clear_pointer_in_scope(
            models,
            &self.svc,
            window,
            self.cfg.kind,
            self.cfg.scope,
            pointer_id,
        );
    }
}

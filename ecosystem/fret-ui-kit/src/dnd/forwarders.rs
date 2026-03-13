use std::sync::Arc;

use fret_core::{MouseButton, Point, PointerType, Rect};
use fret_runtime::{DragKindId, FrameId, Model};
use fret_ui::action::{
    ActionCx, OnPointerCancel, OnPointerDown, OnPointerMove, OnPointerUp, PointerCancelCx,
    PointerDownCx, PointerMoveCx, PointerUpCx, UiPointerActionHost,
};

use super::{
    ActivationConstraint, AutoScrollConfig, CollisionStrategy, DndScopeId, DndServiceModel,
    DndUpdate, handle_pointer_cancel_in_scope, handle_pointer_down_in_scope,
    handle_pointer_move_in_scope, handle_pointer_up_in_scope,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DndPointerActivationConstraints {
    pub mouse: ActivationConstraint,
    pub touch: ActivationConstraint,
    pub pen: ActivationConstraint,
    pub unknown: ActivationConstraint,
}

impl DndPointerActivationConstraints {
    pub fn uniform(constraint: ActivationConstraint) -> Self {
        Self {
            mouse: constraint,
            touch: constraint,
            pen: constraint,
            unknown: constraint,
        }
    }

    pub fn resolve(&self, pointer_type: PointerType) -> ActivationConstraint {
        match pointer_type {
            PointerType::Mouse => self.mouse,
            PointerType::Touch => self.touch,
            PointerType::Pen => self.pen,
            PointerType::Unknown => self.unknown,
        }
    }

    pub fn mouse(mut self, constraint: ActivationConstraint) -> Self {
        self.mouse = constraint;
        self
    }

    pub fn touch(mut self, constraint: ActivationConstraint) -> Self {
        self.touch = constraint;
        self
    }

    pub fn pen(mut self, constraint: ActivationConstraint) -> Self {
        self.pen = constraint;
        self
    }

    pub fn unknown(mut self, constraint: ActivationConstraint) -> Self {
        self.unknown = constraint;
        self
    }
}

impl Default for DndPointerActivationConstraints {
    fn default() -> Self {
        Self::uniform(ActivationConstraint::Distance { px: 2.0 })
    }
}

#[derive(Clone)]
pub struct DndPointerForwardersConfig {
    pub kind: DragKindId,
    pub scope: DndScopeId,
    pub activation_constraint: ActivationConstraint,
    pub pointer_activation_constraints: Option<DndPointerActivationConstraints>,
    pub collision_strategy: CollisionStrategy,
    pub autoscroll: Option<(Rect, AutoScrollConfig)>,
    pub prevent_activation_on_text_input: bool,
    pub prevent_activation_on_pressable_descendant: bool,
    pub prevent_activation: Option<Arc<dyn Fn(ActionCx, PointerDownCx) -> bool + 'static>>,
    pub capture_pointer_on_down: bool,
    pub consume_events: bool,
    pub update_model: Option<Model<DndUpdate>>,
    pub on_update:
        Option<Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, &DndUpdate) + 'static>>,
}

impl DndPointerForwardersConfig {
    pub fn for_kind(kind: DragKindId) -> Self {
        Self {
            kind,
            scope: super::DND_SCOPE_DEFAULT,
            activation_constraint: ActivationConstraint::Distance { px: 2.0 },
            pointer_activation_constraints: None,
            collision_strategy: CollisionStrategy::ClosestCenter,
            autoscroll: None,
            prevent_activation_on_text_input: false,
            prevent_activation_on_pressable_descendant: false,
            prevent_activation: None,
            capture_pointer_on_down: true,
            consume_events: true,
            update_model: None,
            on_update: None,
        }
    }

    pub fn scope(mut self, scope: DndScopeId) -> Self {
        self.scope = scope;
        self
    }

    pub fn activation_constraint(mut self, constraint: ActivationConstraint) -> Self {
        self.activation_constraint = constraint;
        self.pointer_activation_constraints = None;
        self
    }

    pub fn pointer_activation_constraints(
        mut self,
        constraints: DndPointerActivationConstraints,
    ) -> Self {
        self.pointer_activation_constraints = Some(constraints);
        self
    }

    pub fn mouse_activation_constraint(mut self, constraint: ActivationConstraint) -> Self {
        self.pointer_activation_constraints_mut().mouse = constraint;
        self
    }

    pub fn touch_activation_constraint(mut self, constraint: ActivationConstraint) -> Self {
        self.pointer_activation_constraints_mut().touch = constraint;
        self
    }

    pub fn pen_activation_constraint(mut self, constraint: ActivationConstraint) -> Self {
        self.pointer_activation_constraints_mut().pen = constraint;
        self
    }

    pub fn unknown_activation_constraint(mut self, constraint: ActivationConstraint) -> Self {
        self.pointer_activation_constraints_mut().unknown = constraint;
        self
    }

    pub fn collision_strategy(mut self, strategy: CollisionStrategy) -> Self {
        self.collision_strategy = strategy;
        self
    }

    pub fn autoscroll(mut self, autoscroll: Option<(Rect, AutoScrollConfig)>) -> Self {
        self.autoscroll = autoscroll;
        self
    }

    pub fn prevent_activation_on_text_input(mut self, prevent: bool) -> Self {
        self.prevent_activation_on_text_input = prevent;
        self
    }

    pub fn prevent_activation_on_pressable_descendant(mut self, prevent: bool) -> Self {
        self.prevent_activation_on_pressable_descendant = prevent;
        self
    }

    pub fn prevent_activation(
        mut self,
        f: Arc<dyn Fn(ActionCx, PointerDownCx) -> bool + 'static>,
    ) -> Self {
        self.prevent_activation = Some(f);
        self
    }

    pub fn capture_pointer_on_down(mut self, capture: bool) -> Self {
        self.capture_pointer_on_down = capture;
        self
    }

    pub fn consume_events(mut self, consume: bool) -> Self {
        self.consume_events = consume;
        self
    }

    pub fn update_model(mut self, model: Model<DndUpdate>) -> Self {
        self.update_model = Some(model);
        self
    }

    pub fn on_update(
        mut self,
        f: Arc<dyn Fn(&mut dyn UiPointerActionHost, ActionCx, &DndUpdate) + 'static>,
    ) -> Self {
        self.on_update = Some(f);
        self
    }

    fn pointer_activation_constraints_mut(&mut self) -> &mut DndPointerActivationConstraints {
        self.pointer_activation_constraints.get_or_insert_with(|| {
            DndPointerActivationConstraints::uniform(self.activation_constraint)
        })
    }

    fn resolved_activation_constraint(&self, pointer_type: PointerType) -> ActivationConstraint {
        self.pointer_activation_constraints
            .as_ref()
            .map(|constraints| constraints.resolve(pointer_type))
            .unwrap_or(self.activation_constraint)
    }

    fn should_prevent_activation(&self, action_cx: ActionCx, down: PointerDownCx) -> bool {
        if self.prevent_activation_on_text_input && down.hit_is_text_input {
            return true;
        }

        if self.prevent_activation_on_pressable_descendant
            && down
                .hit_pressable_target
                .is_some_and(|target| target != action_cx.target)
        {
            return true;
        }

        self.prevent_activation
            .as_ref()
            .is_some_and(|prevent| prevent(action_cx, down))
    }
}

#[derive(Clone)]
pub struct DndPointerForwarders {
    svc: DndServiceModel,
    frame_id: FrameId,
    cfg: DndPointerForwardersConfig,
}

impl DndPointerForwarders {
    pub fn new(svc: DndServiceModel, frame_id: FrameId, cfg: DndPointerForwardersConfig) -> Self {
        Self { svc, frame_id, cfg }
    }

    pub fn on_pointer_down(&self) -> OnPointerDown {
        let svc = self.svc.clone();
        let frame_id = self.frame_id;
        let cfg = self.cfg.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, down: PointerDownCx| {
                if down.button != MouseButton::Left {
                    return false;
                }

                if cfg.should_prevent_activation(action_cx, down) {
                    return false;
                }

                if cfg.capture_pointer_on_down {
                    host.capture_pointer();
                }

                let update = handle_pointer_down_in_scope(
                    host.models_mut(),
                    &svc,
                    action_cx.window,
                    frame_id,
                    cfg.kind,
                    cfg.scope,
                    down.pointer_id,
                    down.position,
                    down.tick_id,
                    cfg.resolved_activation_constraint(down.pointer_type),
                    cfg.collision_strategy,
                    cfg.autoscroll,
                );

                if let Some(model) = cfg.update_model.as_ref() {
                    let update = update.clone();
                    let _ = host.models_mut().update(model, |v| *v = update);
                }

                if let Some(cb) = cfg.on_update.as_ref() {
                    cb(host, action_cx, &update);
                }

                cfg.consume_events
            },
        )
    }

    pub fn on_pointer_move(&self) -> OnPointerMove {
        let svc = self.svc.clone();
        let frame_id = self.frame_id;
        let cfg = self.cfg.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, mv: PointerMoveCx| {
                let update = handle_pointer_move_in_scope(
                    host.models_mut(),
                    &svc,
                    action_cx.window,
                    frame_id,
                    cfg.kind,
                    cfg.scope,
                    mv.pointer_id,
                    mv.position,
                    mv.tick_id,
                    cfg.resolved_activation_constraint(mv.pointer_type),
                    cfg.collision_strategy,
                    cfg.autoscroll,
                );

                if let Some(model) = cfg.update_model.as_ref() {
                    let update = update.clone();
                    let _ = host.models_mut().update(model, |v| *v = update);
                }

                if let Some(cb) = cfg.on_update.as_ref() {
                    cb(host, action_cx, &update);
                }

                cfg.consume_events
            },
        )
    }

    pub fn on_pointer_up(&self) -> OnPointerUp {
        let svc = self.svc.clone();
        let frame_id = self.frame_id;
        let cfg = self.cfg.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost, action_cx: ActionCx, up: PointerUpCx| {
                let update = handle_pointer_up_in_scope(
                    host.models_mut(),
                    &svc,
                    action_cx.window,
                    frame_id,
                    cfg.kind,
                    cfg.scope,
                    up.pointer_id,
                    up.position,
                    up.tick_id,
                    cfg.resolved_activation_constraint(up.pointer_type),
                    cfg.collision_strategy,
                    cfg.autoscroll,
                );

                if cfg.capture_pointer_on_down {
                    host.release_pointer_capture();
                }

                if let Some(model) = cfg.update_model.as_ref() {
                    let update = update.clone();
                    let _ = host.models_mut().update(model, |v| *v = update);
                }

                if let Some(cb) = cfg.on_update.as_ref() {
                    cb(host, action_cx, &update);
                }

                cfg.consume_events
            },
        )
    }

    pub fn on_pointer_cancel(&self) -> OnPointerCancel {
        let svc = self.svc.clone();
        let frame_id = self.frame_id;
        let cfg = self.cfg.clone();
        Arc::new(
            move |host: &mut dyn UiPointerActionHost,
                  action_cx: ActionCx,
                  cancel: PointerCancelCx| {
                let position: Point = cancel.position.unwrap_or_else(|| host.bounds().origin);
                let update = handle_pointer_cancel_in_scope(
                    host.models_mut(),
                    &svc,
                    action_cx.window,
                    frame_id,
                    cfg.kind,
                    cfg.scope,
                    cancel.pointer_id,
                    position,
                    cancel.tick_id,
                    cfg.resolved_activation_constraint(cancel.pointer_type),
                    cfg.collision_strategy,
                    cfg.autoscroll,
                );

                if cfg.capture_pointer_on_down {
                    host.release_pointer_capture();
                }

                if let Some(model) = cfg.update_model.as_ref() {
                    let update = update.clone();
                    let _ = host.models_mut().update(model, |v| *v = update);
                }

                if let Some(cb) = cfg.on_update.as_ref() {
                    cb(host, action_cx, &update);
                }

                cfg.consume_events
            },
        )
    }
}

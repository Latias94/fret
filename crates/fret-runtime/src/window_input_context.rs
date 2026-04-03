use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::{GlobalsHost, InputContext, WindowCommandAvailabilityService};

/// Window-scoped `InputContext` snapshots published by the UI runtime.
///
/// This is a data-only integration seam that allows runner/platform layers (e.g. OS menu bars) to
/// access focus/modal state without depending on `fret-ui` internals.
#[derive(Debug, Default)]
pub struct WindowInputContextService {
    by_window: HashMap<AppWindowId, InputContext>,
}

impl WindowInputContextService {
    pub fn window_count(&self) -> usize {
        self.by_window.len()
    }

    pub fn snapshot(&self, window: AppWindowId) -> Option<&InputContext> {
        self.by_window.get(&window)
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, input_ctx: InputContext) {
        self.by_window.insert(window, input_ctx);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}

fn apply_window_command_availability(
    app: &impl GlobalsHost,
    window: AppWindowId,
    mut input_ctx: InputContext,
) -> InputContext {
    if let Some(availability) = app
        .global::<WindowCommandAvailabilityService>()
        .and_then(|svc| svc.snapshot(window))
        .copied()
    {
        input_ctx.edit_can_undo = availability.edit_can_undo;
        input_ctx.edit_can_redo = availability.edit_can_redo;
        input_ctx.router_can_back = availability.router_can_back;
        input_ctx.router_can_forward = availability.router_can_forward;
    }
    input_ctx
}

/// Best-effort: returns the published window input context snapshot, correcting command
/// availability booleans from the authoritative `WindowCommandAvailabilityService` when present.
pub fn best_effort_input_context_for_window(
    app: &impl GlobalsHost,
    window: AppWindowId,
) -> Option<InputContext> {
    app.global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .map(|input_ctx| apply_window_command_availability(app, window, input_ctx))
}

/// Best-effort: returns the published window input context snapshot if present, otherwise falls
/// back to `fallback_input_ctx`, correcting command-availability booleans from the authoritative
/// `WindowCommandAvailabilityService` when present.
pub fn best_effort_input_context_for_window_with_fallback(
    app: &impl GlobalsHost,
    window: AppWindowId,
    fallback_input_ctx: InputContext,
) -> InputContext {
    let input_ctx = app
        .global::<WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .unwrap_or(fallback_input_ctx);
    apply_window_command_availability(app, window, input_ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    #[derive(Default)]
    struct TestGlobalsHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
    }

    impl GlobalsHost for TestGlobalsHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|value| value.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value = match self.globals.remove(&type_id) {
                Some(value) => *value
                    .downcast::<T>()
                    .expect("TestGlobalsHost stored wrong type"),
                None => init(),
            };

            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    #[test]
    fn best_effort_input_context_overlays_authoritative_command_availability() {
        let mut host = TestGlobalsHost::default();
        let window = AppWindowId::default();

        host.with_global_mut(WindowInputContextService::default, |svc, _host| {
            svc.set_snapshot(
                window,
                InputContext {
                    edit_can_undo: false,
                    edit_can_redo: false,
                    router_can_back: false,
                    router_can_forward: false,
                    ..Default::default()
                },
            );
        });
        host.with_global_mut(WindowCommandAvailabilityService::default, |svc, _host| {
            svc.set_router_availability(window, true, false);
            svc.set_edit_availability(window, true, false);
        });

        let input_ctx =
            best_effort_input_context_for_window(&host, window).expect("published input context");
        assert!(input_ctx.edit_can_undo);
        assert!(!input_ctx.edit_can_redo);
        assert!(input_ctx.router_can_back);
        assert!(!input_ctx.router_can_forward);
    }

    #[test]
    fn best_effort_input_context_fallback_inherits_command_availability() {
        let mut host = TestGlobalsHost::default();
        let window = AppWindowId::default();

        host.with_global_mut(WindowCommandAvailabilityService::default, |svc, _host| {
            svc.set_router_availability(window, true, true);
            svc.set_edit_availability(window, false, true);
        });

        let input_ctx = best_effort_input_context_for_window_with_fallback(
            &host,
            window,
            InputContext::default(),
        );
        assert!(!input_ctx.edit_can_undo);
        assert!(input_ctx.edit_can_redo);
        assert!(input_ctx.router_can_back);
        assert!(input_ctx.router_can_forward);
    }
}

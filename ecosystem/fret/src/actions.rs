//! Typed action authoring sugar for the `fret` golden path.
//!
//! v1 constraints (see ADR 0307):
//! - `ActionId` is compatible with existing `CommandId` strings (no keymap schema changes).
//! - typed actions are unit marker types (no payload).
//! - action metadata uses the existing command registry surface (`CommandRegistry` + `CommandMeta`).
//!
//! v2 note (ADR 0312):
//! - payload/parameterized actions are supported as an additive, best-effort mechanism via a
//!   transient pending payload store keyed by `(window, ActionId)`.

pub use fret_runtime::{ActionId, ActionMeta, ActionRegistry, CommandId, TypedAction};
pub use fret_ui_kit::command::ElementCommandGatingExt;

use std::any::Any;

type OnAction = std::sync::Arc<
    dyn Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
>;

type OnPayloadAction = std::sync::Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiFocusActionHost,
            fret_ui::action::ActionCx,
            Box<dyn Any + Send + Sync>,
        ) -> bool
        + 'static,
>;

type OnActionAvailability = std::sync::Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
            fret_ui::action::CommandAvailabilityActionCx,
        ) -> fret_ui::CommandAvailability
        + 'static,
>;

/// Typed payload action marker type (v2 prototype).
///
/// Payload actions are pointer/programmatic-only in v2:
/// - keymap/palette/menus remain unit-action surfaces (no payload schema changes),
/// - payload is stored transiently via the UI host (ADR 0312).
pub trait TypedPayloadAction: TypedAction {
    type Payload: Any + Send + Sync + 'static;
}

/// Minimal handler table that dispatches stable [`ActionId`]s through the existing command hooks.
///
/// v1 note: `ActionId` is `CommandId`-compatible, so this is implemented as a thin adapter over
/// `OnCommand` / `OnCommandAvailability`.
#[derive(Default)]
pub(crate) struct ActionHandlerTable {
    on_action: std::collections::HashMap<ActionId, OnAction>,
    on_payload_action: std::collections::HashMap<ActionId, OnPayloadAction>,
    on_action_availability: std::collections::HashMap<ActionId, OnActionAvailability>,
}

impl ActionHandlerTable {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn on<A: TypedAction>(
        mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) -> Self {
        self.on_action
            .insert(A::action_id(), std::sync::Arc::new(f));
        self
    }

    pub(crate) fn on_payload<A: TypedPayloadAction>(
        mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiFocusActionHost,
            fret_ui::action::ActionCx,
            A::Payload,
        ) -> bool
        + 'static,
    ) -> Self {
        let action = A::action_id();
        self.on_payload_action.insert(
            action.clone(),
            std::sync::Arc::new(move |host, acx, payload_any| {
                let Ok(payload) = payload_any.downcast::<A::Payload>() else {
                    return false;
                };
                f(host, acx, *payload)
            }),
        );
        self
    }

    pub(crate) fn availability<A: TypedAction>(
        mut self,
        f: impl Fn(
            &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
            fret_ui::action::CommandAvailabilityActionCx,
        ) -> fret_ui::CommandAvailability
        + 'static,
    ) -> Self {
        self.on_action_availability
            .insert(A::action_id(), std::sync::Arc::new(f));
        self
    }

    pub(crate) fn build(
        self,
    ) -> (
        fret_ui::action::OnCommand,
        fret_ui::action::OnCommandAvailability,
    ) {
        let on_action = self.on_action;
        let on_payload_action = self.on_payload_action;
        let on_action_availability = self.on_action_availability;

        let on_command: fret_ui::action::OnCommand =
            std::sync::Arc::new(move |host, acx, command| {
                if let Some(payload_handler) = on_payload_action.get(&command) {
                    let Some(payload) = host.consume_pending_action_payload(acx.window, &command)
                    else {
                        return false;
                    };
                    return payload_handler(host, acx, payload);
                }
                let Some(handler) = on_action.get(&command) else {
                    return false;
                };
                handler(host, acx)
            });

        let on_command_availability: fret_ui::action::OnCommandAvailability =
            std::sync::Arc::new(move |host, acx, command| {
                let Some(handler) = on_action_availability.get(&command) else {
                    return fret_ui::CommandAvailability::NotHandled;
                };
                handler(host, acx)
            });

        (on_command, on_command_availability)
    }
}

/// Define typed unit actions with stable IDs.
///
/// This macro intentionally requires explicit action ID strings in v1 to keep the mapping
/// predictable for diagnostics and future data-driven frontends.
///
/// Example:
///
/// ```rust,ignore
/// mod act {
///     fret::actions!(fret, [
///         EditorSave = "app.editor.save.v1",
///         WorkspaceTabClose = "workspace.tabs.close.v1",
///     ]);
/// }
///
/// // Works anywhere a `CommandId`/`ActionId` is expected:
/// shadcn::Button::new("Save").action(act::EditorSave);
/// ```
#[macro_export]
macro_rules! actions {
    ([ $( $name:ident = $id:literal ),+ $(,)? ]) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;

            impl $name {
                pub const ID_STR: &'static str = $id;
            }

            impl $crate::TypedAction for $name {
                fn action_id() -> $crate::ActionId {
                    static ID: ::std::sync::OnceLock<$crate::ActionId> = ::std::sync::OnceLock::new();
                    ID.get_or_init(|| $crate::ActionId::from(Self::ID_STR)).clone()
                }
            }

            impl ::core::convert::From<$name> for $crate::CommandId {
                fn from(_: $name) -> $crate::CommandId {
                    <$name as $crate::TypedAction>::action_id()
                }
            }
        )+
    };

    ($fret:ident, [ $( $name:ident = $id:literal ),+ $(,)? ]) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;

            impl $name {
                pub const ID_STR: &'static str = $id;
            }

            impl $fret::TypedAction for $name {
                fn action_id() -> $fret::ActionId {
                    static ID: ::std::sync::OnceLock<$fret::ActionId> = ::std::sync::OnceLock::new();
                    ID.get_or_init(|| $fret::ActionId::from(Self::ID_STR)).clone()
                }
            }

            impl ::core::convert::From<$name> for $fret::CommandId {
                fn from(_: $name) -> $fret::CommandId {
                    <$name as $fret::TypedAction>::action_id()
                }
            }
        )+
    };
}

/// Define typed payload actions with stable IDs (v2 prototype).
///
/// This macro is intentionally additive: it does not change `actions!` and keeps explicit stable
/// IDs to avoid refactor drift.
///
/// Example:
///
/// ```rust,ignore
/// mod act {
///     fret::payload_actions!([
///         TodoRemove(u64) = "todo.remove.v2",
///     ]);
/// }
///
/// // Bind in UI:
/// shadcn::Button::new("Remove").action(act::TodoRemove).into_element(cx);
///
/// // Provide payload before dispatch:
/// host.record_pending_action_payload(acx, &act::TodoRemove::action_id(), Box::new(42u64));
/// ```
#[macro_export]
macro_rules! payload_actions {
    ([ $( $name:ident ( $payload:ty ) = $id:literal ),+ $(,)? ]) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;

            impl $name {
                pub const ID_STR: &'static str = $id;
            }

            impl $crate::TypedAction for $name {
                fn action_id() -> $crate::ActionId {
                    static ID: ::std::sync::OnceLock<$crate::ActionId> = ::std::sync::OnceLock::new();
                    ID.get_or_init(|| $crate::ActionId::from(Self::ID_STR)).clone()
                }
            }

            impl $crate::actions::TypedPayloadAction for $name {
                type Payload = $payload;
            }

            impl ::core::convert::From<$name> for $crate::CommandId {
                fn from(_: $name) -> $crate::CommandId {
                    <$name as $crate::TypedAction>::action_id()
                }
            }
        )+
    };

    ($fret:ident, [ $( $name:ident ( $payload:ty ) = $id:literal ),+ $(,)? ]) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct $name;

            impl $name {
                pub const ID_STR: &'static str = $id;
            }

            impl $fret::TypedAction for $name {
                fn action_id() -> $fret::ActionId {
                    static ID: ::std::sync::OnceLock<$fret::ActionId> = ::std::sync::OnceLock::new();
                    ID.get_or_init(|| $fret::ActionId::from(Self::ID_STR)).clone()
                }
            }

            impl $fret::actions::TypedPayloadAction for $name {
                type Payload = $payload;
            }

            impl ::core::convert::From<$name> for $fret::CommandId {
                fn from(_: $name) -> $fret::CommandId {
                    <$name as $fret::TypedAction>::action_id()
                }
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::{Arc, Mutex};

    use fret_core::AppWindowId;
    use fret_runtime::{Effect, ModelStore, TimerToken};
    use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};

    mod act {
        crate::actions!([
            EditorSave = "app.editor.save.v1",
            WorkspaceTabClose = "workspace.tabs.close.v1",
        ]);

        crate::payload_actions!([TodoRemove(u64) = "todo.remove.v2"]);
    }

    #[test]
    fn typed_actions_convert_to_command_id() {
        let save: crate::CommandId = act::EditorSave.into();
        assert_eq!(save.as_str(), "app.editor.save.v1");

        let close: crate::CommandId = act::WorkspaceTabClose.into();
        assert_eq!(close.as_str(), "workspace.tabs.close.v1");

        let remove: crate::CommandId = act::TodoRemove.into();
        assert_eq!(remove.as_str(), "todo.remove.v2");
    }

    #[test]
    fn payload_actions_consume_pending_payload_when_dispatched() {
        #[derive(Default)]
        struct FakeHost {
            models: ModelStore,
            next_timer: u64,
            payloads: Vec<(AppWindowId, crate::ActionId, Box<dyn Any + Send + Sync>)>,
        }

        impl UiActionHost for FakeHost {
            fn models_mut(&mut self) -> &mut ModelStore {
                &mut self.models
            }

            fn push_effect(&mut self, _effect: Effect) {}

            fn request_redraw(&mut self, _window: AppWindowId) {}

            fn next_timer_token(&mut self) -> TimerToken {
                let id = self.next_timer;
                self.next_timer = self.next_timer.saturating_add(1);
                TimerToken(id)
            }

            fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
                fret_runtime::ClipboardToken::default()
            }

            fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
                fret_runtime::ShareSheetToken::default()
            }

            fn record_pending_action_payload(
                &mut self,
                cx: ActionCx,
                action: &crate::ActionId,
                payload: Box<dyn Any + Send + Sync>,
            ) {
                self.payloads.push((cx.window, action.clone(), payload));
            }

            fn consume_pending_action_payload(
                &mut self,
                window: AppWindowId,
                action: &crate::ActionId,
            ) -> Option<Box<dyn Any + Send + Sync>> {
                let pos = self
                    .payloads
                    .iter()
                    .rposition(|(w, a, _)| *w == window && a == action)?;
                Some(self.payloads.remove(pos).2)
            }
        }

        impl UiFocusActionHost for FakeHost {
            fn request_focus(&mut self, _target: fret_ui::GlobalElementId) {}
        }

        let called_with = Arc::new(Mutex::new(Vec::<u64>::new()));
        let called_with_for_handler = called_with.clone();

        let handlers = crate::actions::ActionHandlerTable::new().on_payload::<act::TodoRemove>(
            move |_host, _acx, id| {
                called_with_for_handler.lock().unwrap().push(id);
                true
            },
        );

        let (on_command, _on_avail) = handlers.build();

        let mut host = FakeHost::default();
        let window = AppWindowId::default();
        let acx = ActionCx {
            window,
            target: fret_ui::GlobalElementId(1),
        };
        let action_id = <act::TodoRemove as crate::TypedAction>::action_id();

        host.record_pending_action_payload(acx, &action_id, Box::new(42u64));
        let handled = on_command(&mut host, acx, action_id.clone());

        assert!(handled);
        assert_eq!(&*called_with.lock().unwrap(), &[42u64]);
    }
}

//! Typed action authoring sugar for the `fret` golden path.
//!
//! v1 constraints (see ADR 0307):
//! - `ActionId` is compatible with existing `CommandId` strings (no keymap schema changes).
//! - typed actions are unit marker types (no payload).
//! - action metadata uses the existing command registry surface (`CommandRegistry` + `CommandMeta`).

pub use fret_runtime::{ActionId, ActionMeta, ActionRegistry, TypedAction};

pub type OnAction = std::sync::Arc<
    dyn Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
>;

pub type OnActionAvailability = std::sync::Arc<
    dyn Fn(
            &mut dyn fret_ui::action::UiCommandAvailabilityActionHost,
            fret_ui::action::CommandAvailabilityActionCx,
        ) -> fret_ui::CommandAvailability
        + 'static,
>;

/// Typed action marker type that also provides metadata for command palette / menus (v1).
///
/// v1 strategy (ADR 0307): this metadata is the existing command metadata surface.
pub trait TypedActionMeta: TypedAction {
    fn meta() -> ActionMeta;
}

pub trait ActionRegistryExt {
    fn register_typed_action<A: TypedActionMeta>(&mut self);
}

impl ActionRegistryExt for fret_runtime::CommandRegistry {
    fn register_typed_action<A: TypedActionMeta>(&mut self) {
        self.register(A::action_id(), A::meta());
    }
}

/// Minimal handler table that dispatches stable [`ActionId`]s through the existing command hooks.
///
/// v1 note: `ActionId` is `CommandId`-compatible, so this is implemented as a thin adapter over
/// `OnCommand` / `OnCommandAvailability`.
#[derive(Default)]
pub struct ActionHandlerTable {
    on_action: std::collections::HashMap<ActionId, OnAction>,
    on_action_availability: std::collections::HashMap<ActionId, OnActionAvailability>,
}

impl ActionHandlerTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on<A: TypedAction>(
        mut self,
        f: impl Fn(&mut dyn fret_ui::action::UiFocusActionHost, fret_ui::action::ActionCx) -> bool
        + 'static,
    ) -> Self {
        self.on_action
            .insert(A::action_id(), std::sync::Arc::new(f));
        self
    }

    pub fn availability<A: TypedAction>(
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

    pub fn build(
        self,
    ) -> (
        fret_ui::action::OnCommand,
        fret_ui::action::OnCommandAvailability,
    ) {
        let on_action = self.on_action;
        let on_action_availability = self.on_action_availability;

        let on_command: fret_ui::action::OnCommand =
            std::sync::Arc::new(move |host, acx, command| {
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

#[cfg(test)]
mod tests {
    mod act {
        crate::actions!([
            EditorSave = "app.editor.save.v1",
            WorkspaceTabClose = "workspace.tabs.close.v1",
        ]);
    }

    #[test]
    fn typed_actions_convert_to_command_id() {
        let save: crate::CommandId = act::EditorSave.into();
        assert_eq!(save.as_str(), "app.editor.save.v1");

        let close: crate::CommandId = act::WorkspaceTabClose.into();
        assert_eq!(close.as_str(), "workspace.tabs.close.v1");
    }
}

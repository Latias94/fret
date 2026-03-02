//! Typed action authoring sugar for the `fret` golden path.
//!
//! v1 constraints (see ADR 0307):
//! - `ActionId` is compatible with existing `CommandId` strings (no keymap schema changes).
//! - typed actions are unit marker types (no payload).

pub use fret_runtime::{ActionId, TypedAction};

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

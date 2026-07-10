use crate::CommandId;

/// Canonical action identifier.
///
/// Per ADR 0307, action identity shares the command identity used by keymaps, menus, and routing.
pub type ActionId = CommandId;

/// Typed unit action marker type.
///
/// This trait is intentionally minimal: it maps a Rust marker type to a stable [`ActionId`].
/// Structured payloads remain an explicit higher-level concern.
pub trait TypedAction: 'static {
    fn action_id() -> ActionId;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommandMeta, CommandRegistry};

    struct Save;

    impl TypedAction for Save {
        fn action_id() -> ActionId {
            ActionId::from("tests.action.save")
        }
    }

    #[test]
    fn action_identity_uses_the_command_registry_contract() {
        let action_id = Save::action_id();
        let command_id: CommandId = action_id.clone();
        let mut commands = CommandRegistry::default();

        commands.register(action_id, CommandMeta::new("Save"));

        assert_eq!(Save::action_id(), command_id);
        assert_eq!(
            commands.get(command_id).map(|meta| meta.title.as_ref()),
            Some("Save")
        );
    }
}

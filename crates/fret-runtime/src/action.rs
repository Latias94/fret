use crate::{CommandId, CommandMeta, CommandRegistry};

/// Stable action identifier (v1).
///
/// v1 compatibility strategy (ADR 0307): `ActionId` is an alias over `CommandId` so we can adopt
/// action-first authoring without keymap schema churn.
pub type ActionId = CommandId;

/// Action metadata (v1).
///
/// v1 strategy (ADR 0307): action metadata is the existing command metadata surface.
pub type ActionMeta = CommandMeta;

/// Action registry (v1).
///
/// v1 strategy (ADR 0307): action metadata is stored in the existing command registry.
pub type ActionRegistry = CommandRegistry;

/// Typed unit action marker type (v1).
///
/// This trait is intentionally minimal: it maps a Rust marker type to a stable [`ActionId`].
/// v1 standardizes on unit actions only (no structured payload).
pub trait TypedAction: 'static {
    fn action_id() -> ActionId;
}

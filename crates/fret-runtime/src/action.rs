use crate::CommandId;

/// Stable action identifier (v1).
///
/// v1 compatibility strategy (ADR 0307): `ActionId` is an alias over `CommandId` so we can adopt
/// action-first authoring without keymap schema churn.
pub type ActionId = CommandId;

/// Typed unit action marker type (v1).
///
/// This trait is intentionally minimal: it maps a Rust marker type to a stable [`ActionId`].
/// v1 standardizes on unit actions only (no structured payload).
pub trait TypedAction: 'static {
    fn action_id() -> ActionId;
}

//! Explicit raw retained-tree and hook seams for advanced/manual assembly.

/// Raw action-registration hooks for host-owned integrations.
pub use crate::view::AppUiRawActionNotifyExt;
/// Raw-model hooks for intentional `Model<T>`-centric code or manual model-store owners.
pub use crate::view::{
    AppUiRawModelExt, LocalStateElementContextExt, LocalStateModelStoreExt, LocalStateRawModelExt,
};
/// Raw `Model<T>` handle and store types for manual model-store integrations.
pub use fret_runtime::{Model, ModelStore, ModelUpdateError};
/// Raw retained-tree host for manual runtime integrations.
pub use fret_ui::UiTree;
/// Handle-first tracked-read helpers for raw `Model<T>` values on manual `ElementContext` surfaces.
pub use fret_ui_kit::declarative::TrackedModelExt;

/// Insert a `LocalState<T>` into a raw `ModelStore` for manual/hybrid surfaces.
///
/// Default app code should use `AppLocalStateExt::local_state(...)` during `View::init` or
/// `cx.state().local*` during render. This helper exists for code that intentionally owns the raw
/// model store.
#[track_caller]
pub fn local_state_in<T>(
    models: &mut fret_runtime::ModelStore,
    value: T,
) -> crate::view::LocalState<T>
where
    T: std::any::Any,
{
    crate::view::LocalState::new_in(models, value)
}

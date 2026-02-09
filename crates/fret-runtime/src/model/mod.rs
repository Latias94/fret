mod debug;
mod error;
mod handle;
mod host;
mod store;

slotmap::new_key_type! {
    pub struct ModelId;
}

pub use debug::{ModelChangedDebugInfo, ModelCreatedDebugInfo};
pub use error::ModelUpdateError;
pub use handle::{Model, WeakModel};
pub use host::{ModelCx, ModelHost};
pub use store::{ModelLease, ModelStore};

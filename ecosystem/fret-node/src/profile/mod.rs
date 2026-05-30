//! Compatibility re-exports for Jellyflow runtime profiles.

pub use jellyflow_runtime::profile::*;

#[cfg(feature = "kit")]
pub use crate::kit::profiles::DataflowProfile;

//! Headless runtime, rules, schema, and profile pipeline for Jellyflow.
//!
//! This crate owns the portable B-layer store and change pipeline built on top of
//! `jellyflow-core` graph transactions. It must stay free of Fret UI, renderer, platform, and
//! windowing dependencies.

#![deny(unsafe_code)]

pub mod core {
    pub use jellyflow_core::core::*;
}

pub mod interaction {
    pub use jellyflow_core::interaction::*;
}

pub mod ops {
    pub use jellyflow_core::ops::*;
}

pub mod types {
    pub use jellyflow_core::types::*;
}

pub mod io;
pub mod profile;
pub mod rules;
pub mod runtime;
pub mod schema;

pub use profile::{
    ApplyPipelineError, GraphProfile, apply_connect_plan_with_profile,
    apply_transaction_with_profile,
};
pub use runtime::store::{DispatchError, DispatchOutcome, NodeGraphStore};

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_stays_free_of_ui_renderer_and_platform_dependencies() {
        let manifest = include_str!("../Cargo.toml");
        for forbidden in ["fret-ui", "fret-runtime", "fret-canvas", "wgpu", "winit"] {
            assert!(
                !manifest.contains(forbidden),
                "jellyflow-runtime must stay headless; forbidden dependency `{forbidden}` found",
            );
        }
    }
}

//! Convenience layer built on top of the node-graph substrate.
//!
//! This module provides common profiles and recipes that are useful for quickly building
//! editor-grade node graphs (dataflow/workflow prototypes) on top of the core substrate.
//!
//! Domain graphs (ShaderGraph/Blueprint/etc.) are expected to provide their own schema + profile
//! specializations and may choose to reuse parts of this kit.

pub mod nodes;
pub mod profiles;

pub use profiles::DataflowProfile;

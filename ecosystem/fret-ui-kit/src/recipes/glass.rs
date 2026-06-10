//! Glass recipe helpers built on top of shared theme/style resolution.
//!
//! This module intentionally lives in `recipes`, not in `declarative` itself:
//! - chrome token resolution stays here,
//! - effect-chain resolution stays here,
//! - and app-facing wrappers can compose the resolved values without owning fallback policy.

mod chrome;
mod effect;

pub use chrome::{GlassTokenKeys, ResolvedGlassChrome, resolve_glass_chrome};
pub use effect::{
    GlassEffectRefinement, GlassEffectTokenKeys, ResolvedGlassEffect, glass_effect_chain,
    glass_effect_chain_for_environment, resolve_glass_effect,
    resolve_glass_effect_chain_for_environment,
};

#[cfg(test)]
mod tests;

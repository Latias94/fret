//! Pixelate recipe helpers built on top of shared theme/style resolution.
//!
//! This module intentionally lives in `recipes`, not in `declarative` itself:
//! - chrome token resolution stays here,
//! - effect-chain resolution stays here,
//! - and app-facing wrappers can compose the resolved values without owning fallback policy.

mod chrome;
mod effect;

pub use chrome::{PixelateTokenKeys, ResolvedPixelateChrome, resolve_pixelate_chrome};
pub use effect::{
    PixelateEffectRefinement, PixelateEffectTokenKeys, ResolvedPixelateEffect,
    pixelate_effect_chain, resolve_pixelate_effect,
};

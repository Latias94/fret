//! Bloom recipe helpers built on top of shared effect-chain resolution.
//!
//! This module owns bloom effect policy while declarative wrappers own layout
//! composition and chrome composition.

mod effect;

pub use effect::{BloomEffect, bloom_effect_chain};

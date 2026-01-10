//! B-layer runtime building blocks (XyFlow-style change pipeline).
//!
//! This module is intentionally **headless-safe**: it must not depend on `fret-ui`.
//! The goal is to provide a stable, ergonomic "runtime/store" surface without coupling to
//! a specific rendering or widget layer.

pub mod apply;
pub mod changes;
pub mod events;
pub mod store;

#[cfg(test)]
mod tests;

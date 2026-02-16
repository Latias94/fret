//! Asset cache helpers for Fret ecosystem components.
//!
//! This crate provides small caching layers for common UI assets (images, SVGs) so components can
//! avoid repeated decode/raster/upload work across frames.
//!
//! This is an ecosystem crate: it composes higher-level policies on top of the core runtime
//! services.

pub mod image_asset_cache;
pub mod image_upload;
pub mod svg_asset_cache;

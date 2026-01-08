//! Fret chart UI adapter for the headless `delinea` engine.
//!
//! This crate focuses on:
//! - translating `delinea` marks into Fret draw ops (`SceneOp`)
//! - mapping UI input into `delinea` actions/patches (planned)

pub mod input_map;
pub mod retained;

pub use input_map::*;
pub use retained::*;

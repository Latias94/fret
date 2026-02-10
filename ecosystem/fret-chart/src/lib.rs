//! Fret chart UI adapter for the headless `delinea` engine.
//!
//! This crate focuses on:
//! - translating `delinea` marks into Fret draw ops (`SceneOp`)
//! - mapping UI input into `delinea` actions/patches

pub mod declarative;
#[cfg(feature = "echarts")]
pub mod echarts;
pub mod input_map;
pub mod linking;
pub mod retained;

mod legend_logic;
mod tooltip_layout;

pub use declarative::*;
pub use input_map::*;
pub use linking::*;
pub use retained::*;

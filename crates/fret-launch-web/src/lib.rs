//! Web/wasm32 launcher implementation for Fret.
//!
//! This crate contains the browser/WebGPU runner and related wiring.
//! Most application code should depend on the `fret-launch` facade instead.

pub mod runner;

pub use fret_launch_core::RunnerError;

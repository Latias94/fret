//! Headless motion primitives (time-driven).
//!
//! This module provides reusable motion building blocks that can be shared across component
//! ecosystems without depending on runner scheduling or theme policy.
//!
//! Design goals:
//!
//! - Use `Duration` (wall-time) as the canonical time unit (refresh-rate independent).
//! - Keep the math portable and deterministic (given deterministic `delta` stepping).
//! - Support both "web-like" tweens (duration + easing) and "native-like" physics (spring/inertia).

pub mod friction;
pub mod simulation;
pub mod spring;
pub mod tolerance;
pub mod tween;

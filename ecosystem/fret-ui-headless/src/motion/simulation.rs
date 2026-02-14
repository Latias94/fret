//! Simulation traits and common sample shapes.

use std::time::Duration;

use super::tolerance::Tolerance;

/// A 1D motion sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSample {
    pub value: f64,
    pub velocity: f64,
    pub done: bool,
}

/// A 1D simulation that can be evaluated at any monotonic time.
///
/// Units are intentionally unspecified; callers should establish a convention and use it
/// consistently. By convention, time is in seconds (via `Duration`) and velocity is units/second.
pub trait Simulation1D {
    fn x(&self, time: Duration) -> f64;
    fn dx(&self, time: Duration) -> f64;
    fn is_done(&self, time: Duration) -> bool;

    fn sample(&self, time: Duration) -> MotionSample {
        let value = self.x(time);
        let velocity = self.dx(time);
        let done = self.is_done(time);
        MotionSample {
            value,
            velocity,
            done,
        }
    }
}

pub(crate) fn near_zero(x: f64, eps: f64) -> bool {
    x.abs() < eps
}

pub(crate) fn secs(time: Duration) -> f64 {
    time.as_secs_f64()
}

pub(crate) fn default_done(tolerance: Tolerance, displacement: f64, velocity: f64) -> bool {
    near_zero(displacement, tolerance.distance) && near_zero(velocity, tolerance.velocity)
}

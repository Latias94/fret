//! Tolerances for motion simulations.

/// Maximum allowable magnitudes for distances, durations, and velocity differences to be
/// considered equal.
///
/// This mirrors Flutter's `Tolerance` shape and is intended to be used by physics simulations to
/// decide when they are effectively at rest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerance {
    /// The magnitude of the maximum distance between two points for them to be considered within
    /// tolerance.
    pub distance: f64,
    /// The magnitude of the maximum duration between two times for them to be considered within
    /// tolerance.
    pub time: f64,
    /// The magnitude of the maximum difference between two velocities for them to be considered
    /// within tolerance.
    pub velocity: f64,
}

impl Tolerance {
    const DEFAULT_EPSILON: f64 = 1e-3;

    pub const DEFAULT: Self = Self {
        distance: Self::DEFAULT_EPSILON,
        time: Self::DEFAULT_EPSILON,
        velocity: Self::DEFAULT_EPSILON,
    };
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::DEFAULT
    }
}

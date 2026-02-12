//! Friction/decay simulations (inertial slowdown).

use std::time::Duration;

use super::simulation::{Simulation1D, secs};
use super::tolerance::Tolerance;

/// A simulation that applies a drag to slow a particle down.
///
/// This mirrors Flutter's `FrictionSimulation` shape. The `drag` coefficient is unitless.
#[derive(Debug, Clone, Copy)]
pub struct FrictionSimulation {
    drag: f64,
    drag_log: f64,
    x0: f64,
    v0: f64,
    constant_deceleration: f64,
    final_time: f64,
    tolerance: Tolerance,
}

impl FrictionSimulation {
    pub fn new(
        drag: f64,
        position: f64,
        velocity: f64,
        constant_deceleration: f64,
        tolerance: Tolerance,
    ) -> Self {
        let drag = drag.max(f64::MIN_POSITIVE);
        let drag_log = drag.ln();
        let constant_deceleration = constant_deceleration.abs() * velocity.signum();

        // When `constant_deceleration` is non-zero, solve for a stop time where dx(t)=0.
        // This is a best-effort cap to avoid negative velocities past the stopping point.
        let final_time = if constant_deceleration == 0.0 {
            f64::INFINITY
        } else {
            // Solve v0*drag^t - c*t = 0 via Newton iterations.
            let mut guess = 0.0;
            for _ in 0..10 {
                let f = velocity * drag.powf(guess) - constant_deceleration * guess;
                let df = velocity * drag.powf(guess) * drag_log - constant_deceleration;
                if df.abs() < 1e-12 {
                    break;
                }
                guess = (guess - f / df).max(0.0);
            }
            guess
        };

        Self {
            drag,
            drag_log,
            x0: position,
            v0: velocity,
            constant_deceleration,
            final_time,
            tolerance,
        }
    }

    /// Construct a simulation that passes through the specified start/end positions and velocities.
    pub fn through(
        start_position: f64,
        end_position: f64,
        start_velocity: f64,
        end_velocity: f64,
    ) -> Self {
        assert!(
            start_velocity == 0.0
                || end_velocity == 0.0
                || start_velocity.signum() == end_velocity.signum()
        );
        assert!(start_velocity.abs() >= end_velocity.abs());
        assert!((end_position - start_position).signum() == start_velocity.signum());

        let drag = ((start_velocity - end_velocity) / (start_position - end_position)).exp();
        Self::new(
            drag,
            start_position,
            start_velocity,
            0.0,
            Tolerance {
                velocity: end_velocity.abs(),
                ..Tolerance::default()
            },
        )
    }

    pub fn final_x(&self) -> f64 {
        if self.constant_deceleration == 0.0 {
            self.x0 - self.v0 / self.drag_log
        } else {
            self.x(Duration::from_secs_f64(self.final_time))
        }
    }

    pub fn tolerance(&self) -> Tolerance {
        self.tolerance
    }
}

impl Simulation1D for FrictionSimulation {
    fn x(&self, time: Duration) -> f64 {
        let t = secs(time);
        let t = t.min(self.final_time);

        self.x0 + self.v0 * self.drag.powf(t) / self.drag_log
            - self.v0 / self.drag_log
            - (self.constant_deceleration * 0.5) * t * t
    }

    fn dx(&self, time: Duration) -> f64 {
        let t = secs(time);
        let t = t.min(self.final_time);
        self.v0 * self.drag.powf(t) - self.constant_deceleration * t
    }

    fn is_done(&self, time: Duration) -> bool {
        self.dx(time).abs() < self.tolerance.velocity
    }
}

/// A friction simulation that clamps the modeled particle to a specific range of values.
#[derive(Debug, Clone, Copy)]
pub struct BoundedFrictionSimulation {
    inner: FrictionSimulation,
    min_x: f64,
    max_x: f64,
}

impl BoundedFrictionSimulation {
    pub fn new(inner: FrictionSimulation, min_x: f64, max_x: f64) -> Self {
        assert!(max_x >= min_x);
        let start = inner.x(Duration::ZERO);
        assert!(start >= min_x && start <= max_x);
        Self {
            inner,
            min_x,
            max_x,
        }
    }
}

impl Simulation1D for BoundedFrictionSimulation {
    fn x(&self, time: Duration) -> f64 {
        self.inner.x(time).clamp(self.min_x, self.max_x)
    }

    fn dx(&self, time: Duration) -> f64 {
        self.inner.dx(time)
    }

    fn is_done(&self, time: Duration) -> bool {
        self.inner.is_done(time)
            || (self.x(time) - self.min_x).abs() < self.inner.tolerance().distance
            || (self.x(time) - self.max_x).abs() < self.inner.tolerance().distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friction_velocity_decays_towards_zero() {
        let sim = FrictionSimulation::new(0.135, 0.0, 100.0, 0.0, Tolerance::default());
        let v0 = sim.dx(Duration::ZERO);
        let v1 = sim.dx(Duration::from_millis(250));
        let v2 = sim.dx(Duration::from_secs(2));
        assert!(v1.abs() < v0.abs());
        assert!(v2.abs() < v1.abs());
    }
}

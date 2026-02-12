//! Analytic spring simulation (Hooke's law).

use std::time::Duration;

use super::simulation::{Simulation1D, default_done, secs};
use super::tolerance::Tolerance;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringDescription {
    pub mass: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl SpringDescription {
    pub fn with_damping_ratio(mass: f64, stiffness: f64, ratio: f64) -> Self {
        let ratio = ratio.max(0.0);
        let damping = ratio * 2.0 * (mass * stiffness).sqrt();
        Self {
            mass,
            stiffness,
            damping,
        }
    }

    /// Construct a spring from perceptual parameters.
    ///
    /// This mirrors Flutter's `SpringDescription.withDurationAndBounce` semantics:
    ///
    /// - `duration` controls the overall pace (approximately the settle time),
    /// - `bounce` controls overshoot:
    ///   - `0` is critically damped (no overshoot),
    ///   - `(0, 1)` is underdamped (some overshoot),
    ///   - negative values are overdamped.
    pub fn with_duration_and_bounce(duration: Duration, bounce: f64) -> Self {
        assert!(duration.as_nanos() > 0, "duration must be positive");
        assert!(
            bounce > -1.0,
            "bounce must be > -1.0 to keep damping derivation finite"
        );

        let duration_secs = duration.as_secs_f64();
        let mass = 1.0;
        let stiffness = (4.0 * std::f64::consts::PI * std::f64::consts::PI * mass)
            / (duration_secs * duration_secs);

        let damping_ratio = if bounce > 0.0 {
            1.0 - bounce
        } else {
            1.0 / (bounce + 1.0)
        };
        let damping = damping_ratio * 2.0 * (mass * stiffness).sqrt();

        Self {
            mass,
            stiffness,
            damping,
        }
    }

    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.mass * self.stiffness).sqrt())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpringType {
    CriticallyDamped,
    UnderDamped,
    OverDamped,
}

#[derive(Debug, Clone, Copy)]
struct SpringSolution {
    spring: SpringDescription,
    // Displacement from equilibrium: y = x - end.
    y0: f64,
    v0: f64,
}

impl SpringSolution {
    fn spring_type(&self) -> SpringType {
        let ratio = self.spring.damping_ratio();
        if (ratio - 1.0).abs() < 1e-9 {
            SpringType::CriticallyDamped
        } else if ratio < 1.0 {
            SpringType::UnderDamped
        } else {
            SpringType::OverDamped
        }
    }

    fn w0(&self) -> f64 {
        (self.spring.stiffness / self.spring.mass).sqrt()
    }

    fn y(&self, time: Duration) -> f64 {
        let t = secs(time);
        if t <= 0.0 {
            return self.y0;
        }

        let zeta = self.spring.damping_ratio();
        let w0 = self.w0();

        match self.spring_type() {
            SpringType::UnderDamped => {
                let wd = w0 * (1.0 - zeta * zeta).sqrt();
                let envelope = (-zeta * w0 * t).exp();
                let c1 = self.y0;
                let c2 = (self.v0 + zeta * w0 * self.y0) / wd;
                envelope * (c1 * (wd * t).cos() + c2 * (wd * t).sin())
            }
            SpringType::CriticallyDamped => {
                let envelope = (-w0 * t).exp();
                envelope * (self.y0 + (self.v0 + w0 * self.y0) * t)
            }
            SpringType::OverDamped => {
                let s = (zeta * zeta - 1.0).sqrt();
                let r1 = -w0 * (zeta - s);
                let r2 = -w0 * (zeta + s);
                let c1 = (self.v0 - r2 * self.y0) / (r1 - r2);
                let c2 = self.y0 - c1;
                c1 * (r1 * t).exp() + c2 * (r2 * t).exp()
            }
        }
    }

    fn dy(&self, time: Duration) -> f64 {
        let t = secs(time);
        if t <= 0.0 {
            return self.v0;
        }

        let zeta = self.spring.damping_ratio();
        let w0 = self.w0();

        match self.spring_type() {
            SpringType::UnderDamped => {
                let wd = w0 * (1.0 - zeta * zeta).sqrt();
                let envelope = (-zeta * w0 * t).exp();
                let c1 = self.y0;
                let c2 = (self.v0 + zeta * w0 * self.y0) / wd;
                let cos = (wd * t).cos();
                let sin = (wd * t).sin();

                let term = c1 * cos + c2 * sin;
                let term_d = -c1 * wd * sin + c2 * wd * cos;
                (-zeta * w0) * envelope * term + envelope * term_d
            }
            SpringType::CriticallyDamped => {
                let envelope = (-w0 * t).exp();
                let a = self.v0 + w0 * self.y0;
                (-w0) * envelope * (self.y0 + a * t) + envelope * a
            }
            SpringType::OverDamped => {
                let s = (zeta * zeta - 1.0).sqrt();
                let r1 = -w0 * (zeta - s);
                let r2 = -w0 * (zeta + s);
                let c1 = (self.v0 - r2 * self.y0) / (r1 - r2);
                let c2 = self.y0 - c1;
                c1 * r1 * (r1 * t).exp() + c2 * r2 * (r2 * t).exp()
            }
        }
    }
}

/// A spring simulation between `start` and `end`.
#[derive(Debug, Clone, Copy)]
pub struct SpringSimulation {
    end: f64,
    solution: SpringSolution,
    snap_to_end: bool,
    tolerance: Tolerance,
}

impl SpringSimulation {
    pub fn new(
        spring: SpringDescription,
        start: f64,
        end: f64,
        velocity: f64,
        snap_to_end: bool,
        tolerance: Tolerance,
    ) -> Self {
        Self {
            end,
            solution: SpringSolution {
                spring,
                y0: start - end,
                v0: velocity,
            },
            snap_to_end,
            tolerance,
        }
    }

    pub fn spring_type(&self) -> SpringType {
        self.solution.spring_type()
    }
}

impl Simulation1D for SpringSimulation {
    fn x(&self, time: Duration) -> f64 {
        if self.snap_to_end && self.is_done(time) {
            self.end
        } else {
            self.end + self.solution.y(time)
        }
    }

    fn dx(&self, time: Duration) -> f64 {
        if self.snap_to_end && self.is_done(time) {
            0.0
        } else {
            self.solution.dy(time)
        }
    }

    fn is_done(&self, time: Duration) -> bool {
        let y = self.solution.y(time);
        let v = self.solution.dy(time);
        default_done(self.tolerance, y, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_converges_to_end_position() {
        let spring = SpringDescription::with_damping_ratio(1.0, 500.0, 1.0);
        let sim = SpringSimulation::new(spring, 0.0, 1.0, 0.0, true, Tolerance::default());

        assert!((sim.x(Duration::ZERO) - 0.0).abs() < 1e-9);
        let late = sim.x(Duration::from_secs(10));
        assert!((late - 1.0).abs() < 1e-6);
        assert!(sim.is_done(Duration::from_secs(10)));
    }

    #[test]
    fn with_duration_and_bounce_produces_reasonable_parameters() {
        let spring = SpringDescription::with_duration_and_bounce(Duration::from_millis(300), 0.2);
        assert!(spring.mass > 0.0);
        assert!(spring.stiffness > 0.0);
        assert!(spring.damping > 0.0);
    }
}

//! Inertia (friction + optional edge bounce) simulation.
//!
//! This is intended for "kinetic" gestures (scroll, drawer/sheet release) where an initial
//! velocity decays over time, optionally bouncing at bounds.

use std::time::Duration;

use super::friction::FrictionSimulation;
use super::simulation::Simulation1D;
use super::spring::{SpringDescription, SpringSimulation};
use super::tolerance::Tolerance;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaBounds {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct InertiaSimulation {
    friction: FrictionSimulation,
    bounds: Option<InertiaBounds>,
    bounce_spring: SpringDescription,
    tolerance: Tolerance,
    hit: Option<Hit>,
}

#[derive(Debug, Clone, Copy)]
struct Hit {
    t_hit: Duration,
    bound: f64,
    v_hit: f64,
}

impl InertiaSimulation {
    pub fn new(
        position: f64,
        velocity: f64,
        drag: f64,
        bounds: Option<InertiaBounds>,
        bounce_spring: SpringDescription,
        tolerance: Tolerance,
    ) -> Self {
        let friction = FrictionSimulation::new(drag, position, velocity, 0.0, tolerance);
        let hit = bounds.and_then(|b| find_hit(&friction, b));
        Self {
            friction,
            bounds,
            bounce_spring,
            tolerance,
            hit,
        }
    }

    pub fn bounds(&self) -> Option<InertiaBounds> {
        self.bounds
    }

    pub fn final_x(&self) -> f64 {
        match self.hit {
            Some(hit) => {
                // The bounce phase settles at the bound.
                hit.bound
            }
            None => self.friction.final_x(),
        }
    }
}

fn find_hit(friction: &FrictionSimulation, bounds: InertiaBounds) -> Option<Hit> {
    debug_assert!(bounds.max >= bounds.min);

    let x0 = friction.x(Duration::ZERO);
    let v0 = friction.dx(Duration::ZERO);
    if v0 == 0.0 {
        return None;
    }

    let final_x = friction.final_x();
    let bound = if v0 > 0.0 && final_x > bounds.max {
        bounds.max
    } else if v0 < 0.0 && final_x < bounds.min {
        bounds.min
    } else {
        return None;
    };

    // The friction simulation uses the closed-form:
    //
    //   x(t) = x0 + v0 * (drag^t - 1) / ln(drag)
    //
    // Solve for t such that x(t) == bound:
    //
    //   drag^t = 1 + (bound - x0) * ln(drag) / v0
    //   t = ln( ... ) / ln(drag)
    //
    // Notes:
    // - This assumes `constant_deceleration == 0.0` (as constructed in `InertiaSimulation`).
    // - `drag` is in (0, 1) for typical decays, so ln(drag) < 0.
    let drag = friction.drag();
    let drag_log = friction.drag_log();
    if drag <= 0.0 || drag == 1.0 || !drag_log.is_finite() {
        return None;
    }

    let a = 1.0 + (bound - x0) * drag_log / v0;
    if !a.is_finite() || a <= 0.0 {
        return None;
    }

    let t = a.ln() / drag_log;
    if !t.is_finite() || t < 0.0 {
        return None;
    }

    let t_hit = Duration::from_secs_f64(t);
    let v_hit = friction.dx(t_hit);
    Some(Hit {
        t_hit,
        bound,
        v_hit,
    })
}

impl Simulation1D for InertiaSimulation {
    fn x(&self, time: Duration) -> f64 {
        match self.hit {
            Some(hit) if time > hit.t_hit => {
                let t = time.saturating_sub(hit.t_hit);
                let spring = SpringSimulation::new(
                    self.bounce_spring,
                    hit.bound,
                    hit.bound,
                    hit.v_hit,
                    true,
                    self.tolerance,
                );
                spring.x(t)
            }
            _ => self.friction.x(time),
        }
    }

    fn dx(&self, time: Duration) -> f64 {
        match self.hit {
            Some(hit) if time > hit.t_hit => {
                let t = time.saturating_sub(hit.t_hit);
                let spring = SpringSimulation::new(
                    self.bounce_spring,
                    hit.bound,
                    hit.bound,
                    hit.v_hit,
                    true,
                    self.tolerance,
                );
                spring.dx(t)
            }
            _ => self.friction.dx(time),
        }
    }

    fn is_done(&self, time: Duration) -> bool {
        match self.hit {
            Some(hit) if time > hit.t_hit => {
                let t = time.saturating_sub(hit.t_hit);
                let spring = SpringSimulation::new(
                    self.bounce_spring,
                    hit.bound,
                    hit.bound,
                    hit.v_hit,
                    true,
                    self.tolerance,
                );
                spring.is_done(t)
            }
            _ => self.friction.is_done(time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inertia_without_bounds_decays_to_final_x() {
        let sim = InertiaSimulation::new(
            0.0,
            1000.0,
            0.135,
            None,
            SpringDescription::with_duration_and_bounce(Duration::from_millis(300), 0.25),
            Tolerance::default(),
        );
        assert!((sim.x(Duration::ZERO) - 0.0).abs() < 1e-9);
        let late = sim.x(Duration::from_secs(10));
        assert!((late - sim.final_x()).abs() < 1e-3);
    }

    #[test]
    fn inertia_with_bounds_bounces_back_to_bound() {
        let bounds = InertiaBounds { min: 0.0, max: 1.0 };
        let sim = InertiaSimulation::new(
            0.5,
            5000.0,
            0.135,
            Some(bounds),
            SpringDescription::with_duration_and_bounce(Duration::from_millis(240), 0.25),
            Tolerance::default(),
        );

        // Early: moving.
        assert!(sim.dx(Duration::from_millis(16)) > 0.0);

        // Late: settled at the max bound.
        let late = sim.x(Duration::from_secs(10));
        assert!((late - bounds.max).abs() < 1e-3);
        assert!(sim.is_done(Duration::from_secs(10)));
    }
}

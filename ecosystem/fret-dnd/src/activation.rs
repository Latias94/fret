use std::collections::HashMap;

use fret_core::{Point, PointerId, Px};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ActivationConstraint {
    #[default]
    None,
    Distance {
        px: f32,
    },
    DelayTicks {
        ticks: u64,
    },
    DelayAndDistance {
        ticks: u64,
        px: f32,
    },
}

impl ActivationConstraint {
    pub fn is_satisfied(
        &self,
        start_tick: u64,
        current_tick: u64,
        start: Point,
        current: Point,
    ) -> bool {
        match *self {
            Self::None => true,
            Self::Distance { px } => distance_satisfies(start, current, px),
            Self::DelayTicks { ticks } => current_tick.saturating_sub(start_tick) >= ticks,
            Self::DelayAndDistance { ticks, px } => {
                current_tick.saturating_sub(start_tick) >= ticks
                    && distance_satisfies(start, current, px)
            }
        }
    }
}

fn distance_satisfies(start: Point, current: Point, px: f32) -> bool {
    let threshold = if px.is_finite() { px.max(0.0) } else { 0.0 };
    if threshold <= 0.0 {
        return true;
    }
    let dx = current.x.0 - start.x.0;
    let dy = current.y.0 - start.y.0;
    dx * dx + dy * dy >= threshold * threshold
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorEvent {
    Down {
        pointer_id: PointerId,
        position: Point,
        tick: u64,
    },
    Move {
        pointer_id: PointerId,
        position: Point,
        tick: u64,
    },
    Up {
        pointer_id: PointerId,
        position: Point,
        tick: u64,
    },
    Cancel {
        pointer_id: PointerId,
        position: Point,
        tick: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SensorOutput {
    Pending,
    DragStart {
        pointer_id: PointerId,
        start: Point,
        position: Point,
    },
    DragMove {
        pointer_id: PointerId,
        start: Point,
        position: Point,
        translation: Point,
    },
    DragEnd {
        pointer_id: PointerId,
        start: Point,
        position: Point,
    },
    DragCancel {
        pointer_id: PointerId,
        start: Point,
        position: Point,
    },
}

#[derive(Debug, Default, Clone)]
pub struct PointerSensor {
    constraint: ActivationConstraint,
    states: HashMap<PointerId, PointerSensorState>,
}

#[derive(Debug, Clone, Copy)]
struct PointerSensorState {
    start: Point,
    start_tick: u64,
    active: bool,
    last: Point,
}

impl PointerSensor {
    pub fn new(constraint: ActivationConstraint) -> Self {
        Self {
            constraint,
            states: HashMap::new(),
        }
    }

    pub fn is_tracking(&self, pointer_id: PointerId) -> bool {
        self.states.contains_key(&pointer_id)
    }

    pub fn set_constraint(&mut self, constraint: ActivationConstraint) {
        self.constraint = constraint;
    }

    pub fn clear_pointer(&mut self, pointer_id: PointerId) {
        self.states.remove(&pointer_id);
    }

    pub fn is_active(&self, pointer_id: PointerId) -> bool {
        self.states.get(&pointer_id).is_some_and(|s| s.active)
    }

    pub fn handle(&mut self, ev: SensorEvent) -> SensorOutput {
        match ev {
            SensorEvent::Down {
                pointer_id,
                position,
                tick,
            } => {
                self.states.insert(
                    pointer_id,
                    PointerSensorState {
                        start: position,
                        start_tick: tick,
                        active: false,
                        last: position,
                    },
                );
                SensorOutput::Pending
            }
            SensorEvent::Move {
                pointer_id,
                position,
                tick,
            } => {
                let Some(state) = self.states.get_mut(&pointer_id) else {
                    return SensorOutput::Pending;
                };
                state.last = position;
                if !state.active
                    && self
                        .constraint
                        .is_satisfied(state.start_tick, tick, state.start, position)
                {
                    state.active = true;
                    return SensorOutput::DragStart {
                        pointer_id,
                        start: state.start,
                        position,
                    };
                }
                if state.active {
                    let translation = Point::new(
                        Px(position.x.0 - state.start.x.0),
                        Px(position.y.0 - state.start.y.0),
                    );
                    return SensorOutput::DragMove {
                        pointer_id,
                        start: state.start,
                        position,
                        translation,
                    };
                }
                SensorOutput::Pending
            }
            SensorEvent::Up {
                pointer_id,
                position,
                ..
            } => {
                let state = self.states.remove(&pointer_id);
                let Some(state) = state else {
                    return SensorOutput::Pending;
                };
                if state.active {
                    SensorOutput::DragEnd {
                        pointer_id,
                        start: state.start,
                        position,
                    }
                } else {
                    SensorOutput::Pending
                }
            }
            SensorEvent::Cancel {
                pointer_id,
                position,
                ..
            } => {
                let state = self.states.remove(&pointer_id);
                let Some(state) = state else {
                    return SensorOutput::Pending;
                };
                if state.active {
                    SensorOutput::DragCancel {
                        pointer_id,
                        start: state.start,
                        position,
                    }
                } else {
                    SensorOutput::Pending
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_constraint_requires_motion() {
        let mut sensor = PointerSensor::new(ActivationConstraint::Distance { px: 4.0 });
        let pointer = PointerId(1);
        let start = Point::new(Px(0.0), Px(0.0));

        assert_eq!(
            sensor.handle(SensorEvent::Down {
                pointer_id: pointer,
                position: start,
                tick: 0
            }),
            SensorOutput::Pending
        );

        assert_eq!(
            sensor.handle(SensorEvent::Move {
                pointer_id: pointer,
                position: Point::new(Px(3.9), Px(0.0)),
                tick: 1
            }),
            SensorOutput::Pending
        );

        assert_eq!(
            sensor.handle(SensorEvent::Move {
                pointer_id: pointer,
                position: Point::new(Px(4.0), Px(0.0)),
                tick: 2
            }),
            SensorOutput::DragStart {
                pointer_id: pointer,
                start,
                position: Point::new(Px(4.0), Px(0.0))
            }
        );
    }

    #[test]
    fn delay_constraint_requires_ticks() {
        let mut sensor = PointerSensor::new(ActivationConstraint::DelayTicks { ticks: 3 });
        let pointer = PointerId(2);
        let start = Point::new(Px(0.0), Px(0.0));

        let _ = sensor.handle(SensorEvent::Down {
            pointer_id: pointer,
            position: start,
            tick: 10,
        });

        assert_eq!(
            sensor.handle(SensorEvent::Move {
                pointer_id: pointer,
                position: Point::new(Px(0.0), Px(0.0)),
                tick: 12
            }),
            SensorOutput::Pending
        );

        assert_eq!(
            sensor.handle(SensorEvent::Move {
                pointer_id: pointer,
                position: Point::new(Px(0.0), Px(0.0)),
                tick: 13
            }),
            SensorOutput::DragStart {
                pointer_id: pointer,
                start,
                position: Point::new(Px(0.0), Px(0.0))
            }
        );
    }

    #[test]
    fn multi_pointer_is_independent() {
        let mut sensor = PointerSensor::new(ActivationConstraint::Distance { px: 2.0 });
        let p1 = PointerId(1);
        let p2 = PointerId(2);
        let start = Point::new(Px(0.0), Px(0.0));
        let _ = sensor.handle(SensorEvent::Down {
            pointer_id: p1,
            position: start,
            tick: 0,
        });
        let _ = sensor.handle(SensorEvent::Down {
            pointer_id: p2,
            position: start,
            tick: 0,
        });

        let out1 = sensor.handle(SensorEvent::Move {
            pointer_id: p1,
            position: Point::new(Px(2.0), Px(0.0)),
            tick: 1,
        });
        assert!(matches!(out1, SensorOutput::DragStart { pointer_id, .. } if pointer_id == p1));
        assert!(!sensor.is_active(p2));
    }
}

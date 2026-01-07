#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WorkBudget {
    pub points: u32,
    pub labels: u32,
    pub marks: u32,
}

impl WorkBudget {
    pub const fn new(points: u32, labels: u32, marks: u32) -> Self {
        Self {
            points,
            labels,
            marks,
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.points == 0 && self.labels == 0 && self.marks == 0
    }

    pub fn take_points(&mut self, count: u32) -> u32 {
        let taken = self.points.min(count);
        self.points -= taken;
        taken
    }

    pub fn take_labels(&mut self, count: u32) -> u32 {
        let taken = self.labels.min(count);
        self.labels -= taken;
        taken
    }

    pub fn take_marks(&mut self, count: u32) -> u32 {
        let taken = self.marks.min(count);
        self.marks -= taken;
        taken
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StepResult {
    pub unfinished: bool,
}

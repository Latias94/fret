#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataWindow {
    pub min: f64,
    pub max: f64,
}

impl DataWindow {
    pub fn is_valid(&self) -> bool {
        self.min.is_finite() && self.max.is_finite() && self.max > self.min
    }

    pub fn clamp_non_degenerate(&mut self) {
        if !self.is_valid() {
            self.min = 0.0;
            self.max = 1.0;
        }
    }
}

pub type DataWindowX = DataWindow;
pub type DataWindowY = DataWindow;

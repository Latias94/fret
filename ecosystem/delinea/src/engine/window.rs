#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataWindowX {
    pub x_min: f64,
    pub x_max: f64,
}

impl DataWindowX {
    pub fn is_valid(&self) -> bool {
        self.x_min.is_finite() && self.x_max.is_finite() && self.x_max > self.x_min
    }

    pub fn clamp_non_degenerate(&mut self) {
        if !self.is_valid() {
            self.x_min = 0.0;
            self.x_max = 1.0;
        }
    }
}

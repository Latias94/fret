pub mod axis;
pub mod colormap;
#[cfg(any(test, feature = "compat-retained-canvas"))]
pub mod decimate;
pub mod grid;
pub(crate) mod histogram;
pub mod histogram2d;
pub mod readout;
pub mod scale;
pub mod shape;
pub mod view;

//! Public floating-surface facade types owned separately from the root `imui.rs` hub.

mod area;
mod window;

pub use area::{FloatingAreaContext, FloatingAreaOptions};
pub use window::{FloatingWindowOptions, FloatingWindowResizeOptions, WindowOptions};

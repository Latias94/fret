//! Cross-platform time primitives.
//!
//! `std::time::Instant` is not available on `wasm32-unknown-unknown`, so we use `web_time` there.

pub use std::time::Duration;

#[cfg(target_arch = "wasm32")]
pub use web_time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Instant, SystemTime, UNIX_EPOCH};

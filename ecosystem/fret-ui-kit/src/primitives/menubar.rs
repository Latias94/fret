//! Radix `Menubar` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/menubar>
//!
//! In Radix, `Menubar` uses `Menu`-like content behavior but has an additional "trigger row"
//! interaction policy (roving between triggers, hover switches menus when one is open, etc.).
//! In Fret the shared menu content/submenu behavior lives in `crate::primitives::menu`; this module
//! exists as a Radix-named facade for consumers that want to align their mental model with Radix.

pub use crate::primitives::menu::*;


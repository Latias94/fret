//! Radix `ContextMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/context-menu>
//!
//! In Radix, `ContextMenu` is built on top of `Menu` with a different trigger/open policy.
//! In Fret we share the same underlying behavior via `crate::primitives::menu` and expose
//! Radix-named entry points here for reuse outside the shadcn layer.

pub use crate::primitives::menu::*;


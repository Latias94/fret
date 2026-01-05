//! Radix `DropdownMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/dropdown-menu>
//!
//! In Radix, `DropdownMenu` is built on top of `Menu` with a trigger button and popper-based
//! placement. In Fret we share the same underlying behavior via `crate::primitives::menu` and
//! expose Radix-named entry points here for reuse outside the shadcn layer.

pub use crate::primitives::menu::*;


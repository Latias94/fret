//! ScrollArea shared types (Radix-aligned outcomes).

/// Matches Radix ScrollArea `type` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollAreaType {
    Auto,
    Always,
    Scroll,
    #[default]
    Hover,
}

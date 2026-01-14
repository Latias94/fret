//! ScrollArea shared types (Radix-aligned outcomes).

/// Matches Radix ScrollArea `type` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAreaType {
    Auto,
    Always,
    Scroll,
    Hover,
}

impl Default for ScrollAreaType {
    fn default() -> Self {
        // Radix default is `type="hover"`.
        Self::Hover
    }
}

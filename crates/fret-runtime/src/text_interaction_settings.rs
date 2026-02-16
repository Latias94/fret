/// Text interaction settings that affect editor-grade behaviors.
///
/// This is intentionally a small, opt-in surface: platform-specific policies (like Linux primary
/// selection) should be disabled by default so cross-platform apps do not accidentally change UX.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextInteractionSettings {
    /// When true, selection changes caused by mouse interactions may update the Linux primary
    /// selection, and middle-click paste will read from primary selection when available.
    pub linux_primary_selection: bool,
    /// Margin (logical px) used for editor-like horizontal auto-scroll behaviors during pointer
    /// selection drags, and for keeping the caret comfortably within view while focused.
    pub horizontal_autoscroll_margin_px: u16,
    /// Maximum horizontal scroll step (logical px) applied per timer tick while auto-scrolling.
    pub horizontal_autoscroll_max_step_px: u16,
}

impl Default for TextInteractionSettings {
    fn default() -> Self {
        Self {
            linux_primary_selection: false,
            horizontal_autoscroll_margin_px: 12,
            horizontal_autoscroll_max_step_px: 24,
        }
    }
}

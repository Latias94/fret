/// Text interaction settings that affect editor-grade behaviors.
///
/// This is intentionally a small, opt-in surface: platform-specific policies (like Linux primary
/// selection) should be disabled by default so cross-platform apps do not accidentally change UX.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextInteractionSettings {
    /// When true, selection changes caused by mouse interactions may update the Linux primary
    /// selection, and middle-click paste will read from primary selection when available.
    pub linux_primary_selection: bool,
    /// When true, focused text inputs blink their caret.
    ///
    /// This is intentionally opt-in to keep diagnostics and tests deterministic unless the app
    /// explicitly enables caret blink.
    pub caret_blink: bool,
    /// Caret blink toggle interval in milliseconds.
    ///
    /// This is a best-effort hint; runners may coalesce timers.
    pub caret_blink_interval_ms: u16,
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
            caret_blink: false,
            caret_blink_interval_ms: 500,
            horizontal_autoscroll_margin_px: 12,
            horizontal_autoscroll_max_step_px: 24,
        }
    }
}

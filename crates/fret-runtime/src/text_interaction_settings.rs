/// Text interaction settings that affect editor-grade behaviors.
///
/// This is intentionally a small, opt-in surface: platform-specific policies (like Linux primary
/// selection) should be disabled by default so cross-platform apps do not accidentally change UX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextInteractionSettings {
    /// When true, selection changes caused by mouse interactions may update the Linux primary
    /// selection, and middle-click paste will read from primary selection when available.
    pub linux_primary_selection: bool,
}

use std::sync::OnceLock;

#[inline]
pub(crate) fn strict_runtime_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(fret_runtime::strict_runtime::strict_runtime_enabled_from_env)
}

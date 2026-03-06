use std::sync::Once;

#[cfg(not(target_arch = "wasm32"))]
fn env_usize(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn env_flag(key: &str) -> bool {
    std::env::var_os(key).is_some_and(|v| !v.is_empty())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn configure_stacksafe_from_env() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let preset_enabled = env_flag("FRET_STACKSAFE");

        let min_bytes = env_usize("FRET_STACKSAFE_MIN_BYTES");
        let alloc_bytes = env_usize("FRET_STACKSAFE_ALLOC_BYTES");

        if !preset_enabled && min_bytes.is_none() && alloc_bytes.is_none() {
            return;
        }

        if preset_enabled {
            // Prefer larger defaults when explicitly enabled.
            stacksafe::set_minimum_stack_size(2 * 1024 * 1024);
            stacksafe::set_stack_allocation_size(8 * 1024 * 1024);
        }

        if let Some(bytes) = min_bytes {
            stacksafe::set_minimum_stack_size(bytes);
        }
        if let Some(bytes) = alloc_bytes {
            stacksafe::set_stack_allocation_size(bytes);
        }

        tracing::info!(
            stacksafe_min_bytes = stacksafe::get_minimum_stack_size(),
            stacksafe_alloc_bytes = stacksafe::get_stack_allocation_size(),
            "stacksafe configured"
        );
    });
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn configure_stacksafe_from_env() {}

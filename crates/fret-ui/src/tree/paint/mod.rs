fn paint_cache_relax_view_cache_gating() -> bool {
    crate::runtime_config::ui_runtime_config().paint_cache_relax_view_cache_gating
}

#[cfg(test)]
thread_local! {
    static PAINT_CACHE_ALLOW_HIT_TEST_ONLY_TEST_OVERRIDE: std::cell::Cell<Option<bool>> =
        const { std::cell::Cell::new(None) };
}

fn paint_cache_allow_hit_test_only() -> bool {
    #[cfg(test)]
    if let Some(value) = PAINT_CACHE_ALLOW_HIT_TEST_ONLY_TEST_OVERRIDE.with(std::cell::Cell::get) {
        return value;
    }

    crate::runtime_config::ui_runtime_config().paint_cache_allow_hit_test_only
}

#[cfg(test)]
fn set_paint_cache_allow_hit_test_only_for_test(value: Option<bool>) {
    PAINT_CACHE_ALLOW_HIT_TEST_ONLY_TEST_OVERRIDE.with(|slot| slot.set(value));
}

mod entry;
mod node;

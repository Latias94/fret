thread_local! {
    static INJECTED_EVENT_SCOPE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Marks a call stack as delivering a synthetic event owned by tooling/runtime plumbing.
///
/// Consumers that normally ignore external input while scripted playback is active can use
/// [`in_injected_event_scope`] to distinguish those synthetic deliveries from real platform input.
pub fn with_injected_event_scope<R>(f: impl FnOnce() -> R) -> R {
    INJECTED_EVENT_SCOPE.with(|cell| {
        let prev = cell.replace(true);
        let out = f();
        cell.set(prev);
        out
    })
}

/// Returns `true` while the current thread is delivering a tooling-owned synthetic event.
pub fn in_injected_event_scope() -> bool {
    INJECTED_EVENT_SCOPE.with(|cell| cell.get())
}

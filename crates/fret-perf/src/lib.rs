use fret_core::time::Instant;
use std::time::Duration;

#[inline]
pub fn measure<T>(enabled: bool, f: impl FnOnce() -> T) -> (T, Option<Duration>) {
    if !enabled {
        return (f(), None);
    }
    let started = Instant::now();
    let out = f();
    (out, Some(started.elapsed()))
}

#[inline]
pub fn measure_span<T>(
    time_enabled: bool,
    span_enabled: bool,
    make_span: impl FnOnce() -> tracing::Span,
    f: impl FnOnce() -> T,
) -> (T, Option<Duration>) {
    if !time_enabled && !span_enabled {
        return (f(), None);
    }

    let started = time_enabled.then(Instant::now);
    let span = span_enabled
        .then(make_span)
        .unwrap_or_else(tracing::Span::none);
    let _guard = span.enter();
    let out = f();
    (out, started.map(|s| s.elapsed()))
}

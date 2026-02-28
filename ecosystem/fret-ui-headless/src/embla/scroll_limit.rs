use crate::embla::limit::Limit;
use crate::embla::utils::array_last;

/// Ported from Embla `ScrollLimit`.
///
/// Upstream: `repo-ref/embla-carousel/packages/embla-carousel/src/components/ScrollLimit.ts`
pub fn scroll_limit(content_size: f32, scroll_snaps: &[f32], loop_enabled: bool) -> Limit {
    let max = scroll_snaps.first().copied().unwrap_or_default();
    let min = if loop_enabled {
        max - content_size
    } else {
        array_last(scroll_snaps)
    };
    Limit::new(min, max)
}

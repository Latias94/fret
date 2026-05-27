//! Stable Material recipe test-id helpers.

use std::sync::Arc;

pub(crate) fn part_test_id(base: &Arc<str>, part: &str) -> Arc<str> {
    Arc::from(format!("{base}.{part}"))
}

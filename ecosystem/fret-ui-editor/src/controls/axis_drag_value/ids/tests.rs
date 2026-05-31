use std::sync::Arc;

use super::axis_drag_value_test_ids;

fn text(value: &Option<Arc<str>>) -> Option<&str> {
    value.as_deref()
}

#[test]
fn axis_drag_value_test_ids_derive_scrub_and_active_typing_segments() {
    let ids = axis_drag_value_test_ids(Some(Arc::from("transform.x")), None, true);

    assert_eq!(text(&ids.scrub), Some("transform.x"));
    assert_eq!(text(&ids.active_typing), Some("transform.x.typing"));
    assert_eq!(text(&ids.scrub_axis), Some("transform.x.axis"));
    assert_eq!(text(&ids.scrub_value), Some("transform.x.value"));
    assert_eq!(text(&ids.scrub_prefix), Some("transform.x.prefix"));
    assert_eq!(text(&ids.scrub_suffix), Some("transform.x.suffix"));
    assert_eq!(text(&ids.typing_axis), Some("transform.x.typing.axis"));
    assert_eq!(text(&ids.typing_input), Some("transform.x.typing.input"));
    assert_eq!(text(&ids.typing_prefix), Some("transform.x.typing.prefix"));
    assert_eq!(text(&ids.typing_suffix), Some("transform.x.typing.suffix"));
    assert_eq!(
        text(&ids.typing_error_icon),
        Some("transform.x.typing.error")
    );
    assert_eq!(text(&ids.scrub_reset), Some("transform.x.reset"));
    assert_eq!(text(&ids.typing_reset), Some("transform.x.typing.reset"));
}

#[test]
fn axis_drag_value_test_ids_use_explicit_reset_ids_for_scrub_and_typing() {
    let ids = axis_drag_value_test_ids(
        Some(Arc::from("transform.x")),
        Some(Arc::from("reset-x")),
        true,
    );

    assert_eq!(text(&ids.scrub_reset), Some("reset-x"));
    assert_eq!(text(&ids.typing_reset), Some("reset-x.typing"));
}

#[test]
fn axis_drag_value_test_ids_skip_typing_segments_when_not_typing() {
    let ids = axis_drag_value_test_ids(
        Some(Arc::from("transform.x")),
        Some(Arc::from("reset-x")),
        false,
    );

    assert_eq!(text(&ids.active_typing), None);
    assert_eq!(text(&ids.typing_axis), None);
    assert_eq!(text(&ids.typing_input), None);
    assert_eq!(text(&ids.typing_prefix), None);
    assert_eq!(text(&ids.typing_suffix), None);
    assert_eq!(text(&ids.typing_error_icon), None);
    assert_eq!(text(&ids.scrub_reset), Some("reset-x"));
    assert_eq!(text(&ids.typing_reset), None);
}

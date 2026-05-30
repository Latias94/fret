use super::{t_from_value, value_from_x};

#[test]
fn slider_t_from_value_returns_zero_for_degenerate_ranges() {
    assert_eq!(t_from_value(1.0, 1.0, true, 1.0), 0.0);
    assert_eq!(t_from_value(0.0, f64::INFINITY, true, 1.0), 0.0);
}

#[test]
fn slider_t_from_value_clamps_when_requested() {
    assert_eq!(t_from_value(0.0, 10.0, true, 12.0), 1.0);
    assert_eq!(t_from_value(0.0, 10.0, true, -2.0), 0.0);
    assert_eq!(t_from_value(0.0, 10.0, false, 12.0), 1.2);
}

#[test]
fn slider_value_from_x_accounts_for_thumb_radius_and_step_quantization() {
    let value = value_from_x(0.0, 10.0, true, Some(0.5), 55.0, 110.0, 10.0);

    assert_eq!(value, 5.0);
}

#[test]
fn slider_value_from_x_returns_quantized_min_when_track_has_no_available_width() {
    let value = value_from_x(0.3, 1.0, true, Some(0.25), 10.0, 8.0, 10.0);

    assert_eq!(value, 0.3);
}

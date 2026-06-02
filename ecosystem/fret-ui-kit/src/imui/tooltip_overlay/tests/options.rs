use super::*;

#[test]
fn tooltip_default_options_use_top_center_placement() {
    let options = TooltipOptions::default();
    assert_eq!(options.placement.side, crate::primitives::popper::Side::Top);
    assert_eq!(
        options.placement.align,
        crate::primitives::popper::Align::Center
    );
    assert_eq!(options.window_margin, Px(8.0));
    assert_eq!(options.open_delay_frames_override, None);
    assert_eq!(options.close_delay_frames_override, None);
    assert!(options.test_id.is_none());
}

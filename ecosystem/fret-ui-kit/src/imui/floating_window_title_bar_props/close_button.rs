use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::{Length, PressableA11y, PressableProps};

pub(in crate::imui) fn title_bar_close_button_props(
    close_button_test_id: Arc<str>,
) -> PressableProps {
    let mut props = PressableProps::default();
    props.a11y = PressableA11y {
        role: Some(SemanticsRole::Button),
        label: Some(Arc::from("Close")),
        test_id: Some(close_button_test_id),
        ..Default::default()
    };
    props.layout.size.width = Length::Px(Px(20.0));
    props.layout.size.height = Length::Px(Px(20.0));
    props.layout.flex.shrink = 0.0;
    props
}

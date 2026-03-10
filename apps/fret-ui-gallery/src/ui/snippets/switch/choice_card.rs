pub const SOURCE: &str = include_str!("choice_card.rs");

// region: example
use fret_core::Px;
use fret_ui_kit::primitives::control_registry::ControlId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    share: Option<Model<bool>>,
    notifications: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (share, notifications) = cx.with_state(Models::default, |st| {
        (st.share.clone(), st.notifications.clone())
    });
    let (share, notifications) = match (share, notifications) {
        (Some(share), Some(notifications)) => (share, notifications),
        _ => {
            let share = cx.app.models_mut().insert(false);
            let notifications = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.share = Some(share.clone());
                st.notifications = Some(notifications.clone());
            });
            (share, notifications)
        }
    };

    let share_id = ControlId::from("ui-gallery-switch-choice-card-share");
    let notifications_id = ControlId::from("ui-gallery-switch-choice-card-notifications");

    let share_card = shadcn::FieldLabel::new("Share across devices")
        .for_control(share_id.clone())
        .test_id("ui-gallery-switch-choice-card-share-label")
        .wrap([shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Share across devices").into_element(cx),
                shadcn::FieldDescription::new(
                    "Focus is shared across devices, and turns off when you leave the app.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(share)
                .control_id(share_id)
                .a11y_label("Share across devices")
                .test_id("ui-gallery-switch-choice-card-share-control")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .into_element(cx)])
        .into_element(cx);

    let notifications_card = shadcn::FieldLabel::new("Enable notifications")
        .for_control(notifications_id.clone())
        .test_id("ui-gallery-switch-choice-card-notifications-label")
        .wrap([shadcn::Field::new([
            shadcn::FieldContent::new([
                shadcn::FieldTitle::new("Enable notifications").into_element(cx),
                shadcn::FieldDescription::new(
                    "Receive notifications when focus mode is enabled or disabled.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::Switch::new(notifications)
                .control_id(notifications_id)
                .a11y_label("Enable notifications")
                .test_id("ui-gallery-switch-choice-card-notifications-control")
                .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_style(
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Lg)
                .p(Space::N4),
        )
        .into_element(cx)])
        .into_element(cx);

    shadcn::FieldGroup::new([share_card, notifications_card])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-switch-choice-card")
}
// endregion: example

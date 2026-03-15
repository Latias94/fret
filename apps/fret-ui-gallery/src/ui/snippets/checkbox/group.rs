pub const SOURCE: &str = include_str!("group.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let group_security = cx.local_model_keyed("group_security", || true);
    let group_updates = cx.local_model_keyed("group_updates", || false);
    let group_marketing = cx.local_model_keyed("group_marketing", || false);

    let group_item = |cx: &mut UiCx<'_>,
                      label: &'static str,
                      desc: &'static str,
                      value: Model<bool>,
                      test_id: &'static str| {
        shadcn::Field::new([
            shadcn::Checkbox::new(value)
                // Required for label click -> focus/toggle forwarding.
                .control_id(test_id)
                .a11y_label(label)
                .test_id(test_id)
                .into_element(cx),
            shadcn::FieldContent::new([
                shadcn::FieldLabel::new(label)
                    .for_control(test_id)
                    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
                    .test_id(format!("{test_id}.label"))
                    .into_element(cx),
                shadcn::FieldDescription::new(desc)
                    .into_element(cx)
                    .test_id(format!("{test_id}.desc")),
            ])
            .into_element(cx),
        ])
        .orientation(shadcn::FieldOrientation::Horizontal)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    ui::v_flex(|cx| {
        vec![
            group_item(
                cx,
                "Security alerts",
                "Critical account changes and sign-in events.",
                group_security,
                "ui-gallery-checkbox-group-security",
            ),
            group_item(
                cx,
                "Product updates",
                "Major feature releases and migration notices.",
                group_updates,
                "ui-gallery-checkbox-group-updates",
            ),
            group_item(
                cx,
                "Marketing emails",
                "Tips, webinars, and promotional announcements.",
                group_marketing,
                "ui-gallery-checkbox-group-marketing",
            ),
        ]
    })
    .gap(Space::N3)
    .items_start()
    .layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
    .into_element(cx)
    .test_id("ui-gallery-checkbox-group")
}
// endregion: example

pub const SOURCE: &str = include_str!("group.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct Models {
    group_security: Option<Model<bool>>,
    group_updates: Option<Model<bool>>,
    group_marketing: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (group_security, group_updates, group_marketing) = cx.with_state(Models::default, |st| {
        (
            st.group_security.clone(),
            st.group_updates.clone(),
            st.group_marketing.clone(),
        )
    });
    let (group_security, group_updates, group_marketing) =
        match (group_security, group_updates, group_marketing) {
            (Some(group_security), Some(group_updates), Some(group_marketing)) => {
                (group_security, group_updates, group_marketing)
            }
            _ => {
                let group_security = cx.app.models_mut().insert(true);
                let group_updates = cx.app.models_mut().insert(false);
                let group_marketing = cx.app.models_mut().insert(false);
                cx.with_state(Models::default, |st| {
                    st.group_security = Some(group_security.clone());
                    st.group_updates = Some(group_updates.clone());
                    st.group_marketing = Some(group_marketing.clone());
                });
                (group_security, group_updates, group_marketing)
            }
        };

    let group_item = |cx: &mut ElementContext<'_, H>,
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

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N3)
            .items_start()
            .layout(LayoutRefinement::default().w_full().max_w(Px(460.0))),
        |cx| {
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
        },
    )
    .test_id("ui-gallery-checkbox-group")
}
// endregion: example

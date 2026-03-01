// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    show_status_bar: Option<Model<bool>>,
    show_activity_bar: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let show_status_bar = match state.show_status_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| {
                st.show_status_bar = Some(model.clone())
            });
            model
        }
    };

    let show_activity_bar = match state.show_activity_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.show_activity_bar = Some(model.clone())
            });
            model
        }
    };

    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Checkboxes Icons")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-checkboxes-icons-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0)),
            |_cx| {
                [
                    shadcn::DropdownMenuCheckboxItem::new(show_status_bar.clone(), "Status Bar")
                        .leading_icon(IconId::new_static("lucide.panel-top"))
                        .test_id("ui-gallery-dropdown-menu-checkboxes-icons-status-bar")
                        .into(),
                    shadcn::DropdownMenuCheckboxItem::new(
                        show_activity_bar.clone(),
                        "Activity Bar",
                    )
                    .leading_icon(IconId::new_static("lucide.layout-dashboard"))
                    .test_id("ui-gallery-dropdown-menu-checkboxes-icons-activity-bar")
                    .into(),
                ]
            },
        )
        .test_id("ui-gallery-dropdown-menu-checkboxes-icons")
}
// endregion: example

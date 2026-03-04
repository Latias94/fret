pub const SOURCE: &str = include_str!("checkboxes.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    show_status_bar: Option<Model<bool>>,
    show_activity_bar: Option<Model<bool>>,
    show_panel: Option<Model<bool>>,
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

    let show_panel = match state.show_panel {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.show_panel = Some(model.clone()));
            model
        }
    };

    shadcn::DropdownMenu::new_controllable(cx, None, false)
        .into_element_parts(
            cx,
            |cx| {
                shadcn::DropdownMenuTrigger::new(
                    shadcn::Button::new("Open")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dropdown-menu-checkboxes-trigger")
                        .into_element(cx),
                )
            },
            shadcn::DropdownMenuContent::new()
                .align(shadcn::DropdownMenuAlign::Start)
                .side_offset(Px(4.0))
                // shadcn/ui docs: `DropdownMenuContent className="w-40"`.
                .min_width(Px(160.0)),
            |_cx| {
                [shadcn::DropdownMenuGroup::new([
                    shadcn::DropdownMenuLabel::new("Appearance").into(),
                    shadcn::DropdownMenuCheckboxItem::new(show_status_bar.clone(), "Status Bar")
                        .test_id("ui-gallery-dropdown-menu-checkboxes-status-bar")
                        .into(),
                    shadcn::DropdownMenuCheckboxItem::new(
                        show_activity_bar.clone(),
                        "Activity Bar",
                    )
                    .disabled(true)
                    .test_id("ui-gallery-dropdown-menu-checkboxes-activity-bar")
                    .into(),
                    shadcn::DropdownMenuCheckboxItem::new(show_panel.clone(), "Panel").into(),
                ])
                .into()]
            },
        )
        .test_id("ui-gallery-dropdown-menu-checkboxes")
}
// endregion: example

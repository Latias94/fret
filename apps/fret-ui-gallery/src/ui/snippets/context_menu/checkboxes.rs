// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    show_status_bar: Option<Model<bool>>,
    show_activity_bar: Option<Model<bool>>,
    show_line_numbers: Option<Model<bool>>,
}

fn trigger_surface<H: UiHost>(cx: &mut ElementContext<'_, H>, label: &'static str) -> AnyElement {
    shadcn::Button::new(label)
        .variant(shadcn::ButtonVariant::Outline)
        .size(shadcn::ButtonSize::Sm)
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let show_status_bar = match state.show_status_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| st.show_status_bar = Some(model.clone()));
            model
        }
    };

    let show_activity_bar = match state.show_activity_bar {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(Models::default, |st| st.show_activity_bar = Some(model.clone()));
            model
        }
    };

    let show_line_numbers = match state.show_line_numbers {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.show_line_numbers = Some(model.clone()));
            model
        }
    };

    shadcn::ContextMenu::new_controllable(cx, None, false)
        .content_test_id("ui-gallery-context-menu-checkboxes-content")
        .into_element(
            cx,
            |cx| {
                trigger_surface(cx, "Right click for checkboxes")
                    .test_id("ui-gallery-context-menu-checkboxes-trigger")
            },
            |_cx| {
                vec![
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(show_status_bar.clone(), "Status Bar")
                            .test_id("ui-gallery-context-menu-checkboxes-status-bar"),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(
                            show_activity_bar.clone(),
                            "Activity Bar",
                        ),
                    ),
                    shadcn::ContextMenuEntry::CheckboxItem(
                        shadcn::ContextMenuCheckboxItem::new(
                            show_line_numbers.clone(),
                            "Show Line Numbers",
                        ),
                    ),
                ]
            },
        )
        .test_id("ui-gallery-context-menu-checkboxes")
}
// endregion: example


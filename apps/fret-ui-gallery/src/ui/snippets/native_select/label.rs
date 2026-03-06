pub const SOURCE: &str = include_str!("label.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    value: Option<Model<Option<Arc<str>>>>,
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let (value, open) = cx.with_state(Models::default, |st| (st.value.clone(), st.open.clone()));
    let (value, open) = match (value, open) {
        (Some(value), Some(open)) => (value, open),
        _ => {
            let value = cx.app.models_mut().insert(None::<Arc<str>>);
            let open = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.value = Some(value.clone());
                st.open = Some(open.clone());
            });
            (value, open)
        }
    };

    let control_id = "ui-gallery-native-select-label-control";
    let label = shadcn::FieldLabel::new("Assignee")
        .for_control(control_id)
        .test_id("ui-gallery-native-select-label-label")
        .into_element(cx);

    let select = shadcn::NativeSelect::new(value, open)
        .control_id(control_id)
        .a11y_label("Assignee")
        .placeholder("Select assignee")
        .trigger_test_id("ui-gallery-native-select-label-trigger")
        .test_id_prefix("ui-gallery-native-select-label-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select assignee"),
            shadcn::NativeSelectOption::new("alice", "Alice"),
            shadcn::NativeSelectOption::new("bob", "Bob"),
            shadcn::NativeSelectOption::new("carol", "Carol"),
        ])
        .refine_layout(LayoutRefinement::default().max_w(Px(320.0)).min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-native-select-label-native");

    shadcn::Field::new(vec![label, select])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-native-select-label")
}
// endregion: example

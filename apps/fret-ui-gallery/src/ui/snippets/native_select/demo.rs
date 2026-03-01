// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> AnyElement {
    // shadcn NativeSelect is `w-fit` at the wrapper level; keep the gallery close to that default
    // (content-driven width), while still clamping to a reasonable max.
    let select_layout = LayoutRefinement::default().max_w(Px(320.0)).min_w_0();

    let select = shadcn::NativeSelect::new(value, open)
        .a11y_label("Native select: status")
        .placeholder("Select status")
        .trigger_test_id("ui-gallery-native-select-basic-native-trigger")
        .test_id_prefix("ui-gallery-native-select-basic-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select status"),
            shadcn::NativeSelectOption::new("todo", "Todo"),
            shadcn::NativeSelectOption::new("in-progress", "In Progress"),
            shadcn::NativeSelectOption::new("done", "Done"),
            shadcn::NativeSelectOption::new("cancelled", "Cancelled"),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-basic-native");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| vec![select],
    )
    .test_id("ui-gallery-native-select-demo")
}
// endregion: example


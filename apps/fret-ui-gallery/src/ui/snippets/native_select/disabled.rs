// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> AnyElement {
    let select_layout = LayoutRefinement::default().max_w(Px(320.0)).min_w_0();

    let select = shadcn::NativeSelect::new(value, open)
        .a11y_label("Native select: priority (disabled)")
        .placeholder("Select priority")
        .disabled(true)
        .trigger_test_id("ui-gallery-native-select-disabled-native-trigger")
        .test_id_prefix("ui-gallery-native-select-disabled-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select priority"),
            shadcn::NativeSelectOption::new("low", "Low"),
            shadcn::NativeSelectOption::new("medium", "Medium"),
            shadcn::NativeSelectOption::new("high", "High"),
            shadcn::NativeSelectOption::new("critical", "Critical"),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-disabled-native");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| vec![select],
    )
    .test_id("ui-gallery-native-select-disabled")
}
// endregion: example


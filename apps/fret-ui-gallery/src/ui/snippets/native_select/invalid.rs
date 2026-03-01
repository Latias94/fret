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
        .a11y_label("Native select: role (invalid)")
        .placeholder("Select role")
        .aria_invalid(true)
        .trigger_test_id("ui-gallery-native-select-error-native-trigger")
        .test_id_prefix("ui-gallery-native-select-error-native")
        .options([
            shadcn::NativeSelectOption::placeholder("Select role"),
            shadcn::NativeSelectOption::new("admin", "Admin"),
            shadcn::NativeSelectOption::new("editor", "Editor"),
            shadcn::NativeSelectOption::new("viewer", "Viewer"),
            shadcn::NativeSelectOption::new("guest", "Guest"),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-error-native");

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |_cx| vec![select],
    )
    .test_id("ui-gallery-native-select-error")
}
// endregion: example


pub const SOURCE: &str = include_str!("with_groups.rs");

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
        .a11y_label("Native select: department")
        .placeholder("Select department")
        .trigger_test_id("ui-gallery-native-select-groups-native-trigger")
        .test_id_prefix("ui-gallery-native-select-groups-native")
        .options([shadcn::NativeSelectOption::placeholder("Select department")])
        .optgroups([
            shadcn::NativeSelectOptGroup::new(
                "Engineering",
                [
                    shadcn::NativeSelectOption::new("frontend", "Frontend"),
                    shadcn::NativeSelectOption::new("backend", "Backend"),
                    shadcn::NativeSelectOption::new("devops", "DevOps"),
                ],
            ),
            shadcn::NativeSelectOptGroup::new(
                "Sales",
                [
                    shadcn::NativeSelectOption::new("sales-rep", "Sales Rep"),
                    shadcn::NativeSelectOption::new("account-manager", "Account Manager"),
                    shadcn::NativeSelectOption::new("sales-director", "Sales Director"),
                ],
            ),
            shadcn::NativeSelectOptGroup::new(
                "Operations",
                [
                    shadcn::NativeSelectOption::new("support", "Customer Support"),
                    shadcn::NativeSelectOption::new("product-manager", "Product Manager"),
                    shadcn::NativeSelectOption::new("ops-manager", "Operations Manager"),
                ],
            ),
        ])
        .refine_layout(select_layout)
        .into_element(cx)
        .test_id("ui-gallery-native-select-groups-native");

    ui::v_flex(|_cx| vec![select])
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-native-select-groups")
}
// endregion: example

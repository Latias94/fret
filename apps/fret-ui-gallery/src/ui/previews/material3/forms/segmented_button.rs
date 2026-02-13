use super::super::super::super::*;

pub(in crate::ui) fn preview_material3_segmented_button(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use std::collections::BTreeSet;

    use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};

    #[derive(Default)]
    struct SegmentedButtonPageModels {
        single_value: Option<Model<Arc<str>>>,
        multi_value: Option<Model<BTreeSet<Arc<str>>>>,
    }

    let single_value = cx.with_state(SegmentedButtonPageModels::default, |st| {
        st.single_value.clone()
    });
    let single_value = match single_value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(Arc::<str>::from("alpha"));
            cx.with_state(SegmentedButtonPageModels::default, |st| {
                st.single_value = Some(model.clone())
            });
            model
        }
    };

    let multi_value = cx.with_state(SegmentedButtonPageModels::default, |st| {
        st.multi_value.clone()
    });
    let multi_value = match multi_value {
        Some(model) => model,
        None => {
            let initial: BTreeSet<Arc<str>> = [Arc::<str>::from("alpha")].into_iter().collect();
            let model = cx.app.models_mut().insert(initial);
            cx.with_state(SegmentedButtonPageModels::default, |st| {
                st.multi_value = Some(model.clone())
            });
            model
        }
    };

    let single_current = cx
        .get_model_cloned(&single_value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let multi_current_len = cx
        .get_model_cloned(&multi_value, Invalidation::Layout)
        .map(|set| set.len())
        .unwrap_or(0);

    let content = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N4).items_start(),
        |cx| {
            vec![
                SegmentedButtonSet::single(single_value.clone())
                    .items(vec![
                        SegmentedButtonItem::new("alpha", "Alpha")
                            .icon(fret_icons::ids::ui::SEARCH)
                            .test_id("ui-gallery-material3-segmented-single-alpha"),
                        SegmentedButtonItem::new("beta", "Beta")
                            .icon(fret_icons::ids::ui::SETTINGS)
                            .test_id("ui-gallery-material3-segmented-single-beta"),
                        SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                            .disabled(true)
                            .icon(fret_icons::ids::ui::MORE_HORIZONTAL)
                            .test_id("ui-gallery-material3-segmented-single-gamma-disabled"),
                    ])
                    .a11y_label("Material 3 Segmented Button (single)")
                    .test_id("ui-gallery-material3-segmented-single")
                    .into_element(cx),
                cx.text(format!("single={}", single_current.as_ref())),
                SegmentedButtonSet::multi(multi_value.clone())
                    .items(vec![
                        SegmentedButtonItem::new("alpha", "Alpha")
                            .test_id("ui-gallery-material3-segmented-multi-alpha"),
                        SegmentedButtonItem::new("beta", "Beta")
                            .test_id("ui-gallery-material3-segmented-multi-beta"),
                        SegmentedButtonItem::new("gamma", "Gamma (disabled)")
                            .disabled(true)
                            .test_id("ui-gallery-material3-segmented-multi-gamma-disabled"),
                    ])
                    .a11y_label("Material 3 Segmented Button (multi)")
                    .test_id("ui-gallery-material3-segmented-multi")
                    .into_element(cx),
                cx.text(format!("multi_count={multi_current_len}")),
            ]
        },
    );

    vec![
        cx.text("Material 3 Segmented Buttons: token-driven outcomes + roving focus + selection."),
        content,
    ]
}

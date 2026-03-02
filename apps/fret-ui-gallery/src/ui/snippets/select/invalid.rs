pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret_core::Px;
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

    let selected = cx
        .get_model_cloned(&value, Invalidation::Paint)
        .unwrap_or_default();
    let invalid = selected.is_none();

    let label = {
        let mut label = shadcn::FieldLabel::new("Fruit");
        if invalid {
            let theme = Theme::global(&*cx.app);
            label = label.text_color(ColorRef::Color(theme.color_token("destructive")));
        }
        label
            .into_element(cx)
            .test_id("ui-gallery-select-invalid-label")
    };

    let select =
        shadcn::Select::new_controllable(cx, Some(value), None::<Arc<str>>, Some(open), false)
            .aria_invalid(invalid)
            .trigger_test_id("ui-gallery-select-invalid-trigger")
            .into_element_parts(
                cx,
                |_cx| {
                    shadcn::SelectTrigger::new()
                        .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
                },
                |_cx| {
                    shadcn::SelectContent::new().with_entries([shadcn::SelectGroup::new([
                        shadcn::SelectItem::new("apple", "Apple")
                            .test_id("ui-gallery-select-invalid-item-apple")
                            .into(),
                        shadcn::SelectItem::new("banana", "Banana")
                            .test_id("ui-gallery-select-invalid-item-banana")
                            .into(),
                        shadcn::SelectItem::new("blueberry", "Blueberry")
                            .test_id("ui-gallery-select-invalid-item-blueberry")
                            .into(),
                    ])
                    .into()])
                },
            );

    let mut children = vec![label, select];
    if invalid {
        children.push(
            shadcn::FieldError::new("Please select a fruit.")
                .into_element(cx)
                .test_id("ui-gallery-select-invalid-error"),
        );
    }

    shadcn::Field::new(children)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
        .into_element(cx)
        .test_id("ui-gallery-select-invalid")
}

// endregion: example

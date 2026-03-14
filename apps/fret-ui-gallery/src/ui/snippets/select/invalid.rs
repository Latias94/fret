pub const SOURCE: &str = include_str!("invalid.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);
    let open = cx.local_model_keyed("open", || false);

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
            .trigger(shadcn::SelectTrigger::new())
            .value(shadcn::SelectValue::new().placeholder("Select a fruit"))
            .content(shadcn::SelectContent::new())
            .entries([shadcn::SelectGroup::new([
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
            .into_element(cx);

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

pub const SOURCE: &str = include_str!("nested_combobox.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn deployment_items() -> Vec<shadcn::ComboboxItem> {
    vec![
        shadcn::ComboboxItem::new("draft", "Draft"),
        shadcn::ComboboxItem::new("review", "In Review"),
        shadcn::ComboboxItem::new("staged", "Staged"),
        shadcn::ComboboxItem::new("release", "Release Ready"),
        shadcn::ComboboxItem::new("archived", "Archived"),
    ]
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let dialog_open = cx.local_model_keyed("nested-combobox-dialog-open", || false);
    let combo_open = cx.local_model_keyed("nested-combobox-open", || false);
    let combo_value = cx.local_model_keyed("nested-combobox-value", || None::<Arc<str>>);
    let combo_query = cx.local_model_keyed("nested-combobox-query", String::new);

    shadcn::Dialog::new(dialog_open)
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("Open deployment dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-nested-combobox-dialog-trigger"),
            )),
            shadcn::DialogPart::content_with(move |cx| {
                let combobox = shadcn::Combobox::new(combo_value.clone(), combo_open.clone())
                    .a11y_label("Deployment status")
                    .query_model(combo_query.clone())
                    .test_id_prefix("ui-gallery-dialog-nested-combobox")
                    .trigger(
                        shadcn::ComboboxTrigger::new()
                            .variant(shadcn::ComboboxTriggerVariant::Button)
                            .width_px(Px(220.0)),
                    )
                    .input(shadcn::ComboboxInput::new().placeholder("Pick status"))
                    .content(
                        shadcn::ComboboxContent::new([
                            shadcn::ComboboxContentPart::input(
                                shadcn::ComboboxInput::new().placeholder("Filter status..."),
                            ),
                            shadcn::ComboboxContentPart::empty(shadcn::ComboboxEmpty::new(
                                "No status found.",
                            )),
                            shadcn::ComboboxContentPart::list(
                                shadcn::ComboboxList::new().items(deployment_items()),
                            ),
                        ])
                        .width_px(Px(260.0))
                        .test_id("ui-gallery-dialog-nested-combobox-content"),
                    )
                    .into_element(cx);

                shadcn::DialogContent::new([])
                    .refine_layout(LayoutRefinement::default().max_w(Px(460.0)))
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([])
                                .with_children(cx, |cx| {
                                    vec![
                                        shadcn::DialogTitle::new("Promote deployment")
                                            .into_element(cx)
                                            .test_id("ui-gallery-dialog-nested-combobox-title"),
                                        shadcn::DialogDescription::new(
                                            "Choose a release status from a nested combobox.",
                                        )
                                        .into_element(cx)
                                        .test_id("ui-gallery-dialog-nested-combobox-description"),
                                    ]
                                })
                                .test_id("ui-gallery-dialog-nested-combobox-header"),
                            ui::v_flex(move |_cx| vec![combobox])
                                .gap(Space::N3)
                                .layout(LayoutRefinement::default().w_full().min_w_0())
                                .into_element(cx)
                                .test_id("ui-gallery-dialog-nested-combobox-body"),
                            shadcn::DialogFooter::new([])
                                .with_children(cx, |cx| {
                                    vec![
                                        shadcn::DialogClose::from_scope().build(
                                            cx,
                                            shadcn::Button::new("Close")
                                                .variant(shadcn::ButtonVariant::Outline)
                                                .test_id("ui-gallery-dialog-nested-combobox-close"),
                                        ),
                                    ]
                                })
                                .test_id("ui-gallery-dialog-nested-combobox-footer"),
                        ]
                    })
                    .test_id("ui-gallery-dialog-nested-combobox-dialog-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example

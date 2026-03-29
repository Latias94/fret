pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn profile_fields<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: Model<String>,
    username: Model<String>,
) -> impl IntoUiElement<H> + use<H> {
    let field = |cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<String>| {
        shadcn::Field::new(ui::children![
            cx;
            shadcn::FieldLabel::new(label),
            shadcn::Input::new(model).refine_layout(LayoutRefinement::default().w_full())
        ])
        .into_element(cx)
    };

    shadcn::FieldSet::new(ui::children![
        cx;
        field(cx, "Name", name),
        field(cx, "Username", username)
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let name = cx.local_model_keyed("name", || String::from("RTL user"));
    let username = cx.local_model_keyed("username", || String::from("@fret-user"));

    let name_model = name.clone();
    let username_model = username.clone();

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Dialog::new(open.clone())
            .children([
                shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                    shadcn::Button::new("Open RTL Dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-rtl-trigger"),
                )),
                shadcn::DialogPart::content_with(move |cx| {
                    let fields = profile_fields(cx, name_model.clone(), username_model.clone())
                        .into_element(cx);
                    shadcn::DialogContent::new([]).with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogTitle::new("RTL Profile").into_element(cx),
                                    shadcn::DialogDescription::new(
                                        "This example renders dialog layout in right-to-left direction.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            fields,
                            shadcn::DialogFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogClose::from_scope().build(
                                        cx,
                                        shadcn::Button::new("Cancel")
                                            .variant(shadcn::ButtonVariant::Outline),
                                    ),
                                    shadcn::DialogClose::from_scope()
                                        .build(cx, shadcn::Button::new("Save")),
                                ]
                            }),
                        ]
                    })
                    .test_id("ui-gallery-dialog-rtl-content")
                }),
            ])
            .into_element(cx)
    })
}
// endregion: example

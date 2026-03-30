pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
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
    let name = cx.local_model_keyed("name", || String::from("Pedro Duarte"));
    let username = cx.local_model_keyed("username", || String::from("@peduarte"));

    let name_model = name.clone();
    let username_model = username.clone();

    shadcn::Dialog::new(open.clone())
        .children([
            shadcn::DialogPart::trigger(shadcn::DialogTrigger::build(
                shadcn::Button::new("Open Dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .refine_layout(LayoutRefinement::default().min_w(Px(220.0)))
                    .test_id("ui-gallery-dialog-demo-trigger"),
            )),
            shadcn::DialogPart::content_with(move |cx| {
                let fields =
                    profile_fields(cx, name_model.clone(), username_model.clone()).into_element(cx);
                shadcn::DialogContent::new([])
                    .refine_layout(LayoutRefinement::default().max_w(Px(425.0)))
                    .with_children(cx, |cx| {
                        vec![
                            shadcn::DialogHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogTitle::new("Edit profile").into_element(cx),
                                    shadcn::DialogDescription::new(
                                        "Make changes to your profile here. Click save when you're done.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            fields,
                            shadcn::DialogFooter::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::DialogClose::from_scope()
                                        .build(
                                            cx,
                                            shadcn::Button::new("Cancel")
                                                .variant(shadcn::ButtonVariant::Outline),
                                        ),
                                    shadcn::Button::new("Save changes").into_element(cx),
                                ]
                            }),
                        ]
                    })
                    .test_id("ui-gallery-dialog-demo-content")
            }),
        ])
        .into_element(cx)
}
// endregion: example

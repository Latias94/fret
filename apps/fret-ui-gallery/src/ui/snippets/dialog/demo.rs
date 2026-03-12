pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn profile_fields<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: Model<String>,
    username: Model<String>,
) -> AnyElement {
    let field = |cx: &mut ElementContext<'_, H>, label: &'static str, model: Model<String>| {
        shadcn::Field::new([
            shadcn::FieldLabel::new(label).into_element(cx),
            shadcn::Input::new(model)
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ])
        .into_element(cx)
    };

    shadcn::FieldSet::new([field(cx, "Name", name), field(cx, "Username", username)])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let name = cx.local_model_keyed("name", || String::from("Pedro Duarte"));
    let username = cx.local_model_keyed("username", || String::from("@peduarte"));

    let trigger_open = open.clone();
    let save_open = open.clone();

    let name_model = name.clone();
    let username_model = username.clone();

    shadcn::Dialog::new(open.clone()).into_element(
        cx,
        move |cx| {
            ui::h_flex(|cx| {
                vec![
                    shadcn::Button::new("Open Dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().flex_1())
                        .test_id("ui-gallery-dialog-demo-trigger")
                        .toggle_model(trigger_open.clone())
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full().max_w(Px(520.0)))
            .into_element(cx)
        },
        move |cx| {
            shadcn::DialogContent::new([
                shadcn::DialogHeader::new([
                    shadcn::DialogTitle::new("Edit profile").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                profile_fields(cx, name_model.clone(), username_model.clone()),
                shadcn::DialogFooter::new([
                    shadcn::Button::new("Cancel")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open.clone())
                        .into_element(cx),
                    shadcn::Button::new("Save changes")
                        .toggle_model(save_open.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
            .test_id("ui-gallery-dialog-demo-content")
        },
    )
}
// endregion: example

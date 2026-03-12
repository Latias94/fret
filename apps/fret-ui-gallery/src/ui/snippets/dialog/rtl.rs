pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
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
    let name = cx.local_model_keyed("name", || String::from("RTL user"));
    let username = cx.local_model_keyed("username", || String::from("@fret-user"));

    let open_for_trigger = open.clone();
    let save_open = open.clone();
    let name_model = name.clone();
    let username_model = username.clone();

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::Dialog::new(open.clone()).into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open RTL Dialog")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-dialog-rtl-trigger")
                    .toggle_model(open_for_trigger.clone())
                    .into_element(cx)
            },
            |cx| {
                shadcn::DialogContent::new([
                    shadcn::DialogHeader::new([
                        shadcn::DialogTitle::new("RTL Profile").into_element(cx),
                        shadcn::DialogDescription::new(
                            "This example renders dialog layout in right-to-left direction.",
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
                        shadcn::Button::new("Save")
                            .toggle_model(save_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id("ui-gallery-dialog-rtl-content")
            },
        )
    })
}
// endregion: example

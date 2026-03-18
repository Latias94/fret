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
                shadcn::DialogContent::new(ui::children![
                    cx;
                    shadcn::DialogHeader::new(ui::children![
                        cx;
                        shadcn::DialogTitle::new("RTL Profile"),
                        shadcn::DialogDescription::new(
                            "This example renders dialog layout in right-to-left direction.",
                        )
                    ]),
                    profile_fields(cx, name_model.clone(), username_model.clone()),
                    shadcn::DialogFooter::new(ui::children![
                        cx;
                        shadcn::DialogClose::from_scope().build(
                            cx,
                            shadcn::Button::new("Cancel").variant(shadcn::ButtonVariant::Outline),
                        ),
                        shadcn::Button::new("Save").toggle_model(save_open.clone()),
                    ]),
                ])
                .into_element(cx)
                .test_id("ui-gallery-dialog-rtl-content")
            },
        )
    })
}
// endregion: example

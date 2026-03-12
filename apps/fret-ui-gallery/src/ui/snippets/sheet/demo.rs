pub const SOURCE: &str = include_str!("demo.rs");

// region: example
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

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.local_model_keyed("open", || false);
    let name = cx.local_model_keyed("name", || String::from("Pedro Duarte"));
    let username = cx.local_model_keyed("username", || String::from("@peduarte"));

    let trigger_open = open.clone();
    let save_open = open.clone();
    let close_open = open.clone();
    let name_model = name.clone();
    let username_model = username.clone();

    shadcn::Sheet::new(open.clone())
        .into_element(
            cx,
            |cx| {
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-sheet-demo-trigger")
                    .toggle_model(trigger_open.clone())
                    .into_element(cx)
            },
            |cx| {
                let fields = {
                    let fields = profile_fields(cx, name_model.clone(), username_model.clone())
                        .into_element(cx);
                    let props = decl_style::container_props(
                        Theme::global(&*cx.app),
                        ChromeRefinement::default().px(Space::N4),
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .min_h_0()
                            .flex_1(),
                    );
                    cx.container(props, move |_cx| vec![fields])
                };
                shadcn::SheetContent::new(ui::children![
                    cx;
                    shadcn::SheetHeader::new(ui::children![
                        cx;
                        shadcn::SheetTitle::new("Edit profile"),
                        shadcn::SheetDescription::new(
                            "Make changes to your profile here. Click save when you're done.",
                        )
                    ]),
                    fields,
                    shadcn::SheetFooter::new(ui::children![
                        cx;
                        shadcn::Button::new("Save changes")
                            .toggle_model(save_open.clone()),
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(close_open.clone()),
                    ]),
                ])
                .into_element(cx)
                .test_id("ui-gallery-sheet-demo-content")
            },
        )
        .test_id("ui-gallery-sheet-demo")
}
// endregion: example

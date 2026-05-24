pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn profile_fields<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: Model<String>,
    username: Model<String>,
) -> impl IntoUiElement<H> + use<H> {
    let field = |cx: &mut ElementContext<'_, H>,
                 label: &'static str,
                 input_test_id: &'static str,
                 model: Model<String>| {
        ui::v_flex(move |cx| {
            ui::children![
                cx;
                shadcn::Label::new(label),
                shadcn::Input::new(model)
                    .test_id(input_test_id)
                    .refine_layout(LayoutRefinement::default().w_full())
            ]
        })
        .gap(Space::N3)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
    };

    ui::v_flex(move |cx| {
        vec![
            field(cx, "Name", "ui-gallery-sheet-demo-name-input", name),
            field(
                cx,
                "Username",
                "ui-gallery-sheet-demo-username-input",
                username,
            ),
        ]
    })
    .gap(Space::N6)
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let name = cx.local_model_keyed("name", || String::from("Pedro Duarte"));
    let username = cx.local_model_keyed("username", || String::from("@peduarte"));

    let name_model = name.clone();
    let username_model = username.clone();

    shadcn::Sheet::new_controllable(cx, None, false)
        .children([
            shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(
                shadcn::Button::new("Open")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-sheet-demo-trigger"),
            )),
            shadcn::SheetPart::content_with(move |cx| {
                let fields = {
                    let fields = profile_fields(cx, name_model.clone(), username_model.clone())
                        .into_element(cx);
                    let props = decl_style::container_props(
                        Theme::global(&*cx.app),
                        ChromeRefinement::default().px(Space::N4),
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .flex_1(),
                    );
                    cx.container(props, move |_cx| vec![fields])
                        .test_id("ui-gallery-sheet-demo-body")
                };

                shadcn::SheetContent::new([]).with_children(cx, |cx| {
                    vec![
                        shadcn::SheetHeader::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::SheetTitle::new("Edit profile")
                                    .into_element(cx)
                                    .test_id("ui-gallery-sheet-demo-dialog-title"),
                                shadcn::SheetDescription::new(
                                    "Make changes to your profile here. Click save when you're done.",
                                )
                                .into_element(cx)
                                .test_id("ui-gallery-sheet-demo-dialog-description"),
                            ]
                        })
                        .test_id("ui-gallery-sheet-demo-header"),
                        fields,
                        shadcn::SheetFooter::new([]).with_children(cx, |cx| {
                            vec![
                                shadcn::Button::new("Save changes")
                                    .test_id("ui-gallery-sheet-demo-save")
                                    .into_element(cx),
                                shadcn::SheetClose::from_scope().build(
                                    cx,
                                    shadcn::Button::new("Close")
                                        .test_id("ui-gallery-sheet-demo-close")
                                        .variant(shadcn::ButtonVariant::Outline),
                                ),
                            ]
                        }),
                    ]
                })
                .test_id("ui-gallery-sheet-demo-panel")
            }),
        ])
        .into_element(cx)
        .test_id("ui-gallery-sheet-demo")
}
// endregion: example

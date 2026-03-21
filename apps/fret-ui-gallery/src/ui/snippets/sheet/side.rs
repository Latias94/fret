pub const SOURCE: &str = include_str!("side.rs");

// region: example
use fret::children::UiElementSinkExt;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let demo_name = cx.local_model_keyed("demo_name", || String::from("Pedro Duarte"));
    let demo_username = cx.local_model_keyed("demo_username", || String::from("@peduarte"));

    let side_sheet = |cx: &mut UiCx<'_>,
                      id: &'static str,
                      label: &'static str,
                      side: shadcn::SheetSide| {
        let name_model = demo_name.clone();
        let username_model = demo_username.clone();

        shadcn::Sheet::new_controllable(cx, None, false)
            .side(side)
            .children([
                shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                        .test_id(format!("ui-gallery-sheet-side-{id}-trigger")),
                )),
                shadcn::SheetPart::content(
                    shadcn::SheetContent::build(move |cx, out| {
                        let fields = ui::v_flex(|cx| {
                            vec![
                                shadcn::Field::new([
                                    shadcn::FieldLabel::new("Name").into_element(cx),
                                    shadcn::Input::new(name_model.clone())
                                        .refine_layout(LayoutRefinement::default().w_full())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                                shadcn::Field::new([
                                    shadcn::FieldLabel::new("Username").into_element(cx),
                                    shadcn::Input::new(username_model.clone())
                                        .refine_layout(LayoutRefinement::default().w_full())
                                        .into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        })
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx);

                        let fields = {
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

                        out.push_ui(
                            cx,
                            shadcn::SheetHeader::build(|cx, out| {
                                out.push_ui(cx, shadcn::SheetTitle::new("Edit profile"));
                                out.push_ui(
                                    cx,
                                    shadcn::SheetDescription::new(
                                        "Make changes to your profile here. Click save when you're done.",
                                    ),
                                );
                            }),
                        );
                        out.push(fields);
                        out.push_ui(
                            cx,
                            shadcn::SheetFooter::build(|cx, out| {
                                out.push_ui(
                                    cx,
                                    shadcn::Button::new("Save changes")
                                        .test_id(format!("ui-gallery-sheet-side-{id}-save")),
                                );
                                let cancel = shadcn::SheetClose::from_scope().build(
                                    cx,
                                    shadcn::Button::new("Cancel")
                                        .variant(shadcn::ButtonVariant::Outline),
                                );
                                out.push(cancel);
                            }),
                        );
                    })
                    .test_id(format!("ui-gallery-sheet-side-{id}-content")),
                ),
            ])
            .into_element(cx)
    };

    // Match upstream demo layout: a strict 2x2 grid of side triggers.
    ui::v_flex(|cx| {
        let row = |cx: &mut UiCx<'_>,
                   a: (&'static str, &'static str, shadcn::SheetSide),
                   b: (&'static str, &'static str, shadcn::SheetSide)| {
            let (id_a, label_a, side_a) = a;
            let (id_b, label_b, side_b) = b;
            ui::h_flex_build(|cx, out| {
                out.push(cx.keyed(id_a, |cx| side_sheet(cx, id_a, label_a, side_a)));
                out.push(cx.keyed(id_b, |cx| side_sheet(cx, id_b, label_b, side_b)));
            })
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .into_element(cx)
        };

        vec![
            row(
                cx,
                ("top", "top", shadcn::SheetSide::Top),
                ("right", "right", shadcn::SheetSide::Right),
            ),
            row(
                cx,
                ("bottom", "bottom", shadcn::SheetSide::Bottom),
                ("left", "left", shadcn::SheetSide::Left),
            ),
        ]
    })
    .gap(Space::N2)
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-sheet-side")
}
// endregion: example

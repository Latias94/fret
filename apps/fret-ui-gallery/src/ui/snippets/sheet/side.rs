pub const SOURCE: &str = include_str!("side.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
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

        let row = |cx: &mut UiCx<'_>,
                   label: &'static str,
                   model: Model<String>,
                   input_test_id: String| {
            let label_cell = ui::h_row(move |cx| ui::children![cx; ui::label(label)])
                .layout(LayoutRefinement::default().w_px(Px(96.0)).flex_shrink_0())
                .justify_end()
                .items_center()
                .into_element(cx);

            let input = shadcn::Input::new(model)
                .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                .test_id(input_test_id)
                .into_element(cx);

            ui::h_flex(move |_cx| vec![label_cell, input])
                .gap(Space::N4)
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx)
        };

        shadcn::Sheet::new_controllable(cx, None, false)
            .side(side)
            .children([
                shadcn::SheetPart::trigger(shadcn::SheetTrigger::build(
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                        .test_id(format!("ui-gallery-sheet-side-{id}-trigger")),
                )),
                shadcn::SheetPart::content_with(move |cx| {
                    let fields = ui::v_flex(|cx| {
                        vec![
                            row(
                                cx,
                                "Name",
                                name_model.clone(),
                                format!("ui-gallery-sheet-side-{id}-name-input"),
                            ),
                            row(
                                cx,
                                "Username",
                                username_model.clone(),
                                format!("ui-gallery-sheet-side-{id}-username-input"),
                            ),
                        ]
                    })
                    .gap(Space::N4)
                    .px_4()
                    .py_4()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx);

                    shadcn::SheetContent::new([]).with_children(cx, |cx| {
                        vec![
                            shadcn::SheetHeader::new([]).with_children(cx, |cx| {
                                vec![
                                    shadcn::SheetTitle::new("Edit profile").into_element(cx),
                                    shadcn::SheetDescription::new(
                                        "Make changes to your profile here. Click save when you're done.",
                                    )
                                    .into_element(cx),
                                ]
                            }),
                            fields,
                            shadcn::SheetFooter::new([]).with_children(cx, |cx| {
                                vec![shadcn::SheetClose::from_scope().build(
                                    cx,
                                    shadcn::Button::new("Save changes")
                                        .test_id(format!("ui-gallery-sheet-side-{id}-save")),
                                )]
                            }),
                        ]
                    })
                    .test_id(format!("ui-gallery-sheet-side-{id}-content"))
                }),
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

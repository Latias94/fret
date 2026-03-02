pub const SOURCE: &str = include_str!("side.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default, Clone)]
struct Models {
    demo_name: Option<Model<String>>,
    demo_username: Option<Model<String>>,
    side_top_open: Option<Model<bool>>,
    side_right_open: Option<Model<bool>>,
    side_bottom_open: Option<Model<bool>>,
    side_left_open: Option<Model<bool>>,
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let state = cx.with_state(Models::default, |st| st.clone());

    let demo_name = match state.demo_name {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("Pedro Duarte"));
            cx.with_state(Models::default, |st| st.demo_name = Some(model.clone()));
            model
        }
    };

    let demo_username = match state.demo_username {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::from("@peduarte"));
            cx.with_state(Models::default, |st| st.demo_username = Some(model.clone()));
            model
        }
    };

    let side_top_open = match state.side_top_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.side_top_open = Some(model.clone()));
            model
        }
    };

    let side_right_open = match state.side_right_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.side_right_open = Some(model.clone())
            });
            model
        }
    };

    let side_bottom_open = match state.side_bottom_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.side_bottom_open = Some(model.clone())
            });
            model
        }
    };

    let side_left_open = match state.side_left_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.side_left_open = Some(model.clone())
            });
            model
        }
    };

    let side_sheet = |cx: &mut ElementContext<'_, H>,
                      id: &'static str,
                      label: &'static str,
                      side: shadcn::SheetSide,
                      open_model: Model<bool>| {
        let trigger_open = open_model.clone();
        let save_open = open_model.clone();

        let sheet = shadcn::Sheet::new(open_model).side(side);
        let sheet = if matches!(side, shadcn::SheetSide::Left | shadcn::SheetSide::Right) {
            sheet.size(Px(420.0))
        } else {
            // Upstream shadcn uses `h-auto` for top/bottom sheets; keep them auto-sized so the
            // footer actions remain fully visible on typical viewport heights.
            sheet
        };

        sheet.into_element(
            cx,
            |cx| {
                shadcn::Button::new(label)
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(trigger_open.clone())
                    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                    .test_id(format!("ui-gallery-sheet-side-{id}-trigger"))
                    .into_element(cx)
            },
            |cx| {
                let fields = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full().min_w_0()),
                    |cx| {
                        vec![
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Name").into_element(cx),
                                shadcn::Input::new(demo_name.clone())
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                            shadcn::Field::new([
                                shadcn::FieldLabel::new("Username").into_element(cx),
                                shadcn::Input::new(demo_username.clone())
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            ])
                            .into_element(cx),
                        ]
                    },
                );

                shadcn::SheetContent::new([
                    shadcn::SheetHeader::new([
                        shadcn::SheetTitle::new("Edit profile").into_element(cx),
                        shadcn::SheetDescription::new(
                            "Make changes to your profile here. Click save when you're done.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    fields,
                    shadcn::SheetFooter::new([shadcn::Button::new("Save changes")
                        .toggle_model(save_open.clone())
                        .test_id(format!("ui-gallery-sheet-side-{id}-save"))
                        .into_element(cx)])
                    .into_element(cx),
                ])
                .into_element(cx)
                .test_id(format!("ui-gallery-sheet-side-{id}-content"))
            },
        )
    };

    // Match upstream demo layout: a strict 2x2 grid of side triggers.
    stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            let row =
                |cx: &mut ElementContext<'_, H>,
                 a: (&'static str, &'static str, shadcn::SheetSide, Model<bool>),
                 b: (&'static str, &'static str, shadcn::SheetSide, Model<bool>)| {
                    let (id_a, label_a, side_a, open_a) = a;
                    let (id_b, label_b, side_b, open_b) = b;
                    stack::hstack_build(
                        cx,
                        stack::HStackProps::default()
                            .gap(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default().w_full().min_w_0()),
                        |cx, out| {
                            out.push(cx.keyed(id_a, |cx| {
                                side_sheet(cx, id_a, label_a, side_a, open_a.clone())
                            }));
                            out.push(cx.keyed(id_b, |cx| {
                                side_sheet(cx, id_b, label_b, side_b, open_b.clone())
                            }));
                        },
                    )
                };

            vec![
                row(
                    cx,
                    ("top", "top", shadcn::SheetSide::Top, side_top_open.clone()),
                    (
                        "right",
                        "right",
                        shadcn::SheetSide::Right,
                        side_right_open.clone(),
                    ),
                ),
                row(
                    cx,
                    (
                        "bottom",
                        "bottom",
                        shadcn::SheetSide::Bottom,
                        side_bottom_open.clone(),
                    ),
                    (
                        "left",
                        "left",
                        shadcn::SheetSide::Left,
                        side_left_open.clone(),
                    ),
                ),
            ]
        },
    )
    .test_id("ui-gallery-sheet-side")
}
// endregion: example

use super::super::super::super::super::*;

pub(in crate::ui) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};

    #[derive(Default)]
    struct SpinnerModels {
        input_value: Option<Model<String>>,
        textarea_value: Option<Model<String>>,
    }

    let (input_value, textarea_value) = cx.with_state(SpinnerModels::default, |st| {
        (st.input_value.clone(), st.textarea_value.clone())
    });
    let (input_value, textarea_value) = match (input_value, textarea_value) {
        (Some(input_value), Some(textarea_value)) => (input_value, textarea_value),
        _ => {
            let input_value = cx.app.models_mut().insert(String::new());
            let textarea_value = cx.app.models_mut().insert(String::new());
            cx.with_state(SpinnerModels::default, |st| {
                st.input_value = Some(input_value.clone());
                st.textarea_value = Some(textarea_value.clone());
            });
            (input_value, textarea_value)
        }
    };

    let theme = Theme::global(&*cx.app).snapshot();

    let sizes = {
        let small = shadcn::Spinner::new()
            .into_element(cx)
            .test_id("ui-gallery-spinner-sizes-small");
        let large = shadcn::Spinner::new()
            .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
            .into_element(cx)
            .test_id("ui-gallery-spinner-sizes-large");

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N6, |_cx| vec![small, large])
            .test_id("ui-gallery-spinner-sizes")
    };

    let buttons = {
        let spinner = |cx: &mut ElementContext<'_, App>| shadcn::Spinner::new().into_element(cx);

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("Submit")
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("Disabled")
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("Small")
                    .size(shadcn::ButtonSize::Sm)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx)
                    .attach_semantics(SemanticsDecoration::default().label("Loading..."))
                    .test_id("ui-gallery-spinner-button-icon-only"),
                shadcn::Button::new("Remove")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .disabled(true)
                    .children([spinner(cx)])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-spinner-buttons")
    };

    let badges = {
        let spinner = |cx: &mut ElementContext<'_, App>| shadcn::Spinner::new().into_element(cx);

        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Badge::new("Badge")
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Badge::new("Badge")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Badge::new("Badge")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .children([spinner(cx)])
                    .into_element(cx),
                shadcn::Badge::new("Badge")
                    .variant(shadcn::BadgeVariant::Outline)
                    .children([spinner(cx)])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-spinner-badges")
    };

    let input_group = {
        let field = shadcn::Field::new([
            shadcn::FieldLabel::new("Input Group").into_element(cx),
            shadcn::InputGroup::new(input_value.clone())
                .a11y_label("Input group")
                .trailing([shadcn::Spinner::new().into_element(cx)])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-input-group");

        field
    };

    let empty = {
        let actions = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Create project").into_element(cx),
                    shadcn::Button::new("Import project")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                ]
            },
        );

        let learn_more = shadcn::Button::new("Learn more")
            .variant(shadcn::ButtonVariant::Link)
            .trailing_icon(fret_icons::IconId::new_static("lucide.arrow-right"))
            .into_element(cx);

        shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("No projects yet").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "You haven't created any projects yet. Get started by creating your first project.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([actions])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
            learn_more,
        ])
        .refine_layout(LayoutRefinement::default().w_full().min_h(Px(520.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-empty")
    };

    let extras = {
        let icon_row = doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N4, |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new()
                    .icon(fret_icons::ids::ui::SETTINGS)
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-spinner-extras-custom-icon");

        let input = shadcn::InputGroup::new(input_value)
            .a11y_label("Send a message")
            .trailing([shadcn::Spinner::new().into_element(cx)])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let textarea = shadcn::InputGroup::new(textarea_value)
            .textarea()
            .a11y_label("Send a message textarea")
            .block_end([stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        shadcn::Spinner::new().into_element(cx),
                        shadcn::typography::muted(cx, "Validating..."),
                        shadcn::InputGroupButton::new("")
                            .a11y_label("Send")
                            .size(shadcn::InputGroupButtonSize::IconSm)
                            .icon(fret_icons::IconId::new_static("lucide.arrow-up"))
                            .into_element(cx),
                    ]
                },
            )
            .test_id("ui-gallery-spinner-extras-textarea-actions")])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().max_w(Px(520.0))),
            |cx| {
                vec![
                    shadcn::typography::muted(
                        cx,
                        "Extras are Fret-specific demos and regression gates (not part of upstream shadcn SpinnerDemo).",
                    ),
                    icon_row,
                    input,
                    textarea,
                ]
            },
        )
        .test_id("ui-gallery-spinner-extras")
    };

    let rtl = {
        let rtl_demo = doc_layout::rtl(cx, |cx| {
            shadcn::Item::new([
                shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
                shadcn::ItemContent::new([
                    shadcn::ItemTitle::new("Processing payment...").into_element(cx)
                ])
                .into_element(cx),
                shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
            ])
            .variant(shadcn::ItemVariant::Muted)
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(360.0)))
            .into_element(cx)
        })
        .test_id("ui-gallery-spinner-rtl");

        rtl_demo
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Spinner demo (new-york-v4).",
            "The `Empty` section is not pixel-perfect (no anchor-as-child), but preserves the structure and semantics intent.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("An indicator that can be used to show a loading state."),
        vec![
            DocSection::new("Sizes", sizes)
                .test_id_prefix("ui-gallery-spinner-sizes")
                .code(
                    "rust",
                    r#"shadcn::Spinner::new()
    .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
    .into_element(cx);"#,
                ),
            DocSection::new("Buttons", buttons)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-spinner-buttons")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Submit")
    .children([shadcn::Spinner::new().into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Badges", badges)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-spinner-badges")
                .code(
                    "rust",
                    r#"shadcn::Badge::new("Badge")
    .variant(shadcn::BadgeVariant::Outline)
    .children([shadcn::Spinner::new().into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Input Group", input_group)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-spinner-input-group")
                .code(
                    "rust",
                    r#"shadcn::InputGroup::new(model)
    .a11y_label("Input group")
    .trailing([shadcn::Spinner::new().into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Empty", empty)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-spinner-empty")
                .code(
                    "rust",
                    r#"shadcn::Empty::new([
    shadcn::empty::EmptyHeader::new([
        shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
        shadcn::empty::EmptyTitle::new("No projects yet").into_element(cx),
    ]).into_element(cx),
]).into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-spinner-rtl")
                .code(
                    "rust",
                    r#"doc_layout::rtl(cx, |cx| {
    shadcn::Spinner::new().into_element(cx)
});"#,
                ),
            DocSection::new("Extras", extras)
                .no_shell()
                .test_id_prefix("ui-gallery-spinner-extras")
                .code(
                    "rust",
                    r#"// Extras are Fret-specific compositions that exercise token + layout behavior.
stack::vstack(cx, props, |_cx| vec![/* ... */]);"#,
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-spinner-notes"),
        ],
    );

    vec![body]
}

use super::super::super::super::super::*;

pub(in crate::ui) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
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

    let centered = |cx: &mut ElementContext<'_, App>, body: AnyElement| {
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify_center(),
            move |_cx| [body],
        )
    };

    let section = |cx: &mut ElementContext<'_, App>, title: &'static str, body: AnyElement| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .items_start()
                .layout(LayoutRefinement::default().w_full()),
            move |cx| vec![shadcn::typography::h4(cx, title), body],
        )
    };

    let shell = |cx: &mut ElementContext<'_, App>, layout: LayoutRefinement, body: AnyElement| {
        let props = cx.with_theme(|theme| {
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .p(Space::N4),
                layout,
            )
        });
        cx.container(props, move |_cx| [body])
    };

    let demo = {
        let item = shadcn::Item::new([
            shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)]).into_element(cx),
            shadcn::ItemContent::new([
                shadcn::ItemTitle::new("Processing payment...").into_element(cx)
            ])
            .into_element(cx),
            shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
        ])
        .variant(shadcn::ItemVariant::Muted)
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-demo");
        let body = centered(cx, item);
        section(cx, "Demo", body)
    };

    let custom = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new().into_element(cx),
                    shadcn::Spinner::new()
                        .icon(fret_icons::ids::ui::SETTINGS)
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-custom");
        let body = centered(cx, row);
        section(cx, "Customization", body)
    };

    let size = {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N6).items_center(),
            |cx| {
                vec![
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(12.0)).h_px(Px(12.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(16.0)).h_px(Px(16.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(24.0)).h_px(Px(24.0)))
                        .into_element(cx),
                    shadcn::Spinner::new()
                        .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-size");
        let body = centered(cx, row);
        section(cx, "Size", body)
    };

    let button = {
        let group = stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Loading...")
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Button::new("Please wait")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Button::new("Processing")
                        .variant(shadcn::ButtonVariant::Secondary)
                        .size(shadcn::ButtonSize::Sm)
                        .disabled(true)
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-button");
        let body = centered(cx, group);
        section(cx, "Button", body)
    };

    let badge = {
        let (secondary_fg, outline_fg) = cx.with_theme(|theme| {
            (
                ColorRef::Color(theme.color_required("secondary-foreground")),
                ColorRef::Color(theme.color_required("foreground")),
            )
        });

        let row = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            |cx| {
                vec![
                    shadcn::Badge::new("Syncing")
                        .children([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::Badge::new("Updating")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .children([shadcn::Spinner::new()
                            .color(secondary_fg.clone())
                            .into_element(cx)])
                        .into_element(cx),
                    shadcn::Badge::new("Processing")
                        .variant(shadcn::BadgeVariant::Outline)
                        .children([shadcn::Spinner::new()
                            .color(outline_fg.clone())
                            .into_element(cx)])
                        .into_element(cx),
                ]
            },
        )
        .test_id("ui-gallery-spinner-badge");
        let body = centered(cx, row);
        section(cx, "Badge", body)
    };

    let input_group = {
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
                            .size(shadcn::InputGroupButtonSize::IconSm)
                            .children([shadcn::icon::icon(
                                cx,
                                fret_icons::IconId::new_static("lucide.arrow-up"),
                            )])
                            .into_element(cx),
                    ]
                },
            )])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx);

        let group = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(LayoutRefinement::default().w_full()),
            |_cx| vec![input, textarea],
        );

        let card = shell(
            cx,
            LayoutRefinement::default().w_full().max_w(Px(480.0)),
            group,
        )
        .test_id("ui-gallery-spinner-input-group");

        let body = centered(cx, card);
        section(cx, "Input Group", body)
    };

    let empty = {
        let card = shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyMedia::new([shadcn::Spinner::new().into_element(cx)])
                    .variant(shadcn::empty::EmptyMediaVariant::Icon)
                    .into_element(cx),
                shadcn::empty::EmptyTitle::new("Processing your request").into_element(cx),
                shadcn::empty::EmptyDescription::new(
                    "Please wait while we process your request. Do not refresh the page.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Outline)
                .size(shadcn::ButtonSize::Sm)
                .into_element(cx)])
            .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(560.0)))
        .into_element(cx)
        .test_id("ui-gallery-spinner-empty");

        let body = centered(cx, card);
        section(cx, "Empty", body)
    };

    let rtl = {
        let body = fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                shadcn::Item::new([
                    shadcn::ItemMedia::new([shadcn::Spinner::new().into_element(cx)])
                        .into_element(cx),
                    shadcn::ItemContent::new([
                        shadcn::ItemTitle::new("Processing payment...").into_element(cx)
                    ])
                    .into_element(cx),
                    shadcn::ItemActions::new([cx.text("$100.00")]).into_element(cx),
                ])
                .variant(shadcn::ItemVariant::Muted)
                .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
                .into_element(cx)
            },
        )
        .test_id("ui-gallery-spinner-rtl");

        let centered_body = centered(cx, body);
        section(cx, "RTL", centered_body)
    };

    vec![
        cx.text("An indicator that can be used to show a loading state."),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N6), |_cx| {
            vec![demo, custom, size, button, badge, input_group, empty, rtl]
        }),
    ]
}

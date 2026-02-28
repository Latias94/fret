use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let variants = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("Default")
                    .test_id("ui-gallery-button-variant-default")
                    .into_element(cx),
                shadcn::Button::new("Secondary")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-button-variant-secondary")
                    .into_element(cx),
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .test_id("ui-gallery-button-variant-destructive")
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-button-variant-outline")
                    .into_element(cx),
                shadcn::Button::new("Ghost")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .test_id("ui-gallery-button-variant-ghost")
                    .into_element(cx),
                shadcn::Button::new("Link")
                    .variant(shadcn::ButtonVariant::Link)
                    .test_id("ui-gallery-button-variant-link")
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-variants")
    };

    let link_render = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("Dashboard")
                    .render(shadcn::ButtonRender::Link {
                        href: Arc::<str>::from("https://example.com/dashboard"),
                        target: None,
                        rel: None,
                    })
                    // Keep the gallery deterministic: demonstrate link semantics without opening
                    // the browser during scripted runs.
                    .on_click(CMD_APP_OPEN)
                    .test_id("ui-gallery-button-render-link")
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-render-link-row")
    };

    let size = {
        let row = |cx: &mut ElementContext<'_, App>,
                   label: &'static str,
                   text_size: shadcn::ButtonSize,
                   icon_size: shadcn::ButtonSize| {
            doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
                vec![
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(text_size)
                        .into_element(cx),
                    shadcn::Button::new("")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(icon_size)
                        .a11y_label("Open")
                        .icon(fret_icons::IconId::new_static("lucide.arrow-up-right"))
                        .into_element(cx),
                ]
            })
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .items_start()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            |cx| {
                vec![
                    row(
                        cx,
                        "Small",
                        shadcn::ButtonSize::Sm,
                        shadcn::ButtonSize::IconSm,
                    ),
                    row(
                        cx,
                        "Default",
                        shadcn::ButtonSize::Default,
                        shadcn::ButtonSize::Icon,
                    ),
                    row(
                        cx,
                        "Large",
                        shadcn::ButtonSize::Lg,
                        shadcn::ButtonSize::IconLg,
                    ),
                ]
            },
        )
        .test_id("ui-gallery-button-size")
    };

    let icon_only = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconSm)
                    .a11y_label("Open")
                    .icon(fret_icons::IconId::new_static("lucide.arrow-up-right"))
                    .into_element(cx),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .a11y_label("Open")
                    .icon(fret_icons::IconId::new_static("lucide.arrow-up-right"))
                    .into_element(cx),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconLg)
                    .a11y_label("Open")
                    .icon(fret_icons::IconId::new_static("lucide.arrow-up-right"))
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-icon-only")
    };

    let with_icon = {
        shadcn::Button::new("New Branch")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .leading_icon(fret_icons::IconId::new_static("lucide.git-branch"))
            .test_id("ui-gallery-button-with-icon")
            .into_element(cx)
    };

    let loading = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                // Upstream: `registry/new-york-v4/examples/button-loading.tsx`.
                shadcn::Button::new("Submit")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .disabled(true)
                    .test_id("ui-gallery-button-loading-submit")
                    .children([
                        shadcn::Spinner::new().into_element(cx),
                        ui::text(cx, "Submit")
                            .font_medium()
                            .nowrap()
                            .into_element(cx),
                    ])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-loading")
    };

    let rounded = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("Rounded")
                    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                    .test_id("ui-gallery-button-rounded")
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-rounded-row")
    };

    let button_group = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::ButtonGroup::new(
                    [
                        shadcn::Button::new("Left").variant(shadcn::ButtonVariant::Outline),
                        shadcn::Button::new("Middle").variant(shadcn::ButtonVariant::Outline),
                        shadcn::Button::new("Right").variant(shadcn::ButtonVariant::Outline),
                    ]
                    .into_iter()
                    .map(Into::into),
                )
                .a11y_label("Button group")
                .into_element(cx)
                .test_id("ui-gallery-button-button-group"),
            ]
        })
        .test_id("ui-gallery-button-button-group-row")
    };

    let rtl = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![fret_ui_kit::primitives::direction::with_direction_provider(
                cx,
                fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
                |cx| {
                    stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .gap_x(Space::N2)
                            .items_center()
                            .layout(LayoutRefinement::default().w_full()),
                        |cx| {
                            vec![
                                shadcn::Button::new("Default")
                                    .test_id("ui-gallery-button-rtl-default")
                                    .into_element(cx),
                                shadcn::Button::new("Secondary")
                                    .variant(shadcn::ButtonVariant::Secondary)
                                    .test_id("ui-gallery-button-rtl-secondary")
                                    .into_element(cx),
                                shadcn::Button::new("Destructive")
                                    .variant(shadcn::ButtonVariant::Destructive)
                                    .test_id("ui-gallery-button-rtl-destructive")
                                    .into_element(cx),
                                shadcn::Button::new("Back")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .leading_icon(fret_icons::IconId::new_static(
                                        "lucide.arrow-left",
                                    ))
                                    .test_id("ui-gallery-button-rtl-back")
                                    .into_element(cx),
                                shadcn::Button::new("Next")
                                    .variant(shadcn::ButtonVariant::Outline)
                                    .trailing_icon(fret_icons::IconId::new_static(
                                        "lucide.arrow-right",
                                    ))
                                    .test_id("ui-gallery-button-rtl-next")
                                    .into_element(cx),
                            ]
                        },
                    )
                    .test_id("ui-gallery-button-rtl-row-inner")
                },
            )]
        })
        .test_id("ui-gallery-button-rtl-row")
    };

    let notes = doc_layout::notes(
        cx,
        [
            "Preview aims to match shadcn Button docs order so you can visually compare variants quickly.",
            "Prefer icon-only buttons to use explicit `ButtonSize::Icon*` to keep square chrome.",
            "For long-running actions, combine a disabled button with a spinner + label.",
            "Use `ButtonRender::Link` when you want link semantics (`role=link`, Enter-only activation) on the pressable root.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Button docs (plus a compact variants row and a deterministic link render example).",
        ),
        vec![
            DocSection::new("Variants", variants)
                .description("Default shadcn button variants.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Secondary")
    .variant(shadcn::ButtonVariant::Secondary)
    .into_element(cx);"#,
                ),
            DocSection::new("Link (render)", link_render)
                .description(
                    "Render the button with link semantics (shadcn `asChild`-style composition).",
                )
                .code(
                    "rust",
                    r#"shadcn::Button::new("Dashboard")
    .render(shadcn::ButtonRender::Link { href: Arc::from("..."), target: None, rel: None })
    .into_element(cx);"#,
                ),
            DocSection::new("Size", size)
                .description("Text and icon-only sizes.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Small")
    .size(shadcn::ButtonSize::Sm)
    .into_element(cx);"#,
                ),
            DocSection::new("Icon", icon_only)
                .description("Icon-only buttons.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("")
    .a11y_label("Open")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Icon)
    .icon(fret_icons::IconId::new_static("lucide.arrow-up-right"))
    .into_element(cx);"#,
                ),
            DocSection::new("With Icon", with_icon)
                .description("Compose an icon + text label.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("New Branch")
    .variant(shadcn::ButtonVariant::Outline)
    .leading_icon(fret_icons::IconId::new_static("lucide.git-branch"))
    .into_element(cx);"#,
                ),
            DocSection::new("Loading", loading)
                .description("Spinner + label for in-flight actions.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Submit")
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Sm)
    .disabled(true)
    .children([
        shadcn::Spinner::new().into_element(cx),
        ui::text(cx, "Submit").into_element(cx),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Rounded", rounded)
                .description("Use a fully-rounded chrome for pill-shaped buttons.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Rounded")
    .refine_style(ChromeRefinement::default().rounded(Radius::Full))
    .into_element(cx);"#,
                ),
            DocSection::new("Button Group", button_group)
                .description("A grouped set of buttons with shared borders and radii.")
                .code(
                    "rust",
                    r#"shadcn::ButtonGroup::new([
    shadcn::Button::new("Left").variant(shadcn::ButtonVariant::Outline),
    shadcn::Button::new("Right").variant(shadcn::ButtonVariant::Outline),
])
.into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Button layout should work under an RTL direction provider.")
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| {
        shadcn::Button::new("Back")
            .variant(shadcn::ButtonVariant::Outline)
            .leading_icon(fret_icons::IconId::new_static("lucide.arrow-left"))
            .into_element(cx)
    },
)"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-button")]
}

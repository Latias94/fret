use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).snapshot();

    let outline_fg = ColorRef::Color(theme.color_token("foreground"));

    let icon = |cx: &mut ElementContext<'_, App>, name: &'static str, fg: ColorRef| {
        shadcn::icon::icon_with(cx, fret_icons::IconId::new_static(name), None, Some(fg))
    };

    let variants = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("Default").into_element(cx),
                shadcn::Button::new("Secondary")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .into_element(cx),
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Ghost")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx),
                shadcn::Button::new("Link")
                    .variant(shadcn::ButtonVariant::Link)
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
                        .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
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
                    .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                    .into_element(cx),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Icon)
                    .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                    .into_element(cx),
                shadcn::Button::new("")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::IconLg)
                    .children([icon(cx, "lucide.arrow-up-right", outline_fg.clone())])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-icon-only")
    };

    let with_icon = {
        shadcn::Button::new("New Branch")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .children([
                icon(cx, "lucide.git-branch", outline_fg.clone())
                    .test_id("ui-gallery-button-with-icon-icon"),
                ui::text(cx, "New Branch")
                    .font_medium()
                    .nowrap()
                    .text_color(outline_fg.clone())
                    .into_element(cx)
                    .test_id("ui-gallery-button-with-icon-label"),
            ])
            .into_element(cx)
            .test_id("ui-gallery-button-with-icon")
    };

    let loading = {
        doc_layout::wrap_controls_row_snapshot(cx, &theme, Space::N2, |cx| {
            vec![
                shadcn::Button::new("Generating")
                    .variant(shadcn::ButtonVariant::Outline)
                    .disabled(true)
                    .children([
                        shadcn::Spinner::new()
                            .color(outline_fg.clone())
                            .into_element(cx),
                        ui::text(cx, "Generating")
                            .font_medium()
                            .nowrap()
                            .text_color(outline_fg.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx),
                shadcn::Button::new("Deleting")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .disabled(true)
                    .children([
                        shadcn::Spinner::new().into_element(cx),
                        ui::text(cx, "Deleting")
                            .font_medium()
                            .nowrap()
                            .into_element(cx),
                    ])
                    .into_element(cx),
            ]
        })
        .test_id("ui-gallery-button-loading")
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
        Some("Preview follows shadcn Button docs order: Variants, Size, Icon, With Icon, Loading."),
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
                .description("Render the button with link semantics (shadcn `asChild`-style composition).")
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
    .variant(shadcn::ButtonVariant::Outline)
    .size(shadcn::ButtonSize::Icon)
    .children([shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.arrow-up-right"))])
    .into_element(cx);"#,
                ),
            DocSection::new("With Icon", with_icon)
                .description("Compose an icon + text label.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("New Branch")
    .variant(shadcn::ButtonVariant::Outline)
    .children([shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.git-branch"))])
    .into_element(cx);"#,
                ),
            DocSection::new("Loading", loading)
                .description("Spinner + label for in-flight actions.")
                .code(
                    "rust",
                    r#"shadcn::Button::new("Generating")
    .disabled(true)
    .children([shadcn::Spinner::new().into_element(cx)])
    .into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-button")]
}

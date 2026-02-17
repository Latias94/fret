use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Sm)
                    .a11y_label("Toggle bookmark")
                    .children([
                        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bookmark")),
                        cx.text("Bookmark"),
                    ])
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-toggle-demo");

    let outline = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .a11y_label("Toggle italic")
                    .children([
                        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.italic")),
                        cx.text("Italic"),
                    ])
                    .into_element(cx),
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .a11y_label("Toggle bold")
                    .children([
                        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bold")),
                        cx.text("Bold"),
                    ])
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-toggle-outline");

    let with_text = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .a11y_label("Toggle italic with text")
                    .children([
                        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.italic")),
                        cx.text("Italic"),
                    ])
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-toggle-with-text");

    let size = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Sm)
                    .a11y_label("Toggle small")
                    .label("Small")
                    .into_element(cx),
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Default)
                    .a11y_label("Toggle default")
                    .label("Default")
                    .into_element(cx),
                shadcn::Toggle::uncontrolled(false)
                    .variant(shadcn::ToggleVariant::Outline)
                    .size(shadcn::ToggleSize::Lg)
                    .a11y_label("Toggle large")
                    .label("Large")
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-toggle-size");

    let disabled = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Toggle::uncontrolled(false)
                    .disabled(true)
                    .a11y_label("Toggle disabled")
                    .label("Disabled")
                    .into_element(cx),
                shadcn::Toggle::uncontrolled(false)
                    .disabled(true)
                    .variant(shadcn::ToggleVariant::Outline)
                    .a11y_label("Toggle disabled outline")
                    .label("Disabled")
                    .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-toggle-disabled");

    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::Toggle::uncontrolled(false)
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .a11y_label("Toggle bookmark rtl")
            .children([
                shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bookmark")),
                cx.text("Bookmark"),
            ])
            .into_element(cx)
    })
    .test_id("ui-gallery-toggle-rtl");

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/toggle.rs` and `ecosystem/fret-ui-shadcn/src/toggle_group.rs`.",
            "Use Outline when toggle sits in dense toolbars and needs stronger boundaries.",
            "Prefer icon + short text labels so state remains understandable in compact layouts.",
            "Keep `a11y_label` explicit for icon-heavy toggles to improve accessibility tree quality.",
            "For quick keyboard validation, tab through toggles and verify pressed visual parity.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Toggle docs order: Demo, Outline, With Text, Size, Disabled, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A small outline toggle with an icon + label.")
                .max_w(Px(480.0))
                .code(
                    "rust",
                    r#"shadcn::Toggle::uncontrolled(false)
    .variant(shadcn::ToggleVariant::Outline)
    .size(shadcn::ToggleSize::Sm)
    .a11y_label("Toggle bookmark")
    .children([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bookmark")),
        cx.text("Bookmark"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Outline", outline)
                .description("Outline variant for dense toolbars.")
                .max_w(Px(480.0))
                .code(
                    "rust",
                    r#"shadcn::Toggle::uncontrolled(false)
    .variant(shadcn::ToggleVariant::Outline)
    .a11y_label("Toggle bold")
    .children([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bold")),
        cx.text("Bold"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("With Text", with_text)
                .description("Default variant with icon + text.")
                .max_w(Px(480.0))
                .code(
                    "rust",
                    r#"shadcn::Toggle::uncontrolled(false)
    .a11y_label("Toggle italic with text")
    .children([
        shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.italic")),
        cx.text("Italic"),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Size", size)
                .description("Size presets: Sm / Default / Lg.")
                .max_w(Px(480.0))
                .code(
                    "rust",
                    r#"stack::hstack(cx, stack::HStackProps::default().gap(Space::N2).items_center(), |cx| {
    vec![
        shadcn::Toggle::uncontrolled(false)
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Sm)
            .label("Small")
            .into_element(cx),
        shadcn::Toggle::uncontrolled(false)
            .variant(shadcn::ToggleVariant::Outline)
            .size(shadcn::ToggleSize::Lg)
            .label("Large")
            .into_element(cx),
    ]
})
.into_element(cx);"#,
                ),
            DocSection::new("Disabled", disabled)
                .description("Disabled toggles remain readable and non-interactive.")
                .max_w(Px(480.0))
                .code(
                    "rust",
                    r#"shadcn::Toggle::uncontrolled(false)
    .disabled(true)
    .variant(shadcn::ToggleVariant::Outline)
    .a11y_label("Toggle disabled")
    .label("Disabled")
    .into_element(cx);"#,
                ),
            DocSection::new("RTL", rtl)
                .description("Toggle content order and alignment under RTL.")
                .max_w(Px(480.0))
                .code(
                    "rust",
                    r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::Toggle::uncontrolled(false).children([cx.text("Bookmark")]).into_element(cx),
);"#,
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and accessibility notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-toggle")]
}

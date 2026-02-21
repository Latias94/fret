use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use fret_ui_kit::declarative::style as decl_style;

pub(super) fn preview_icons(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_icons::ids;

    let icon_cell =
        |cx: &mut ElementContext<'_, App>, label: &str, icon_id: IconId| -> AnyElement {
            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                },
            );

            let theme = Theme::global(&*cx.app);
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .rounded(Radius::Md)
                        .border_1()
                        .p(Space::N3),
                    LayoutRefinement::default().w_full(),
                ),
                |_cx| [row],
            )
        };

    let grid = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                icon_cell(cx, "ui.search", ids::ui::SEARCH),
                icon_cell(cx, "ui.settings", ids::ui::SETTINGS),
                icon_cell(cx, "ui.chevron.right", ids::ui::CHEVRON_RIGHT),
                icon_cell(cx, "ui.close", ids::ui::CLOSE),
                icon_cell(
                    cx,
                    "lucide.loader-circle",
                    IconId::new_static("lucide.loader-circle"),
                ),
            ]
        },
    )
    .test_id("ui-gallery-icons-grid");

    let spinner_row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new().speed(0.0).into_element(cx),
                cx.text("Spinner (animated / static)"),
            ]
        },
    )
    .test_id("ui-gallery-icons-spinner-row");

    let notes = doc_layout::notes(
        cx,
        [
            "Prefer stable icon IDs (e.g. `lucide.search`) so demos remain predictable across updates.",
            "Icon size should be explicit in docs to avoid token drift.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Sample icons and spinners used across the gallery."),
        vec![
            DocSection::new("Icons", grid)
                .max_w(Px(980.0))
                .description("Icons rendered via `fret_icons` IDs.")
                .code(
                    "rust",
                    r#"shadcn::icon::icon_with(
    cx,
    fret_icons::IconId::new_static("lucide.loader-circle"),
    Some(Px(16.0)),
    None,
);"#,
                ),
            DocSection::new("Spinner", spinner_row)
                .description("Spinner can be animated or static.")
                .code(
                    "rust",
                    r#"shadcn::Spinner::new().into_element(cx);
shadcn::Spinner::new().speed(0.0).into_element(cx);"#,
                ),
            DocSection::new("Notes", notes).description("Usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-icons")]
}

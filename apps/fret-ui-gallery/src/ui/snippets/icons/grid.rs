pub const SOURCE: &str = include_str!("grid.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    use fret_icons::ids;

    let icon_cell = |cx: &mut ElementContext<'_, H>,
                     label: &'static str,
                     icon_id: fret_icons::IconId|
     -> AnyElement {
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            |cx| {
                vec![
                    shadcn::icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
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

    stack::vstack(
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
                    fret_icons::IconId::new_static("lucide.loader-circle"),
                ),
            ]
        },
    )
    .test_id("ui-gallery-icons-grid")
}
// endregion: example

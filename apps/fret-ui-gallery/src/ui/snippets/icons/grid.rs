pub const SOURCE: &str = include_str!("grid.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    use fret_icons::ids;

    let icon_cell = |cx: &mut ElementContext<'_, H>,
                     label: &'static str,
                     icon_id: fret_icons::IconId|
     -> AnyElement {
        let row = ui::h_flex(|cx| {
            vec![
                fret_ui_shadcn::icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
                cx.text(label),
            ]
        })
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

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

    ui::v_flex(|cx| {
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
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N2)
    .into_element(cx)
    .test_id("ui-gallery-icons-grid")
}
// endregion: example

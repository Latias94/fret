use super::super::super::super::*;

pub(in crate::ui) fn preview_icons(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pages::preview_icons(cx)
}

#[cfg(any())]
pub(in crate::ui) fn preview_icons_legacy(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_icons::ids;

    let icon_cell =
        |cx: &mut ElementContext<'_, App>, label: &str, icon_id: IconId| -> AnyElement {
            let row = ui::h_flex(|cx| {
                    vec![
                        icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                })
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center().into_element(cx);

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

    let grid = ui::v_flex(|cx| {
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
        })
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2).into_element(cx);

    let spinner_row = ui::h_row(|cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new().speed(0.0).into_element(cx),
                cx.text("Spinner (animated / static)"),
            ]
        }).gap(Space::N2).items_center().into_element(cx);

    vec![grid, spinner_row]
}

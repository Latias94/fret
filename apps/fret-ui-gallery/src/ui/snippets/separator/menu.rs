pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::viewport_queries;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    description: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let _ = cx;
    ui::v_stack(move |cx| {
        vec![
            shadcn::raw::typography::small(title).into_element(cx),
            shadcn::raw::typography::muted(description).into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_start()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let is_md = viewport_queries::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        viewport_queries::tailwind::MD,
        viewport_queries::ViewportQueryHysteresis::default(),
    );

    ui::h_flex(|cx| {
        let mut children = vec![
            section(cx, "Settings", "Manage preferences").into_element(cx),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into_element(cx),
            section(cx, "Account", "Profile & security").into_element(cx),
        ];

        if is_md {
            children.push(
                shadcn::Separator::new()
                    .orientation(shadcn::SeparatorOrientation::Vertical)
                    .into_element(cx),
            );
            children.push(section(cx, "Help", "Support & docs").into_element(cx));
        }

        children
    })
    .gap(if is_md { Space::N4 } else { Space::N2 })
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-separator-menu")
}
// endregion: example

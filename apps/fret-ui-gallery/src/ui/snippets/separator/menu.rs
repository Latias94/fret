pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::viewport_queries;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    test_id: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let muted_fg = Theme::global(&*cx.app).color_token("muted-foreground");
    ui::v_stack(move |cx| {
        vec![
            ui::text(title)
                .text_sm()
                .font_medium()
                .fixed_line_box_px(Px(20.0))
                .into_element(cx),
            ui::text(description)
                .text_xs()
                .fixed_line_box_px(Px(16.0))
                .text_color(ColorRef::Color(muted_fg))
                .into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .into_element(cx)
    .test_id(test_id)
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
            section(
                cx,
                "ui-gallery-separator-menu-settings",
                "Settings",
                "Manage preferences",
            )
            .into_element(cx),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into_element(cx)
                .test_id("ui-gallery-separator-menu-divider-primary"),
            section(
                cx,
                "ui-gallery-separator-menu-account",
                "Account",
                "Profile & security",
            )
            .into_element(cx),
        ];

        if is_md {
            children.push(
                shadcn::Separator::new()
                    .orientation(shadcn::SeparatorOrientation::Vertical)
                    .into_element(cx)
                    .test_id("ui-gallery-separator-menu-divider-secondary"),
            );
            children.push(
                section(
                    cx,
                    "ui-gallery-separator-menu-help",
                    "Help",
                    "Support & docs",
                )
                .into_element(cx),
            );
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

pub const SOURCE: &str = include_str!("menu.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui_kit::{
    IntoUiElement,
    declarative::{text as decl_text, viewport_queries},
};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: &'static str,
    description: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    ui::v_stack(move |cx| {
        vec![
            decl_text::text_section_chrome_label(cx, title),
            decl_text::text_control_readout(cx, description),
        ]
    })
    .gap(Space::N1)
    .items_start()
    .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let is_md = viewport_queries::viewport_width_at_least(
        cx,
        Invalidation::Layout,
        viewport_queries::tailwind::MD,
        viewport_queries::ViewportQueryHysteresis::default(),
    );

    ui::h_flex(|cx| {
        let mut children = vec![
            section(cx, "Settings", "Manage preferences")
                .into_element(cx)
                .test_id("ui-gallery-separator-menu-settings"),
            shadcn::Separator::new()
                .orientation(shadcn::SeparatorOrientation::Vertical)
                .into_element(cx)
                .test_id("ui-gallery-separator-menu-divider-primary"),
            section(cx, "Account", "Profile & security")
                .into_element(cx)
                .test_id("ui-gallery-separator-menu-account"),
        ];

        if is_md {
            children.push(
                shadcn::Separator::new()
                    .orientation(shadcn::SeparatorOrientation::Vertical)
                    .into_element(cx)
                    .test_id("ui-gallery-separator-menu-divider-secondary"),
            );
            children.push(
                section(cx, "Help", "Support & docs")
                    .into_element(cx)
                    .test_id("ui-gallery-separator-menu-help"),
            );
        }

        children
    })
    .gap(if is_md { Space::N4 } else { Space::N2 })
    .items_center()
    .layout(
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .min_h(Px(40.0)),
    )
    .into_element(cx)
    .test_id("ui-gallery-separator-menu")
}
// endregion: example

pub const SOURCE: &str = include_str!("background.rs");

// region: example
use fret_core::scene::{ColorSpace, GradientStop, LinearGradient, MAX_STOPS, Paint, TileMode};
use fret_ui::Invalidation;
use fret_ui::element::LayoutQueryRegionProps;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let region_layout = cx.with_theme(|theme| {
        fret_ui_kit::declarative::style::layout_style(
            theme,
            LayoutRefinement::default().w_full().min_h(Px(280.0)),
        )
    });

    fret_ui_kit::declarative::container_query_region_with_id(
        cx,
        "ui-gallery.empty.background",
        LayoutQueryRegionProps {
            layout: region_layout,
            name: None,
        },
        move |cx, region_id| {
            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let muted = theme.color_token("muted");
            let bg = theme.color_token("background");

            let paint = cx
                .layout_query_bounds(region_id, Invalidation::Layout)
                .map(|rect| {
                    let mut from = muted;
                    from.a = (from.a * 0.5).clamp(0.0, 1.0);

                    let mut stops =
                        [GradientStop::new(0.0, fret_core::Color::TRANSPARENT); MAX_STOPS];
                    stops[0] = GradientStop::new(0.30, from);
                    stops[1] = GradientStop::new(1.0, bg);

                    Paint::LinearGradient(LinearGradient {
                        start: rect.origin,
                        end: fret_core::Point::new(
                            rect.origin.x,
                            Px(rect.origin.y.0 + rect.size.height.0),
                        ),
                        tile_mode: TileMode::Clamp,
                        color_space: ColorSpace::Srgb,
                        stop_count: 2,
                        stops,
                    })
                })
                .unwrap_or_else(|| Paint::Solid(muted));

            let refresh_icon =
                fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.refresh-cw"));
            let refresh_text = cx.text("Refresh");
            let refresh_button = shadcn::Button::new("Refresh")
                .variant(shadcn::ButtonVariant::Outline)
                .children([refresh_icon, refresh_text])
                .into_element(cx);

            let header_icon =
                fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.bell"));
            let empty = shadcn::empty(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_header(|cx| {
                        ui::children![
                            cx;
                            shadcn::empty_media(|cx| ui::children![cx; header_icon])
                                .variant(fret_ui_shadcn::empty::EmptyMediaVariant::Icon),
                            shadcn::empty_title("No Notifications"),
                            shadcn::empty_description(
                                "You're all caught up. New notifications will appear here.",
                            ),
                        ]
                    }),
                    shadcn::empty_content(|cx| ui::children![cx; refresh_button]),
                ]
            })
            .refine_style(ChromeRefinement::default().background_paint(paint))
            .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
            .into_element(cx)
            .test_id("ui-gallery-empty-background");

            vec![empty]
        },
    )
}
// endregion: example

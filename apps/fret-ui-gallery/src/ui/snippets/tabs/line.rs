pub const SOURCE: &str = include_str!("line.rs");

// region: example
use fret_app::App;
use fret_core::{Color as CoreColor, Px};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

fn line_style(cx: &mut ElementContext<'_, App>) -> shadcn::tabs::TabsStyle {
    let primary = cx.with_theme(|theme| theme.color_token("primary"));
    shadcn::tabs::TabsStyle::default()
        .trigger_background(fret_ui_kit::WidgetStateProperty::new(Some(
            ColorRef::Color(CoreColor::TRANSPARENT),
        )))
        .trigger_border_color(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(CoreColor::TRANSPARENT)))
                .when(
                    fret_ui_kit::WidgetStates::SELECTED,
                    Some(ColorRef::Color(primary)),
                ),
        )
}

pub fn render(cx: &mut ElementContext<'_, App>) -> AnyElement {
    shadcn::Tabs::uncontrolled(Some(Arc::<str>::from("preview")))
        .style(line_style(cx))
        .refine_style(ChromeRefinement::default().bg(ColorRef::Color(CoreColor::TRANSPARENT)))
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(460.0)))
        .items([
            shadcn::TabsItem::new("preview", "Preview", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.app-window")),
            shadcn::TabsItem::new("code", "Code", Vec::<AnyElement>::new())
                .trigger_leading_icon(IconId::new_static("lucide.code")),
        ])
        .into_element(cx)
        .test_id("ui-gallery-tabs-line")
}

// endregion: example

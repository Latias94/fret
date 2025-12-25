use fret_components_icons::{IconGlyph, IconId, IconRegistry};
use fret_core::{Color, FontId, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::TextProps;
use fret_ui::{ElementCx, Theme, UiHost};

use super::style;
use crate::{ColorRef, LayoutRefinement, MetricRef};

#[track_caller]
pub fn icon<H: UiHost>(cx: &mut ElementCx<'_, H>, icon: IconId) -> fret_ui::element::AnyElement {
    icon_with(cx, icon, None, None)
}

#[track_caller]
pub fn icon_with<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    icon: IconId,
    size: Option<Px>,
    color: Option<ColorRef>,
) -> fret_ui::element::AnyElement {
    cx.scope(|cx| {
        let glyph: IconGlyph = cx
            .app
            .with_global_mut(IconRegistry::default, |icons, _app| {
                icons
                    .glyph(&icon)
                    .cloned()
                    .unwrap_or_else(|| IconGlyph::new("?"))
            });

        let size = size.unwrap_or(glyph.size);
        let theme = Theme::global(&*cx.app);
        let color: Color = color
            .map(|c| c.resolve(theme))
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        let layout = style::layout_style(
            theme,
            LayoutRefinement::default()
                .w_px(MetricRef::Px(size))
                .h_px(MetricRef::Px(size)),
        );

        let props = TextProps {
            layout,
            text: glyph.text.into(),
            style: Some(TextStyle {
                font: glyph.font,
                size,
                line_height: Some(size),
                ..Default::default()
            }),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        };

        cx.text_props(props)
    })
}

#[track_caller]
pub fn icon_fixed_font<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    text: &'static str,
    font: FontId,
    size: Px,
    color: Option<ColorRef>,
) -> fret_ui::element::AnyElement {
    cx.scope(|cx| {
        let theme = Theme::global(&*cx.app);
        let color: Color = color
            .map(|c| c.resolve(theme))
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);

        let layout = style::layout_style(
            theme,
            LayoutRefinement::default()
                .w_px(MetricRef::Px(size))
                .h_px(MetricRef::Px(size)),
        );

        cx.text_props(TextProps {
            layout,
            text: text.into(),
            style: Some(TextStyle {
                font,
                size,
                line_height: Some(size),
                ..Default::default()
            }),
            color: Some(color),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    })
}

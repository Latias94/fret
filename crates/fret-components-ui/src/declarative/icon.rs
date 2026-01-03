use fret_components_icons::{IconId, IconRegistry, MISSING_ICON_SVG, ResolvedSvg};
use fret_core::{Color, Px};
use fret_ui::SvgSource;
use fret_ui::element::SvgIconProps;
use fret_ui::{ElementContext, Theme, UiHost};

use super::style;
use crate::{ColorRef, LayoutRefinement, MetricRef};

#[track_caller]
pub fn icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: IconId,
) -> fret_ui::element::AnyElement {
    icon_with(cx, icon, None, None)
}

#[track_caller]
pub fn icon_with<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    icon: IconId,
    size: Option<Px>,
    color: Option<ColorRef>,
) -> fret_ui::element::AnyElement {
    cx.scope(|cx| {
        let svg: SvgSource =
            cx.app
                .with_global_mut(IconRegistry::default, |icons, _app| {
                    match icons.resolve_svg(&icon) {
                        Some(ResolvedSvg::Static(bytes)) => SvgSource::Static(bytes),
                        Some(ResolvedSvg::Bytes(bytes)) => SvgSource::Bytes(bytes.clone()),
                        None => SvgSource::Static(MISSING_ICON_SVG),
                    }
                });

        let theme = Theme::global(&*cx.app);
        let size = size.unwrap_or(Px(16.0));
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

        let mut props = SvgIconProps::new(svg);
        props.layout = layout;
        props.color = color;
        cx.svg_icon_props(props)
    })
}

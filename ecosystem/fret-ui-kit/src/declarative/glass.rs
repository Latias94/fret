use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{EffectMode, EffectQuality};
use fret_ui::element::{
    AnyElement, ContainerProps, EffectLayerProps, LayoutStyle, Length, Overflow,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::ChromeRefinement;
use crate::declarative::reduced_transparency_queries;
use crate::recipes::glass::{
    GlassEffectRefinement, GlassEffectTokenKeys, GlassTokenKeys,
    glass_effect_chain_for_environment, resolve_glass_chrome, resolve_glass_effect,
};

#[derive(Debug, Clone)]
pub struct GlassPanelProps {
    pub layout: LayoutStyle,
    pub mode: EffectMode,
    pub quality: EffectQuality,
    pub chrome: ChromeRefinement,
    pub chrome_keys: GlassTokenKeys,
    pub effect: GlassEffectRefinement,
    pub effect_keys: GlassEffectTokenKeys,
}

impl Default for GlassPanelProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            mode: EffectMode::Backdrop,
            quality: EffectQuality::Auto,
            chrome: ChromeRefinement::default(),
            chrome_keys: GlassTokenKeys::none(),
            effect: GlassEffectRefinement::default(),
            effect_keys: GlassEffectTokenKeys::none(),
        }
    }
}

pub fn glass_panel<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: GlassPanelProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let prefers_reduced_transparency =
        reduced_transparency_queries::prefers_reduced_transparency(cx, Invalidation::Paint, false);

    let theme = Theme::global(&*cx.app);
    let chrome = resolve_glass_chrome(theme, &props.chrome, props.chrome_keys);
    let effect = resolve_glass_effect(theme, &props.effect, props.effect_keys);
    let chain = glass_effect_chain_for_environment(effect, prefers_reduced_transparency);

    // Structure:
    //
    // - Outer wrapper provides the rounded clip that the renderer can consume at the effect
    //   boundary (ADR 0119 + ADR 0153).
    // - Effect layer wraps a tinted/bordered container (tint + border are drawn after the blurred
    //   backdrop inside the group).
    let outer = ContainerProps {
        layout: LayoutStyle {
            overflow: Overflow::Clip,
            ..props.layout
        },
        corner_radii: Corners::all(chrome.radius),
        ..Default::default()
    };

    cx.container(outer, move |cx| {
        let mut effect_layout = LayoutStyle::default();
        effect_layout.size.width = Length::Fill;
        effect_layout.size.height = Length::Fill;

        let layer = cx.effect_layer_props(
            EffectLayerProps {
                layout: effect_layout,
                mode: props.mode,
                chain,
                quality: props.quality,
            },
            |cx| {
                let mut inner_layout = LayoutStyle::default();
                inner_layout.size.width = Length::Fill;
                inner_layout.size.height = Length::Fill;
                let inner = ContainerProps {
                    layout: inner_layout,
                    padding: Edges::symmetric(chrome.padding_x, chrome.padding_y),
                    background: Some(chrome.tint),
                    border: Edges::all(chrome.border_width),
                    border_color: Some(chrome.border),
                    corner_radii: Corners::all(chrome.radius),
                    ..Default::default()
                };

                vec![cx.container(inner, children)]
            },
        );

        vec![layer]
    })
}

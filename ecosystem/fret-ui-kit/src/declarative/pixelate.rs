use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{EffectMode, EffectQuality};
use fret_ui::element::{
    AnyElement, ContainerProps, EffectLayerProps, LayoutStyle, Length, Overflow,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::ChromeRefinement;
use crate::recipes::pixelate::{
    PixelateEffectRefinement, PixelateEffectTokenKeys, PixelateTokenKeys, pixelate_effect_chain,
    resolve_pixelate_chrome, resolve_pixelate_effect,
};

#[derive(Debug, Clone)]
pub struct PixelatePanelProps {
    pub layout: LayoutStyle,
    pub mode: EffectMode,
    pub quality: EffectQuality,
    pub chrome: ChromeRefinement,
    pub chrome_keys: PixelateTokenKeys,
    pub effect: PixelateEffectRefinement,
    pub effect_keys: PixelateEffectTokenKeys,
}

impl Default for PixelatePanelProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            mode: EffectMode::FilterContent,
            quality: EffectQuality::Auto,
            chrome: ChromeRefinement::default(),
            chrome_keys: PixelateTokenKeys::none(),
            effect: PixelateEffectRefinement::default(),
            effect_keys: PixelateEffectTokenKeys::none(),
        }
    }
}

pub fn pixelate_panel<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: PixelatePanelProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let theme = Theme::global(&*cx.app);
    let chrome = resolve_pixelate_chrome(theme, &props.chrome, props.chrome_keys);
    let effect = resolve_pixelate_effect(theme, &props.effect, props.effect_keys);
    let chain = pixelate_effect_chain(effect);

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
                    background: Some(chrome.background),
                    border: Edges::all(chrome.border_width),
                    border_color: Some(chrome.border_color),
                    corner_radii: Corners::all(chrome.radius),
                    ..Default::default()
                };

                vec![cx.container(inner, children)]
            },
        );

        vec![layer]
    })
}

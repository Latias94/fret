use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{BlendMode, EffectMode, EffectQuality};
use fret_ui::element::{
    AnyElement, CompositeGroupProps, ContainerProps, EffectLayerProps, FocusTraversalGateProps,
    HitTestGateProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::ChromeRefinement;
use crate::recipes::bloom::{BloomEffect, bloom_effect_chain};
use crate::recipes::surface::{SurfaceTokenKeys, resolve_surface_chrome};

#[derive(Debug, Clone)]
pub struct BloomPanelProps {
    pub layout: LayoutStyle,
    pub mode: EffectMode,
    pub quality: EffectQuality,
    pub chrome: ChromeRefinement,
    pub chrome_keys: SurfaceTokenKeys,
    pub effect: BloomEffect,
}

impl Default for BloomPanelProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
        Self {
            layout,
            mode: EffectMode::FilterContent,
            quality: EffectQuality::Auto,
            chrome: ChromeRefinement::default(),
            chrome_keys: SurfaceTokenKeys {
                padding_x: None,
                padding_y: None,
                radius: None,
                border_width: None,
                bg: None,
                border: None,
            },
            effect: BloomEffect::default(),
        }
    }
}

pub fn bloom_panel<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: BloomPanelProps,
    mut children: impl FnMut(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let theme = Theme::global(&*cx.app);
    let chrome = resolve_surface_chrome(theme, &props.chrome, props.chrome_keys);

    let chain = bloom_effect_chain(props.effect);
    chain.report_if_degraded(&mut *cx.app);

    let outer = ContainerProps {
        layout: LayoutStyle {
            overflow: Overflow::Visible,
            ..props.layout
        },
        corner_radii: Corners::all(chrome.radius),
        ..Default::default()
    };

    cx.container(outer, move |cx| {
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

        let base_children: Vec<AnyElement> = children(cx).into_iter().collect();
        let glow_children: Vec<AnyElement> = children(cx).into_iter().collect();

        let base = cx.container(inner, move |_cx| base_children);

        let overlay_layout = LayoutStyle {
            position: PositionStyle::Absolute,
            inset: InsetStyle {
                top: Some(fret_core::Px(0.0)),
                right: Some(fret_core::Px(0.0)),
                bottom: Some(fret_core::Px(0.0)),
                left: Some(fret_core::Px(0.0)),
            },
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                ..Default::default()
            },
            ..Default::default()
        };

        let overlay = cx.focus_traversal_gate_props(
            FocusTraversalGateProps {
                layout: overlay_layout,
                traverse: false,
            },
            move |cx| {
                let mut hit_layout = LayoutStyle::default();
                hit_layout.size.width = Length::Fill;
                hit_layout.size.height = Length::Fill;

                [cx.hit_test_gate_props(
                    HitTestGateProps {
                        layout: hit_layout,
                        hit_test: false,
                    },
                    move |cx| {
                        let mut fill = LayoutStyle::default();
                        fill.size.width = Length::Fill;
                        fill.size.height = Length::Fill;

                        let group = cx.composite_group_props(
                            CompositeGroupProps {
                                layout: fill,
                                mode: BlendMode::Add,
                                quality: props.quality,
                            },
                            move |cx| {
                                let mut effect_layout = LayoutStyle::default();
                                effect_layout.size.width = Length::Fill;
                                effect_layout.size.height = Length::Fill;

                                vec![cx.effect_layer_props(
                                    EffectLayerProps {
                                        layout: effect_layout,
                                        mode: props.mode,
                                        chain: chain.value,
                                        quality: props.quality,
                                    },
                                    move |_cx| glow_children,
                                )]
                            },
                        );

                        vec![group]
                    },
                )]
            },
        );

        vec![base, overlay]
    })
}

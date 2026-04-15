use std::sync::Arc;

use fret::component::prelude::*;
use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_core::scene::{EffectChain, EffectMode, EffectStep};

const TEST_ID_ROOT: &str = "cookbook.effects_layer_basics.root";
const TEST_ID_GROUP: &str = "cookbook.effects_layer_basics.effect_group";
const TEST_ID_NONE: &str = "cookbook.effects_layer_basics.effect.none";
const TEST_ID_PIXELATE: &str = "cookbook.effects_layer_basics.effect.pixelate";
const TEST_ID_BLUR: &str = "cookbook.effects_layer_basics.effect.blur";
const TEST_ID_PREVIEW: &str = "cookbook.effects_layer_basics.preview";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectKind {
    None,
    Pixelate,
    Blur,
}

impl EffectKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pixelate => "pixelate",
            Self::Blur => "blur",
        }
    }

    fn from_value(value: Option<&str>) -> Self {
        match value {
            Some(value) if value == Self::Pixelate.as_str() => Self::Pixelate,
            Some(value) if value == Self::Blur.as_str() => Self::Blur,
            _ => Self::None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Pixelate => "Pixelate",
            Self::Blur => "Blur",
        }
    }

    fn a11y_label(self) -> &'static str {
        match self {
            Self::None => "No effect",
            Self::Pixelate => "Pixelate effect",
            Self::Blur => "Blur effect",
        }
    }

    fn test_id(self) -> &'static str {
        match self {
            Self::None => TEST_ID_NONE,
            Self::Pixelate => TEST_ID_PIXELATE,
            Self::Blur => TEST_ID_BLUR,
        }
    }
}

struct EffectsLayerBasicsView {
    effect: Model<Option<Arc<str>>>,
}

impl View for EffectsLayerBasicsView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            effect: app
                .models_mut()
                .insert(Some(Arc::from(EffectKind::None.as_str()))),
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = cx.theme_snapshot();

        let effect_kind = EffectKind::from_value(
            self.effect
                .layout(cx)
                .value_or(Some(Arc::<str>::from(EffectKind::None.as_str())))
                .as_deref(),
        );

        let chain = match effect_kind {
            EffectKind::None => EffectChain::EMPTY,
            EffectKind::Pixelate => EffectChain::from_steps(&[EffectStep::Pixelate { scale: 10 }]),
            EffectKind::Blur => EffectChain::from_steps(&[EffectStep::GaussianBlur {
                radius_px: Px(6.0),
                downsample: 1,
            }]),
        };

        let preview_content = ui::v_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("EffectLayer preview"),
                ui::h_flex(|cx| {
                    let tile = |_cx: &mut UiCx<'_>, color: ColorRef| {
                        ui::container(|_cx| Vec::<AnyElement>::new())
                            .w_px(Px(28.0))
                            .h_px(Px(28.0))
                            .bg(color)
                            .rounded_md()
                    };
                    ui::children![
                        cx;
                        tile(cx, ColorRef::Color(theme.color_token("chart.1"))),
                        tile(cx, ColorRef::Color(theme.color_token("chart.2"))),
                        tile(cx, ColorRef::Color(theme.color_token("chart.3"))),
                        tile(cx, ColorRef::Color(theme.color_token("chart.4"))),
                    ]
                })
                .gap(Space::N2),
                shadcn::Label::new("Toggle an effect above. This is a mechanism-level API."),
            ]
        })
        .gap(Space::N3);

        let preview = {
            // Keep a definite pixel-sized box here: percent-fill sizing inside a shrink-wrapped
            // effect layer can create cyclic layout dependencies.
            let body = if effect_kind == EffectKind::None {
                ui::container(move |_cx| [preview_content])
                    .w_px(Px(460.0))
                    .h_px(Px(180.0))
                    .p(Space::N4)
                    .bg(ColorRef::Color(theme.color_token("background")))
                    .into_element(cx)
            } else {
                let content = ui::container(move |_cx| [preview_content])
                    .w_px(Px(460.0))
                    .h_px(Px(180.0))
                    .p(Space::N4)
                    .bg(ColorRef::Color(theme.color_token("background")));
                ui::effect_layer(EffectMode::FilterContent, chain, move |_cx| [content])
                    .into_element(cx)
            };

            ui::container(|_cx| [body])
                .w_px(Px(460.0))
                .h_px(Px(180.0))
                .border_1()
                .border_color(ColorRef::Color(theme.color_token("border")))
                .rounded_md()
                .overflow_hidden()
                .test_id(TEST_ID_PREVIEW)
        };

        let controls = shadcn::ToggleGroup::single(&self.effect)
            .deselectable(false)
            .variant(shadcn::ToggleVariant::Outline)
            .spacing(Space::N2)
            .items([
                shadcn::ToggleGroupItem::new(
                    EffectKind::None.as_str(),
                    [cx.text(EffectKind::None.label())],
                )
                .a11y_label(EffectKind::None.a11y_label())
                .test_id(EffectKind::None.test_id()),
                shadcn::ToggleGroupItem::new(
                    EffectKind::Pixelate.as_str(),
                    [cx.text(EffectKind::Pixelate.label())],
                )
                .a11y_label(EffectKind::Pixelate.a11y_label())
                .test_id(EffectKind::Pixelate.test_id()),
                shadcn::ToggleGroupItem::new(
                    EffectKind::Blur.as_str(),
                    [cx.text(EffectKind::Blur.label())],
                )
                .a11y_label(EffectKind::Blur.a11y_label())
                .test_id(EffectKind::Blur.test_id()),
            ])
            .refine_layout(LayoutRefinement::default().flex_none())
            .into_element(cx)
            .test_id(TEST_ID_GROUP);

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Effects layer basics"),
                        shadcn::card_description(
                            "A minimal example showing `EffectLayer` + `EffectChain` (Pixelate, Blur).",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| {
                    ui::children![
                        cx;
                        controls,
                        ui::v_flex(|cx| ui::children![cx; preview]).gap(Space::N3),
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(520.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-effects-layer-basics")
        .window("cookbook-effects-layer-basics", (680.0, 460.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<EffectsLayerBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}

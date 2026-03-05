use fret::prelude::*;
use fret_core::scene::{EffectChain, EffectMode, EffectStep};

mod act {
    fret::actions!([
        None = "cookbook.effects_layer_basics.effect.none.v1",
        Pixelate = "cookbook.effects_layer_basics.effect.pixelate.v1",
        Blur = "cookbook.effects_layer_basics.effect.blur.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.effects_layer_basics.root";
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

struct EffectsLayerBasicsView {
    effect: Model<EffectKind>,
}

impl View for EffectsLayerBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        Self {
            effect: app.models_mut().insert(EffectKind::None),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let effect_kind = cx
            .watch_model(&self.effect)
            .layout()
            .copied_or(EffectKind::None);

        cx.on_action::<act::None>({
            let effect = self.effect.clone();
            move |host, acx| {
                let _ = host.models_mut().update(&effect, |v| *v = EffectKind::None);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action::<act::Pixelate>({
            let effect = self.effect.clone();
            move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&effect, |v| *v = EffectKind::Pixelate);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action::<act::Blur>({
            let effect = self.effect.clone();
            move |host, acx| {
                let _ = host.models_mut().update(&effect, |v| *v = EffectKind::Blur);
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        let button = |cx: &mut ElementContext<'_, App>,
                      label: &'static str,
                      effect: EffectKind,
                      action: fret_runtime::ActionId,
                      test_id: &'static str| {
            let selected = effect_kind == effect;
            shadcn::Button::new(label)
                .variant(if selected {
                    shadcn::ButtonVariant::Default
                } else {
                    shadcn::ButtonVariant::Outline
                })
                .action(action)
                .into_element(cx)
                .role(SemanticsRole::Button)
                .test_id(test_id)
        };

        let chain = match effect_kind {
            EffectKind::None => EffectChain::EMPTY,
            EffectKind::Pixelate => EffectChain::from_steps(&[EffectStep::Pixelate { scale: 10 }]),
            EffectKind::Blur => EffectChain::from_steps(&[EffectStep::GaussianBlur {
                radius_px: Px(6.0),
                downsample: 1,
            }]),
        };

        let preview_content = ui::v_flex(|cx| {
            [
                shadcn::Label::new("EffectLayer preview").into_element(cx),
                ui::h_flex(|cx| {
                    let tile = |cx: &mut ElementContext<'_, App>, color: ColorRef| {
                        ui::container(|_cx| Vec::<AnyElement>::new())
                            .w_px(Px(28.0))
                            .h_px(Px(28.0))
                            .bg(color)
                            .rounded_md()
                            .into_element(cx)
                    };
                    [
                        tile(cx, ColorRef::Color(theme.color_token("chart.1"))),
                        tile(cx, ColorRef::Color(theme.color_token("chart.2"))),
                        tile(cx, ColorRef::Color(theme.color_token("chart.3"))),
                        tile(cx, ColorRef::Color(theme.color_token("chart.4"))),
                    ]
                })
                .gap(Space::N2)
                .into_element(cx),
                shadcn::Label::new("Toggle an effect above. This is a mechanism-level API.")
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .into_element(cx);

        let preview = {
            // Keep a definite pixel-sized box here: percent-fill sizing inside a shrink-wrapped
            // effect layer can create cyclic layout dependencies.
            let content = ui::container(move |_cx| [preview_content])
                .w_px(Px(460.0))
                .h_px(Px(180.0))
                .p(Space::N4)
                .bg(ColorRef::Color(theme.color_token("background")))
                .into_element(cx);

            let body = if effect_kind == EffectKind::None {
                content
            } else {
                cx.effect_layer(EffectMode::FilterContent, chain, move |_cx| [content])
            };

            ui::container(|_cx| [body])
                .w_px(Px(460.0))
                .h_px(Px(180.0))
                .border_1()
                .border_color(ColorRef::Color(theme.color_token("border")))
                .rounded_md()
                .overflow_hidden()
                .into_element(cx)
                .test_id(TEST_ID_PREVIEW)
        };

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Effects layer basics").into_element(cx),
            shadcn::CardDescription::new(
                "A minimal example showing `EffectLayer` + `EffectChain` (Pixelate, Blur).",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let controls = ui::h_flex(|cx| {
            [
                button(cx, "None", EffectKind::None, act::None.into(), TEST_ID_NONE),
                button(
                    cx,
                    "Pixelate",
                    EffectKind::Pixelate,
                    act::Pixelate.into(),
                    TEST_ID_PIXELATE,
                ),
                button(cx, "Blur", EffectKind::Blur, act::Blur.into(), TEST_ID_BLUR),
            ]
        })
        .gap(Space::N2)
        .into_element(cx);

        let content = shadcn::CardContent::new([
            controls,
            ui::v_flex(|_cx| [preview]).gap(Space::N3).into_element(cx),
        ])
        .into_element(cx);

        let card = shadcn::Card::new([header, content])
            .ui()
            .w_full()
            .max_w(Px(520.0))
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-effects-layer-basics")
        .window("cookbook-effects-layer-basics", (680.0, 460.0))
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<EffectsLayerBasicsView>()
        .map_err(anyhow::Error::from)
}

use fret::{FretApp, advanced::prelude::*, shadcn};
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
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            effect: app.models_mut().insert(EffectKind::None),
        }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements {
        let theme = Theme::global(&*cx.app).snapshot();

        let effect_kind = cx
            .watch_model(&self.effect)
            .layout()
            .value_or(EffectKind::None);

        cx.on_action_notify_model_set::<act::None, EffectKind>(
            self.effect.clone(),
            EffectKind::None,
        );
        cx.on_action_notify_model_set::<act::Pixelate, EffectKind>(
            self.effect.clone(),
            EffectKind::Pixelate,
        );
        cx.on_action_notify_model_set::<act::Blur, EffectKind>(
            self.effect.clone(),
            EffectKind::Blur,
        );

        let button = |_cx: &mut ElementContext<'_, KernelApp>,
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
                .a11y_role(SemanticsRole::Button)
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
            ui::children![
                cx;
                shadcn::Label::new("EffectLayer preview"),
                ui::h_flex(|cx| {
                    let tile = |_cx: &mut ElementContext<'_, KernelApp>, color: ColorRef| {
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

        let controls = ui::h_flex(|cx| {
            ui::children![
                cx;
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
        .gap(Space::N2);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Effects layer basics"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "A minimal example showing `EffectLayer` + `EffectChain` (Pixelate, Blur).",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, controls);
                    out.push_ui(cx, ui::v_flex(|cx| ui::children![cx; preview]).gap(Space::N3));
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(520.0));

        fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-effects-layer-basics")
        .window("cookbook-effects-layer-basics", (680.0, 460.0))
        .setup(fret_cookbook::install_cookbook_defaults)
        .run_view::<EffectsLayerBasicsView>()
        .map_err(anyhow::Error::from)
}

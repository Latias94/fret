//! Renderer semantics: DropShadowV1.
//!
//! This example is intentionally small and “visual-first”:
//! - Toggle a single DropShadowV1 effect layer on/off.
//! - Optionally enable a small grid to increase the number of effect layers (useful for manual
//!   perf inspection).
//!
//! Notes:
//! - DropShadowV1 is a bounded multi-pass effect (scissored computation).
//! - The effect is computed within the effect layer bounds; leave padding inside the layer so the
//!   shadow has space to be visible.

#![cfg(not(target_arch = "wasm32"))]

use fret::{FretApp, advanced::prelude::*, shadcn};
use fret_core::Point;
use fret_core::scene::{Color, DropShadowV1, EffectChain, EffectMode, EffectStep};

const TEST_ID_ROOT: &str = "cookbook.drop_shadow_basics.root";
const TEST_ID_SWITCH_SHADOW: &str = "cookbook.drop_shadow_basics.switch.shadow";
const TEST_ID_SWITCH_STRESS: &str = "cookbook.drop_shadow_basics.switch.stress";
const TEST_ID_STAGE: &str = "cookbook.drop_shadow_basics.stage";

fn shadow_chain() -> EffectChain {
    let shadow = DropShadowV1 {
        offset_px: Point::new(Px(0.0), Px(8.0)),
        blur_radius_px: Px(10.0),
        downsample: 2,
        color: Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.55,
        },
    };

    EffectChain::from_steps(&[EffectStep::DropShadowV1(shadow)]).sanitize()
}

struct DropShadowBasicsView;

impl View for DropShadowBasicsView {
    fn init(_app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let theme = Theme::global(&*cx.app).snapshot();

        let enabled_state = cx.state().local_init(|| true);
        let stress_state = cx.state().local_init(|| false);

        let enabled = cx.state().watch(&enabled_state).layout().value_or(true);
        let stress = cx.state().watch(&stress_state).layout().value_or(false);

        let toolbar = ui::v_flex(|cx| {
            let row_shadow = ui::h_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new("Enable DropShadowV1:"),
                    shadcn::Switch::new(enabled_state.clone_model())
                        .test_id(TEST_ID_SWITCH_SHADOW),
                ]
            })
            .gap(Space::N2)
            .items_center();

            let row_stress = ui::h_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new("Stress grid:"),
                    shadcn::Switch::new(stress_state.clone_model())
                        .test_id(TEST_ID_SWITCH_STRESS),
                ]
            })
            .gap(Space::N2)
            .items_center();

            ui::children![
                cx;
                shadcn::Alert::new(ui::children![
                    cx;
                    shadcn::AlertTitle::new("Renderer semantics"),
                    shadcn::AlertDescription::new(
                        "DropShadowV1 is computed within effect bounds (scissored). Keep padding inside the layer so the shadow can be visible.",
                    ),
                ]),
                row_shadow,
                row_stress,
            ]
        })
        .gap(Space::N3);

        let chain = shadow_chain();

        let card = |cx: &mut ElementContext<'_, KernelApp>, title: String| -> AnyElement {
            let surface = ui::v_flex(|cx| {
                ui::children![
                    cx;
                    shadcn::Label::new(title),
                    shadcn::Badge::new("DropShadowV1")
                        .variant(shadcn::BadgeVariant::Secondary),
                ]
            })
            .gap(Space::N2)
            .p(Space::N4)
            .bg(ColorRef::Color(theme.color_token("background")))
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .rounded_md()
            .size_full();

            // Keep a fixed-size effect layer and pad inside it so the shadow has space.
            let padded = ui::container(|_cx| [surface]).p(Space::N5).size_full();

            let bounds = ui::container(|_cx| [padded])
                .w_px(Px(260.0))
                .h_px(Px(160.0))
                .overflow_hidden()
                .bg(ColorRef::Color(theme.color_token("muted")))
                .rounded_md();

            if enabled {
                ui::effect_layer(EffectMode::FilterContent, chain.clone(), move |_cx| {
                    [bounds]
                })
                .into_element(cx)
            } else {
                bounds.into_element(cx)
            }
        };

        let (rows, cols) = if stress {
            (4usize, 3usize)
        } else {
            (2usize, 3usize)
        };
        let stage = ui::v_flex_build(|cx, out| {
            for r in 0..rows {
                out.push_ui(
                    cx,
                    ui::h_flex(|cx| {
                        let mut row: Vec<AnyElement> = Vec::with_capacity(cols);
                        for c in 0..cols {
                            let i = r * cols + c;
                            row.push(card(cx, format!("Card {i}")));
                        }
                        row
                    })
                    .gap(Space::N4),
                );
            }
        })
        .gap(Space::N4)
        .items_center()
        .test_id(TEST_ID_STAGE);

        let header = shadcn::CardHeader::build(|cx, out| {
            out.push_ui(cx, shadcn::CardTitle::new("Drop shadow basics"));
            out.push_ui(
                cx,
                shadcn::CardDescription::new(
                    "A small, deterministic surface for DropShadowV1 renderer semantics (toggle + screenshot baseline).",
                ),
            );
        });

        let content = ui::v_flex(|cx| ui::children![cx; toolbar, stage]).gap(Space::N5);

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(cx, header);
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, content);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(1180.0));

        fret_cookbook::scaffold::centered_page_background_ui(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-drop-shadow-basics")
        .window("cookbook-drop-shadow-basics", (1280.0, 860.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<DropShadowBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}

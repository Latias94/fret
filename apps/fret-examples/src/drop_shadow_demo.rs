//! Drop shadow demo (EffectStep::DropShadowV1).
//!
//! This is intended as a renderer semantics validation surface:
//! - bounded multi-pass behavior (scissored computation),
//! - deterministic degradation under budgets,
//! - and a stable target for `fretboard diag` perf baselines.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, component::prelude::*};
use fret_core::scene::{Color, DropShadowV1, EffectChain, EffectMode, EffectQuality, EffectStep};
use fret_core::{Corners, Edges, Point, Px};
use fret_runtime::Model;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, Overflow, SizeStyle, SpacerProps};
use fret_ui_kit::{IntoUiElement, LayoutRefinement, Space, ui};
use fret_ui_shadcn::facade as shadcn;

fn srgb(r: u8, g: u8, b: u8, a: f32) -> Color {
    let mut c = fret_ui_kit::colors::linear_from_hex_rgb(
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
    );
    c.a = a.clamp(0.0, 1.0);
    c
}

fn shadow_chain() -> EffectChain {
    EffectChain::from_steps(&[EffectStep::DropShadowV1(DropShadowV1 {
        offset_px: Point::new(Px(0.0), Px(8.0)),
        blur_radius_px: Px(10.0),
        downsample: 2,
        color: srgb(0, 0, 0, 0.55),
    })])
    .sanitize()
}

fn install_demo_theme(app: &mut KernelApp) {
    shadcn::themes::apply_shadcn_new_york(
        app,
        shadcn::themes::ShadcnBaseColor::Slate,
        shadcn::themes::ShadcnColorScheme::Dark,
    );
}

fn card<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: Arc<str>,
    subtitle: Arc<str>,
    enabled: bool,
) -> impl IntoUiElement<H> + use<H> {
    let radius = Px(16.0);

    let mut outer_layout = LayoutStyle::default();
    outer_layout.size.width = Length::Px(Px(260.0));
    outer_layout.size.height = Length::Px(Px(160.0));
    outer_layout.overflow = Overflow::Clip;

    let mut inner_layout = LayoutStyle::default();
    inner_layout.size.width = Length::Fill;
    inner_layout.size.height = Length::Fill;

    let content = move |cx: &mut ElementContext<'_, H>| {
        let card = cx.container(
            ContainerProps {
                layout: inner_layout,
                padding: Edges::all(Px(14.0)).into(),
                background: Some(srgb(248, 250, 252, 0.90)),
                border: Edges::all(Px(1.0)),
                border_color: Some(srgb(226, 232, 240, 0.90)),
                corner_radii: Corners::all(radius),
                ..Default::default()
            },
            move |cx| {
                vec![
                    shadcn::raw::typography::large(title.clone()).into_element(cx),
                    shadcn::raw::typography::muted(subtitle.clone()).into_element(cx),
                    cx.spacer(SpacerProps::default()),
                    shadcn::Badge::new("DropShadowV1")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                ]
            },
        );
        vec![card]
    };

    if !enabled {
        return cx.container(
            ContainerProps {
                layout: outer_layout,
                ..Default::default()
            },
            content,
        );
    }

    cx.effect_layer_props(
        fret_ui::element::EffectLayerProps {
            layout: outer_layout,
            mode: EffectMode::FilterContent,
            chain: shadow_chain(),
            quality: EffectQuality::Auto,
        },
        move |cx| {
            // Leave some margin inside the effect bounds so the shadow can be visible
            // (DropShadowV1 does not implicitly expand beyond computation bounds).
            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: fret_ui::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::all(Px(22.0)).into(),
                    ..Default::default()
                },
                content,
            )]
        },
    )
}

#[derive(Clone)]
struct DropShadowDemoState {
    enabled: Model<bool>,
    stress: Model<bool>,
}

struct DropShadowDemoView {
    st: DropShadowDemoState,
}

impl View for DropShadowDemoView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        Self {
            st: DropShadowDemoState {
                enabled: app.models_mut().insert(false),
                stress: app.models_mut().insert(false),
            },
        }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let enabled = cx.watch_model(&self.st.enabled).layout().value_or_default();
        let stress = cx.watch_model(&self.st.stress).layout().value_or_default();

        let stage = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                background: Some(srgb(2, 6, 23, 1.0)),
                ..Default::default()
            },
            move |cx| {
                let grid = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        padding: Edges::all(Px(32.0)).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        let cols = if stress { 4 } else { 2 };
                        let rows = if stress { 4 } else { 2 };

                        let grid = ui::v_flex(move |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            out.reserve(rows);
                            for r in 0..rows {
                                let mut row_items = Vec::with_capacity(cols);
                                for c in 0..cols {
                                    let i = r * cols + c;
                                    let card = card(
                                        cx,
                                        Arc::from(format!("Card {i}")),
                                        Arc::from("Shadow behind content (scissored)"),
                                        enabled,
                                    );
                                    row_items.push(card.into_element(cx));
                                }
                                out.push(
                                    ui::h_row(move |_cx| row_items)
                                        .gap(Space::N4)
                                        .items_center()
                                        .into_element(cx),
                                );
                            }
                            out
                        })
                        .gap(Space::N4)
                        .items_center()
                        .into_element(cx);

                        vec![
                            ui::v_flex(move |_cx| [grid])
                                .layout(LayoutRefinement::default().size_full())
                                .justify_center()
                                .items_center()
                                .into_element(cx),
                        ]
                    },
                );

                vec![grid]
            },
        );

        let inspector = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(360.0)),
                        height: Length::Fill,
                        ..Default::default()
                    },
                    flex: fret_ui::element::FlexItemStyle {
                        shrink: 0.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                padding: Edges::all(Px(16.0)).into(),
                background: Some(srgb(17, 19, 24, 0.92)),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::v_flex(|cx| {
                        vec![
                            shadcn::raw::typography::h4("Drop shadow demo").into_element(cx),
                            shadcn::raw::typography::muted(
                                "Toggle DropShadowV1 and a small stress grid.",
                            )
                            .into_element(cx),
                            shadcn::Separator::new().into_element(cx),
                            ui::h_row(|cx| {
                                [
                                    shadcn::Switch::new(self.st.enabled.clone())
                                        .a11y_label("Enable drop shadow")
                                        .test_id("drop-shadow-switch-enabled")
                                        .into_element(cx),
                                    shadcn::Label::new("Enable shadow").into_element(cx),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx),
                            ui::h_row(|cx| {
                                [
                                    shadcn::Switch::new(self.st.stress.clone())
                                        .a11y_label("Enable stress grid")
                                        .test_id("drop-shadow-switch-stress")
                                        .into_element(cx),
                                    shadcn::Label::new("Stress grid").into_element(cx),
                                ]
                            })
                            .gap(Space::N2)
                            .items_center()
                            .into_element(cx),
                            shadcn::Separator::new().into_element(cx),
                            shadcn::raw::typography::muted(
                                "Perf baseline suite: drop-shadow-v1-steady",
                            )
                            .into_element(cx),
                        ]
                    })
                    .gap(Space::N3)
                    .items_stretch()
                    .into_element(cx),
                ]
            },
        );

        let root = ui::h_flex(move |_cx| [stage, inspector])
            .layout(LayoutRefinement::default().size_full())
            .items_stretch()
            .gap(Space::N0)
            .into_element(cx)
            .test_id("drop-shadow-demo-root");

        root.into()
    }
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("drop-shadow-demo")
        .window("drop-shadow-demo", (1280.0, 720.0))
        .setup(install_demo_theme)
        .view::<DropShadowDemoView>()?
        .run()
        .map_err(anyhow::Error::from)
}

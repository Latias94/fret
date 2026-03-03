//! Drop shadow demo (EffectStep::DropShadowV1).
//!
//! This is intended as a renderer semantics validation surface:
//! - bounded multi-pass behavior (scissored computation),
//! - deterministic degradation under budgets,
//! - and a stable target for `fretboard diag` perf baselines.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use fret::legacy::prelude::*;
use fret_core::scene::{Color, DropShadowV1, EffectChain, EffectMode, EffectQuality, EffectStep};
use fret_core::{Corners, Edges, Point, Px};
use fret_runtime::Model;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, Overflow, SizeStyle, SpacerProps};
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn as shadcn;

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

fn card<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    title: Arc<str>,
    subtitle: Arc<str>,
    enabled: bool,
) -> AnyElement {
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
                    shadcn::typography::large(cx, title.clone()),
                    shadcn::typography::muted(cx, subtitle.clone()),
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

struct DropShadowDemoProgram;

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<DropShadowDemoProgram>("drop-shadow-demo")?
        .with_main_window("drop_shadow_demo", (1280.0, 720.0))
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            );
        })
        .run()?;
    Ok(())
}

impl MvuProgram for DropShadowDemoProgram {
    type State = DropShadowDemoState;
    type Message = ();

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        Self::State {
            enabled: app.models_mut().insert(false),
            stress: app.models_mut().insert(false),
        }
    }

    fn update(_app: &mut App, _st: &mut Self::State, _message: Self::Message) {}

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        _msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        let enabled = cx
            .watch_model(&st.enabled)
            .layout()
            .read_ref(|v| *v)
            .ok()
            .unwrap_or(false);
        let stress = cx
            .watch_model(&st.stress)
            .layout()
            .read_ref(|v| *v)
            .ok()
            .unwrap_or(false);

        let (rows, cols) = if stress {
            (4usize, 3usize)
        } else {
            (2usize, 3usize)
        };

        let stage = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    flex: fret_ui::element::FlexItemStyle {
                        grow: 1.0,
                        basis: Length::Px(Px(0.0)),
                        ..Default::default()
                    },
                    overflow: Overflow::Clip,
                    ..Default::default()
                },
                background: Some(srgb(9, 11, 15, 1.0)),
                ..Default::default()
            },
            move |cx| {
                let grid = shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default()
                        .gap(Space::N6)
                        .items_center(),
                    move |cx| {
                        let mut out: Vec<AnyElement> = Vec::with_capacity(rows);
                        for r in 0..rows {
                            let enabled = enabled;
                            out.push(shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N6)
                                    .items_start(),
                                move |cx| {
                                    let mut row_items: Vec<AnyElement> = Vec::with_capacity(cols);
                                    for c in 0..cols {
                                        let i = r * cols + c;
                                        row_items.push(card(
                                            cx,
                                            Arc::from(format!("Card {i}")),
                                            Arc::from("Shadow behind content (scissored)"),
                                            enabled,
                                        ));
                                    }
                                    row_items
                                },
                            ));
                        }
                        out
                    },
                );

                vec![shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default()
                        .layout(LayoutRefinement::default().size_full())
                        .justify_center()
                        .items_center(),
                    move |_cx| vec![grid],
                )]
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
                vec![shadcn::stack::vstack(
                    cx,
                    shadcn::stack::VStackProps::default()
                        .gap(Space::N3)
                        .items_stretch(),
                    |cx| {
                        vec![
                            shadcn::typography::h4(cx, "Drop shadow demo"),
                            shadcn::typography::muted(
                                cx,
                                "Toggle DropShadowV1 and a small stress grid.",
                            ),
                            shadcn::Separator::new().into_element(cx),
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Switch::new(st.enabled.clone())
                                            .a11y_label("Enable drop shadow")
                                            .test_id("drop-shadow-switch-enabled")
                                            .into_element(cx),
                                        shadcn::Label::new("Enable shadow").into_element(cx),
                                    ]
                                },
                            ),
                            shadcn::stack::hstack(
                                cx,
                                shadcn::stack::HStackProps::default()
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Switch::new(st.stress.clone())
                                            .a11y_label("Enable stress grid")
                                            .test_id("drop-shadow-switch-stress")
                                            .into_element(cx),
                                        shadcn::Label::new("Stress grid").into_element(cx),
                                    ]
                                },
                            ),
                            shadcn::Separator::new().into_element(cx),
                            shadcn::typography::muted(
                                cx,
                                "Perf baseline suite: drop-shadow-v1-steady",
                            ),
                        ]
                    },
                )]
            },
        );

        let root = shadcn::stack::hstack(
            cx,
            shadcn::stack::HStackProps::default()
                .layout(LayoutRefinement::default().size_full())
                .items_stretch()
                .gap(Space::N0),
            move |_cx| vec![stage, inspector],
        )
        .test_id("drop-shadow-demo-root");

        vec![root].into()
    }
}

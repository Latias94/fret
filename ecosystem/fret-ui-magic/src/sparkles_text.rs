use fret_core::geometry::{Corners, Edges, Px};
use fret_core::scene::{BlendMode, MaterialParams, Paint};
use fret_core::{Color, MaterialDescriptor, MaterialId, MaterialKind};
use fret_ui::element::{
    AnyElement, ContainerProps, FocusTraversalGateProps, HitTestGateProps, InsetStyle, LayoutStyle,
    Length, Overflow, PositionStyle, SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::reduced_motion_queries;
use fret_ui_kit::declarative::scheduling::set_continuous_frames;
use fret_ui_kit::recipes::catalog::VisualCatalog;
use fret_ui_kit::recipes::resolve::{
    DegradationReason, RecipeDegradedEvent, report_recipe_degraded,
};

fn rgba(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

fn material_id_from_catalog<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    desc: MaterialDescriptor,
) -> Option<MaterialId> {
    cx.app
        .global::<VisualCatalog>()
        .and_then(|cat| cat.materials.get(desc))
}

fn report_degraded_missing_material<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
) {
    report_recipe_degraded(
        cx.app,
        RecipeDegradedEvent {
            label,
            reason: DegradationReason::UnsupportedCapability,
        },
    );
}

#[derive(Debug, Clone)]
pub struct SparklesTextProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub blend: BlendMode,
    pub base: Color,
    pub sparkles: Color,
    pub cell_size: Px,
    pub density: f32,
    /// If > 0, overrides the default sparkle radius (in pixels).
    pub sparkle_radius: Px,
    pub seed: u32,
    /// If true and the runner provides a frame clock snapshot, the sparkle field animates.
    pub animate: bool,
}

impl Default for SparklesTextProps {
    fn default() -> Self {
        let layout = LayoutStyle::default();
        Self {
            layout,
            padding: Edges::all(Px(0.0)),
            corner_radii: Corners::all(Px(0.0)),
            blend: BlendMode::Add,
            base: Color::TRANSPARENT,
            sparkles: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.55,
            },
            cell_size: Px(26.0),
            density: 0.22,
            sparkle_radius: Px(0.0),
            seed: 0,
            animate: true,
        }
    }
}

/// A Phase 0 SparklesText-like wrapper.
///
/// This draws a deterministic “sparkle field” material in an additive compositing group over the
/// child content. It does **not** yet clip sparkles to glyph alpha (requires a richer alpha mask
/// substrate than v1 gradient masks).
pub fn sparkles_text<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: SparklesTextProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.sparkles_text.sparkle_field";
    let desc = MaterialDescriptor::new(MaterialKind::Sparkle);

    let prefers_reduced_motion =
        reduced_motion_queries::prefers_reduced_motion(cx, Invalidation::Paint, false);

    let clock = cx
        .app
        .global::<fret_core::WindowFrameClockService>()
        .and_then(|svc| svc.snapshot(cx.window));

    let can_animate = props.animate && !prefers_reduced_motion && clock.is_some();
    set_continuous_frames(cx, can_animate);
    if can_animate {
        cx.notify_for_animation_frame();
    }

    let t = clock
        .filter(|_| can_animate)
        .map(|clock| clock.now_monotonic.as_secs_f32())
        .unwrap_or(0.0);

    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    overflow: Overflow::Clip,
                    ..props.layout
                },
                padding: props.padding.into(),
                corner_radii: props.corner_radii,
                background: None,
                ..Default::default()
            },
            children,
        );
    };

    let density = props.density.clamp(0.0, 1.0);
    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.sparkles),
            [
                props.cell_size.0,
                density,
                props.sparkle_radius.0,
                props.seed as f32,
            ],
            [t, 0.0, 0.0, 0.0],
        ],
    };

    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                overflow: Overflow::Clip,
                ..props.layout
            },
            padding: props.padding.into(),
            corner_radii: props.corner_radii,
            background: None,
            ..Default::default()
        },
        move |cx| {
            let mut out: Vec<AnyElement> = children(cx).into_iter().collect();

            let overlay_layout = LayoutStyle {
                position: PositionStyle::Absolute,
                inset: InsetStyle {
                    top: Some(Px(0.0)).into(),
                    right: Some(Px(0.0)).into(),
                    bottom: Some(Px(0.0)).into(),
                    left: Some(Px(0.0)).into(),
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
                |cx| {
                    vec![cx.hit_test_gate_props(
                        HitTestGateProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            hit_test: false,
                        },
                        |cx| {
                            vec![cx.composite_group(props.blend, |cx| {
                                vec![cx.container(
                                    ContainerProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Fill,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        background: None,
                                        background_paint: Some(Paint::Material { id, params }),
                                        corner_radii: props.corner_radii,
                                        ..Default::default()
                                    },
                                    |_| Vec::new(),
                                )]
                            })]
                        },
                    )]
                },
            );

            out.push(overlay);
            out
        },
    )
}

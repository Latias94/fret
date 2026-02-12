use fret_core::geometry::{Corners, Edges};
use fret_core::scene::{MaterialParams, Paint};
use fret_core::{Color, MaterialDescriptor, MaterialId, MaterialKind, Px};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle};
use fret_ui::{ElementContext, UiHost};

use fret_ui_kit::recipes::catalog::VisualCatalog;
use fret_ui_kit::recipes::resolve::{
    DegradationReason, RecipeDegradedEvent, report_recipe_degraded,
};

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

fn rgba(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

#[derive(Debug, Clone, Copy)]
pub struct DotPatternProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub dots: Color,
    pub spacing: Px,
    pub radius: Px,
    pub seed: u32,
}

impl Default for DotPatternProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            dots: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.16,
            },
            spacing: Px(18.0),
            radius: Px(2.2),
            seed: 0,
        }
    }
}

pub fn dot_pattern<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: DotPatternProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.pattern.dot_grid";
    let desc = MaterialDescriptor::new(MaterialKind::DotGrid);
    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: props.layout,
                padding: props.padding,
                background: Some(props.base),
                corner_radii: props.corner_radii,
                ..Default::default()
            },
            children,
        );
    };

    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.dots),
            [
                props.spacing.0,
                props.spacing.0,
                props.radius.0,
                props.seed as f32,
            ],
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    cx.container(
        ContainerProps {
            layout: props.layout,
            padding: props.padding,
            background: None,
            background_paint: Some(Paint::Material { id, params }),
            corner_radii: props.corner_radii,
            ..Default::default()
        },
        children,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct GridPatternProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub lines: Color,
    pub spacing: (Px, Px),
    pub line_width: Px,
    pub seed: u32,
}

impl Default for GridPatternProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            lines: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.12,
            },
            spacing: (Px(22.0), Px(22.0)),
            line_width: Px(1.0),
            seed: 0,
        }
    }
}

pub fn grid_pattern<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: GridPatternProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.pattern.grid";
    let desc = MaterialDescriptor::new(MaterialKind::Grid);
    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: props.layout,
                padding: props.padding,
                background: Some(props.base),
                corner_radii: props.corner_radii,
                ..Default::default()
            },
            children,
        );
    };

    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.lines),
            [
                props.spacing.0.0,
                props.spacing.1.0,
                props.line_width.0,
                props.seed as f32,
            ],
            [0.0, 0.0, 0.0, 0.0],
        ],
    };

    cx.container(
        ContainerProps {
            layout: props.layout,
            padding: props.padding,
            background: None,
            background_paint: Some(Paint::Material { id, params }),
            corner_radii: props.corner_radii,
            ..Default::default()
        },
        children,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct StripePatternProps {
    pub layout: LayoutStyle,
    pub padding: Edges,
    pub corner_radii: Corners,
    pub base: Color,
    pub stripes: Color,
    pub spacing: Px,
    pub stripe_width: Px,
    pub angle_radians: f32,
    pub seed: u32,
}

impl Default for StripePatternProps {
    fn default() -> Self {
        let mut layout = LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Fill;
        Self {
            layout,
            padding: Edges::all(Px(16.0)),
            corner_radii: Corners::all(Px(12.0)),
            base: Color::TRANSPARENT,
            stripes: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.10,
            },
            spacing: Px(20.0),
            stripe_width: Px(6.0),
            angle_radians: std::f32::consts::FRAC_PI_4,
            seed: 0,
        }
    }
}

pub fn stripe_pattern<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: StripePatternProps,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let label = "magic.pattern.stripe";
    let desc = MaterialDescriptor::new(MaterialKind::Stripe);
    let Some(id) = material_id_from_catalog(cx, desc) else {
        report_degraded_missing_material(cx, label);
        return cx.container(
            ContainerProps {
                layout: props.layout,
                padding: props.padding,
                background: Some(props.base),
                corner_radii: props.corner_radii,
                ..Default::default()
            },
            children,
        );
    };

    let params = MaterialParams {
        vec4s: [
            rgba(props.base),
            rgba(props.stripes),
            [
                props.spacing.0,
                0.0,
                props.stripe_width.0,
                props.seed as f32,
            ],
            [0.0, props.angle_radians, 0.0, 0.0],
        ],
    };

    cx.container(
        ContainerProps {
            layout: props.layout,
            padding: props.padding,
            background: None,
            background_paint: Some(Paint::Material { id, params }),
            corner_radii: props.corner_radii,
            ..Default::default()
        },
        children,
    )
}

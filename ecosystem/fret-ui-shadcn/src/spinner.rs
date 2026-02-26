use fret_core::{Point, Px, Transform2D};
use fret_icons::{IconId, ids};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SvgIconProps, VisualTransformProps};
use fret_ui::{ElementContext, Invalidation, SvgSource, Theme, UiHost};
use fret_ui_kit::declarative::icon as icon_runtime;
use fret_ui_kit::declarative::prefers_reduced_motion;
use fret_ui_kit::declarative::scheduling;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ColorRef, LayoutRefinement};

/// shadcn/ui `Spinner` (v4).
///
/// Upstream uses a spinning lucide icon (`Loader2Icon` + `animate-spin`). In Fret, we implement
/// this via a paint-only `VisualTransform` wrapper around an `SvgIcon`, and request animation
/// frames while the spinner is rendered.
#[derive(Debug, Clone)]
pub struct Spinner {
    layout: LayoutRefinement,
    color: Option<ColorRef>,
    icon: IconId,
    /// Rotation speed in radians per frame. (`0.0` disables animation.)
    speed: f32,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            color: None,
            icon: ids::ui::LOADER,
            speed: 0.12,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = icon;
        self
    }

    /// Rotation speed in radians per frame. (`0.0` disables animation.)
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let base_layout = LayoutRefinement::default()
            .w_px(Px(16.0))
            .h_px(Px(16.0))
            .flex_shrink_0();

        let layout = decl_style::layout_style(&theme, base_layout.merge(self.layout));
        let (color, inherit_color) = if let Some(color) = self.color {
            (color.resolve(&theme), false)
        } else {
            (
                theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_token("foreground")),
                true,
            )
        };

        let svg: SvgSource = icon_runtime::resolve_svg_source_from_globals(
            cx.app,
            &self.icon,
            "fret_ui_shadcn.spinner",
        );

        let mut center = Point::new(Px(8.0), Px(8.0));
        if let (Length::Px(w), Length::Px(h)) = (layout.size.width, layout.size.height) {
            center = Point::new(Px(w.0 * 0.5), Px(h.0 * 0.5));
        }

        let reduced_motion = prefers_reduced_motion(cx, Invalidation::Paint, false);
        let speed = if reduced_motion { 0.0 } else { self.speed };
        let angle = cx.app.frame_id().0 as f32 * speed;
        scheduling::set_continuous_frames(cx, speed != 0.0);
        let transform = Transform2D::rotation_about_radians(angle, center);

        cx.visual_transform_props(VisualTransformProps { layout, transform }, |cx| {
            let mut props = SvgIconProps::new(svg);
            props.layout = {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout
            };
            props.color = color;
            props.inherit_color = inherit_color;
            vec![cx.svg_icon_props(props)]
        })
    }
}

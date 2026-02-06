use fret_core::{Point, Px, Transform2D};
use fret_icons::{FrozenIconRegistry, IconId, IconRegistry, ResolvedSvgOwned};
use fret_ui::element::{AnyElement, Length, SvgIconProps, VisualTransformProps};
use fret_ui::{ElementContext, SvgSource, Theme, UiHost};
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
            icon: IconId::new_static("lucide.loader-circle"),
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_layout = LayoutRefinement::default()
            .w_px(Px(16.0))
            .h_px(Px(16.0))
            .flex_shrink_0();

        let layout = decl_style::layout_style(&theme, base_layout.merge(self.layout));
        let color = self
            .color
            .map(|c| c.resolve(&theme))
            .or_else(|| theme.color_by_key("foreground"))
            .unwrap_or_else(|| theme.color_required("foreground"));

        let resolved = cx
            .app
            .global::<FrozenIconRegistry>()
            .map(|frozen| frozen.resolve_or_missing_owned(&self.icon))
            .unwrap_or_else(|| {
                cx.app.with_global_mut(IconRegistry::default, |icons, app| {
                    let frozen = icons.freeze().unwrap_or_default();
                    let resolved = frozen.resolve_or_missing_owned(&self.icon);
                    app.set_global(frozen);
                    resolved
                })
            });

        let svg: SvgSource = match resolved {
            ResolvedSvgOwned::Static(bytes) => SvgSource::Static(bytes),
            ResolvedSvgOwned::Bytes(bytes) => SvgSource::Bytes(bytes),
        };

        let mut center = Point::new(Px(8.0), Px(8.0));
        if let (Length::Px(w), Length::Px(h)) = (layout.size.width, layout.size.height) {
            center = Point::new(Px(w.0 * 0.5), Px(h.0 * 0.5));
        }

        let angle = cx.app.frame_id().0 as f32 * self.speed;
        scheduling::set_continuous_frames(cx, self.speed != 0.0);
        let transform = Transform2D::rotation_about_radians(angle, center);

        cx.visual_transform_props(VisualTransformProps { layout, transform }, |cx| {
            let mut props = SvgIconProps::new(svg);
            props.color = color;
            vec![cx.svg_icon_props(props)]
        })
    }
}

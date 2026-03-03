use std::f32::consts::TAU;
use std::time::Duration;

use fret_core::{Point, Px, SemanticsLive, SemanticsRole, Transform2D};
use fret_icons::{IconId, ids};
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, SemanticsDecoration, SvgIconProps, VisualTransformProps,
};
use fret_ui::{ElementContext, SvgSource, Theme, UiHost};
use fret_ui_kit::declarative::icon as icon_runtime;
use fret_ui_kit::declarative::motion;
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
    /// Rotation speed in radians per 60Hz tick. (`0.0` disables animation.)
    ///
    /// This is duration-driven under the hood, so the perceived speed remains stable under
    /// `--fixed-frame-delta-ms` and across different refresh rates.
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

    /// Rotation speed in radians per 60Hz tick. (`0.0` disables animation.)
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

        let angular_speed = self.speed * 60.0;
        let period = if angular_speed.abs() > 0.0 {
            Duration::from_secs_f32((TAU / angular_speed.abs()).max(0.0))
        } else {
            Duration::ZERO
        };
        let spin = motion::drive_loop_progress_keyed(
            cx,
            ("shadcn.spinner.spin", cx.root_id()),
            angular_speed.abs() > 0.0,
            period,
        );
        let mut angle = TAU * spin.progress;
        if angular_speed < 0.0 {
            angle = -angle;
        }
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
        .attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::ProgressBar)
                .label("Loading")
                .busy(true)
                .live(Some(SemanticsLive::Polite)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size, Transform2D, WindowFrameClockService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::element::{ElementKind, Length};
    use fret_ui::elements;
    use fret_ui::elements::ElementRuntime;
    use std::time::Duration;

    #[test]
    fn spinner_defaults_to_size_4_and_has_loading_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Slate,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));

        let el = elements::with_element_cx(&mut app, window, bounds, "spinner", |cx| {
            Spinner::new().into_element(cx)
        });

        let ElementKind::VisualTransform(props) = &el.kind else {
            panic!("expected Spinner to build a VisualTransform wrapper");
        };
        assert_eq!(props.layout.size.width, Length::Px(Px(16.0)));
        assert_eq!(props.layout.size.height, Length::Px(Px(16.0)));

        let sem = el
            .semantics_decoration
            .as_ref()
            .expect("semantics decoration");
        assert_eq!(sem.role, Some(SemanticsRole::ProgressBar));
        assert_eq!(sem.label.as_deref(), Some("Loading"));
        assert_eq!(sem.busy, Some(true));
        assert_eq!(sem.live, Some(Some(SemanticsLive::Polite)));

        let child = el.children.first().expect("visual transform child");
        let ElementKind::SvgIcon(_) = &child.kind else {
            panic!("expected Spinner child to be an SvgIcon");
        };
    }

    #[test]
    fn spinner_rotation_advances_with_fixed_delta_and_respects_reduced_motion() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.with_global_mut(WindowFrameClockService::default, |svc, _app| {
            svc.set_fixed_delta(window, Some(Duration::from_millis(16)));
        });

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(80.0)));

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));
        let t0 = elements::with_element_cx(&mut app, window, bounds, "spinner", |cx| {
            Spinner::new().into_element(cx)
        });
        let ElementKind::VisualTransform(p0) = &t0.kind else {
            panic!("expected Spinner to build a VisualTransform wrapper");
        };
        assert_eq!(
            p0.transform,
            Transform2D::IDENTITY,
            "expected initial rotation to start at identity"
        );

        app.set_tick_id(TickId(2));
        app.set_frame_id(FrameId(2));
        let t1 = elements::with_element_cx(&mut app, window, bounds, "spinner", |cx| {
            Spinner::new().into_element(cx)
        });
        let ElementKind::VisualTransform(p1) = &t1.kind else {
            panic!("expected Spinner to build a VisualTransform wrapper");
        };
        assert_ne!(
            p1.transform,
            Transform2D::IDENTITY,
            "expected spinner rotation to advance across frames under fixed delta"
        );

        app.with_global_mut(ElementRuntime::new, |rt, _app| {
            rt.set_window_prefers_reduced_motion(window, Some(true));
        });

        app.set_tick_id(TickId(3));
        app.set_frame_id(FrameId(3));
        let r0 = elements::with_element_cx(&mut app, window, bounds, "spinner", |cx| {
            Spinner::new().into_element(cx)
        });
        let ElementKind::VisualTransform(r0p) = &r0.kind else {
            panic!("expected Spinner to build a VisualTransform wrapper");
        };
        assert_eq!(
            r0p.transform,
            Transform2D::IDENTITY,
            "expected reduced motion to stop spinner rotation"
        );

        app.set_tick_id(TickId(4));
        app.set_frame_id(FrameId(4));
        let r1 = elements::with_element_cx(&mut app, window, bounds, "spinner", |cx| {
            Spinner::new().into_element(cx)
        });
        let ElementKind::VisualTransform(r1p) = &r1.kind else {
            panic!("expected Spinner to build a VisualTransform wrapper");
        };
        assert_eq!(
            r1p.transform,
            Transform2D::IDENTITY,
            "expected reduced motion to keep spinner rotation disabled across frames"
        );
    }
}

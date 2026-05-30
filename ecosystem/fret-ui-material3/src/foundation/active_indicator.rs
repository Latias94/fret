//! Shared Material active-indicator motion and paint helpers.

use std::sync::Arc;

use fret_core::{Color, Corners, DrawOrder, Point, Px, Rect, Size};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, CanvasProps, Length, PositionStyle, SemanticsDecoration};
use fret_ui::elements::ElementContext;
use fret_ui_headless::motion::spring::SpringDescription;
use fret_ui_headless::motion::tolerance::Tolerance;
use fret_ui_kit::declarative::motion_value::{
    MotionToSpecF32, MotionValueF32Update, SpringSpecF32, drive_motion_value_f32,
};

use crate::motion::SpringSpec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ActiveIndicatorRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ActiveIndicatorRect {
    pub(crate) const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[cfg(test)]
    pub(crate) const fn empty() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
}

pub(crate) fn material_active_indicator_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    target: ActiveIndicatorRect,
    color: Color,
    corner_radii: Corners,
    spring: SpringSpec,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let spec = spring_motion_spec(spring);

    let x = drive_motion_value_f32(
        cx,
        target.x,
        MotionValueF32Update::To {
            target: target.x,
            spec,
            kick: None,
        },
    );
    let y = drive_motion_value_f32(
        cx,
        target.y,
        MotionValueF32Update::To {
            target: target.y,
            spec,
            kick: None,
        },
    );
    let width = drive_motion_value_f32(
        cx,
        target.width,
        MotionValueF32Update::To {
            target: target.width,
            spec,
            kick: None,
        },
    );
    let height = drive_motion_value_f32(
        cx,
        target.height,
        MotionValueF32Update::To {
            target: target.height,
            spec,
            kick: None,
        },
    );

    let mut props = CanvasProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.layout.size.min_width = Some(Length::Px(Px((target.x + target.width).max(0.0))));
    props.layout.size.min_height = Some(Length::Px(Px((target.y + target.height).max(0.0))));
    props.layout.inset.top = Some(Px(0.0)).into();
    props.layout.inset.right = Some(Px(0.0)).into();
    props.layout.inset.bottom = Some(Px(0.0)).into();
    props.layout.inset.left = Some(Px(0.0)).into();

    let mut indicator = cx.canvas(props, move |p| {
        if color.a <= 0.0 {
            return;
        }

        let target = ActiveIndicatorRect::new(x.value, y.value, width.value, height.value);
        let Some(rect) = clamped_active_indicator_rect(p.bounds(), target) else {
            return;
        };

        fret_ui::paint::paint_state_layer(p.scene(), DrawOrder(0), rect, color, 1.0, corner_radii);
    });

    if let Some(test_id) = test_id {
        indicator = indicator.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    indicator
}

fn spring_motion_spec(spring: SpringSpec) -> MotionToSpecF32 {
    let spring =
        SpringDescription::with_damping_ratio(1.0, spring.stiffness as f64, spring.damping as f64);
    MotionToSpecF32::Spring(SpringSpecF32 {
        spring,
        tolerance: Tolerance::default(),
        snap_to_target: true,
    })
}

fn clamped_active_indicator_rect(bounds: Rect, target: ActiveIndicatorRect) -> Option<Rect> {
    if target.width <= 0.0 || target.height <= 0.0 {
        return None;
    }

    let x = target.x.clamp(0.0, bounds.size.width.0);
    let y = target.y.clamp(0.0, bounds.size.height.0);
    let max_width = (bounds.size.width.0 - x).max(0.0);
    let max_height = (bounds.size.height.0 - y).max(0.0);
    let width = target.width.clamp(0.0, max_width);
    let height = target.height.clamp(0.0, max_height);

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some(Rect::new(
        Point::new(Px(bounds.origin.x.0 + x), Px(bounds.origin.y.0 + y)),
        Size::new(Px(width), Px(height)),
    ))
}

#[cfg(test)]
mod tests {
    use super::{ActiveIndicatorRect, clamped_active_indicator_rect};
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn active_indicator_rect_clamps_to_canvas_bounds() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(100.0), Px(40.0)),
        );

        let rect =
            clamped_active_indicator_rect(bounds, ActiveIndicatorRect::new(80.0, 30.0, 40.0, 20.0))
                .expect("expected visible rect");

        assert_eq!(rect.origin.x, Px(90.0));
        assert_eq!(rect.origin.y, Px(50.0));
        assert_eq!(rect.size.width, Px(20.0));
        assert_eq!(rect.size.height, Px(10.0));
    }

    #[test]
    fn active_indicator_rect_omits_empty_targets() {
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));

        assert!(clamped_active_indicator_rect(bounds, ActiveIndicatorRect::empty()).is_none());
        assert!(
            clamped_active_indicator_rect(bounds, ActiveIndicatorRect::new(100.0, 0.0, 20.0, 10.0))
                .is_none()
        );
    }
}

use super::super::DebugDrawCommandKind;

pub(super) enum DebugDrawListSummaryClass {
    ClipPush,
    ClipPop,
    Image,
    Svg,
    Text,
    Geometry,
}

pub(super) fn classify_debug_draw_summary_kind(
    kind: DebugDrawCommandKind,
) -> DebugDrawListSummaryClass {
    match kind {
        DebugDrawCommandKind::PushClipRect => DebugDrawListSummaryClass::ClipPush,
        DebugDrawCommandKind::PopClipRect => DebugDrawListSummaryClass::ClipPop,
        DebugDrawCommandKind::Image
        | DebugDrawCommandKind::ImageRegion
        | DebugDrawCommandKind::ImageQuad
        | DebugDrawCommandKind::ImageRounded
        | DebugDrawCommandKind::ImageRegionRounded
        | DebugDrawCommandKind::ImageTriangleMesh => DebugDrawListSummaryClass::Image,
        DebugDrawCommandKind::SvgImage | DebugDrawCommandKind::SvgMaskIcon => {
            DebugDrawListSummaryClass::Svg
        }
        DebugDrawCommandKind::Text => DebugDrawListSummaryClass::Text,
        DebugDrawCommandKind::Line
        | DebugDrawCommandKind::Polyline
        | DebugDrawCommandKind::ConvexPolyFilled
        | DebugDrawCommandKind::ConcavePolyFilled
        | DebugDrawCommandKind::Rect
        | DebugDrawCommandKind::RectFilled
        | DebugDrawCommandKind::RectFilledMultiColor
        | DebugDrawCommandKind::Quad
        | DebugDrawCommandKind::QuadFilled
        | DebugDrawCommandKind::Triangle
        | DebugDrawCommandKind::TriangleFilled
        | DebugDrawCommandKind::TriangleMesh
        | DebugDrawCommandKind::Circle
        | DebugDrawCommandKind::CircleFilled
        | DebugDrawCommandKind::Ngon
        | DebugDrawCommandKind::NgonFilled
        | DebugDrawCommandKind::Ellipse
        | DebugDrawCommandKind::EllipseFilled
        | DebugDrawCommandKind::BezierQuadratic
        | DebugDrawCommandKind::BezierCubic => DebugDrawListSummaryClass::Geometry,
    }
}

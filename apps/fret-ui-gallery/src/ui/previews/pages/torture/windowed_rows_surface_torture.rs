use super::super::super::super::*;

pub(in crate::ui) fn preview_windowed_rows_surface_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfaceProps, windowed_rows_surface,
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline scroll windowing via a stable element tree (Scroll + Canvas)."),
                cx.text("This is the 'single-node surface' escape hatch: paint only visible rows, avoid per-row subtrees."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let mut props = WindowedRowsSurfaceProps::default();
            props.scroll.layout.size.width = fret_ui::element::Length::Fill;
            props.scroll.layout.size.height = fret_ui::element::Length::Px(Px(420.0));
            props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
            props.len = len;
            props.row_height = row_h;
            props.overscan = overscan;
            props.scroll_handle = scroll_handle.clone();
            props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let surface = windowed_rows_surface(cx, props, move |painter, index, rect| {
                let background = if (index % 2) == 0 { bg_even } else { bg_odd };
                painter.scene().push(fret_core::SceneOp::Quad {
                    order: DrawOrder(0),
                    rect,
                    background: fret_core::Paint::Solid(background),
                    border: Edges::all(Px(0.0)),
                    border_paint: fret_core::Paint::TRANSPARENT,

                    corner_radii: Corners::all(Px(0.0)),
                });

                let label = format!("Row {index}");
                let origin =
                    fret_core::Point::new(Px(rect.origin.x.0 + 8.0), Px(rect.origin.y.0 + 4.0));
                let scope = painter.key_scope(&"ui-gallery-windowed-rows");
                let key: u64 = painter.child_key(scope, &index).into();
                let _ = painter.text(
                    key,
                    DrawOrder(1),
                    origin,
                    label,
                    text_style.clone(),
                    fg,
                    CanvasTextConstraints {
                        max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    },
                    painter.scale_factor(),
                );
            });

            vec![
                surface.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id("ui-gallery-windowed-rows-root"),
                ),
            ]
        });

    vec![header, surface]
}

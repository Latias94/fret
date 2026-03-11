use super::super::*;

fn selected_group_rects(groups: &[(Rect, Arc<str>, bool)]) -> Vec<Rect> {
    groups
        .iter()
        .filter_map(|(rect, _title, selected)| selected.then_some(*rect))
        .collect()
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn paint_selected_group_overlay_rects<I>(
        &mut self,
        scene: &mut fret_core::Scene,
        rects: I,
        zoom: f32,
    ) where
        I: IntoIterator<Item = Rect>,
    {
        let corner_radius = Px(10.0 / zoom);
        let border_width = Px(1.0 / zoom);
        for rect in rects {
            scene.push(SceneOp::Quad {
                order: DrawOrder(1),
                rect,
                background: fret_core::Paint::Solid(self.style.paint.group_background).into(),
                border: Edges::all(border_width),
                border_paint: fret_core::Paint::Solid(self.style.paint.node_border_selected).into(),
                corner_radii: Corners::all(corner_radius),
            });
        }
    }

    pub(in super::super) fn paint_groups_selected_overlay(
        &mut self,
        scene: &mut fret_core::Scene,
        groups: &[(Rect, Arc<str>, bool)],
        zoom: f32,
    ) {
        if groups.is_empty() {
            return;
        }

        self.paint_selected_group_overlay_rects(scene, selected_group_rects(groups), zoom);
    }

    pub(in super::super) fn paint_selected_groups_overlay_from_snapshot<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render_cull_rect: Option<Rect>,
        zoom: f32,
    ) {
        let selected_groups = snapshot.selected_groups.clone();
        let mut overlay_rects = Vec::with_capacity(selected_groups.len());
        let _ = self.graph.read_ref(cx.app, |graph| {
            for group_id in &selected_groups {
                let Some(group) = graph.groups.get(group_id) else {
                    continue;
                };
                let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                let rect = Rect::new(
                    Point::new(Px(rect0.origin.x), Px(rect0.origin.y)),
                    Size::new(Px(rect0.size.width), Px(rect0.size.height)),
                );
                if render_cull_rect.is_some_and(|cull_rect| !rects_intersect(rect, cull_rect)) {
                    continue;
                }
                overlay_rects.push(rect);
            }
        });

        self.paint_selected_group_overlay_rects(cx.scene, overlay_rects, zoom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_group_rects_keeps_only_selected_entries() {
        let rect_a = Rect::new(Point::new(Px(1.0), Px(2.0)), Size::new(Px(3.0), Px(4.0)));
        let rect_b = Rect::new(Point::new(Px(5.0), Px(6.0)), Size::new(Px(7.0), Px(8.0)));
        let rect_c = Rect::new(Point::new(Px(9.0), Px(10.0)), Size::new(Px(11.0), Px(12.0)));
        let groups = vec![
            (rect_a, Arc::<str>::from("A"), false),
            (rect_b, Arc::<str>::from("B"), true),
            (rect_c, Arc::<str>::from("C"), true),
        ];

        assert_eq!(selected_group_rects(&groups), vec![rect_b, rect_c]);
    }
}

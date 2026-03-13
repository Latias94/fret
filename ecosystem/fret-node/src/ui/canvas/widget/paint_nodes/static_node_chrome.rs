#[path = "static_node_chrome/quads.rs"]
mod quads;
#[path = "static_node_chrome/shadow.rs"]
mod shadow;
#[path = "static_node_chrome/style.rs"]
mod style;
#[path = "static_node_chrome/text.rs"]
mod text;

use crate::ui::canvas::widget::*;
use fret_core::TextStyle;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_static_node(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        node: GraphNodeId,
        rect: Rect,
        is_selected: bool,
        title: &Arc<str>,
        body: Option<&Arc<str>>,
        pin_rows: usize,
        hint: crate::ui::NodeChromeHint,
        node_text_style: &TextStyle,
        zoom: f32,
        corner: Px,
        title_pad: f32,
        title_h: f32,
    ) {
        let border_w = Px(1.0 / zoom);
        let paint_style = style::resolve_static_node_paint_style(self, node, is_selected, hint);

        let shadow_pushed = shadow::push_static_node_shadow(scene, rect, zoom, paint_style.shadow);

        quads::paint_static_node_quads(
            scene,
            rect,
            paint_style.body_background,
            paint_style.header_background,
            paint_style.border_paint,
            border_w,
            corner,
            title_h,
        );

        if shadow_pushed {
            scene.push(SceneOp::PopEffect);
        }

        text::paint_static_node_title(
            self,
            scene,
            services,
            scale_factor,
            rect,
            title,
            paint_style.title_text,
            node_text_style,
            zoom,
            title_pad,
            title_h,
        );
        text::paint_static_node_body(
            self,
            scene,
            services,
            scale_factor,
            rect,
            body,
            pin_rows,
            node_text_style,
            zoom,
            title_pad,
        );
    }
}

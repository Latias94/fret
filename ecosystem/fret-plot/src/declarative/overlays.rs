//! Declarative line-plot overlay paint owner.

mod annotation;
mod draggable_labels;
mod draggable_shapes;
mod images;
mod reference_lines;
mod tags;
mod text;

pub(in crate::declarative) use annotation::{
    line_plot_annotation_label, line_plot_annotation_tokens, line_plot_clamp_plot_left,
    line_plot_clamp_plot_top, paint_line_plot_annotation_text_box, paint_line_plot_tag_x_overlay,
    paint_line_plot_tag_y_overlay,
};
pub(super) use draggable_labels::paint_line_plot_draggable_overlay_labels;
pub(super) use draggable_shapes::paint_line_plot_draggable_shapes;
pub(super) use images::paint_line_plot_images;
pub(super) use reference_lines::paint_line_plot_reference_lines;
pub(super) use tags::paint_line_plot_tag_overlays;
pub(super) use text::paint_line_plot_text_overlays;

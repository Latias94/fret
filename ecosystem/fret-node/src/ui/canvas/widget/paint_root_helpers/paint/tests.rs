use super::append_static_scene_paint_style_key;
use crate::ui::style::NodeGraphStyle;
use fret_canvas::cache::TileCacheKeyBuilder;

#[test]
fn append_static_scene_paint_style_key_changes_when_paint_tokens_change() {
    let mut style = NodeGraphStyle::default();
    let mut a = TileCacheKeyBuilder::new("paint-root.paint");
    append_static_scene_paint_style_key(&mut a, &style);
    let a = a.finish();

    style.paint.context_menu_text.a = 0.5;
    let mut b = TileCacheKeyBuilder::new("paint-root.paint");
    append_static_scene_paint_style_key(&mut b, &style);
    let b = b.finish();

    assert_ne!(a, b);
}

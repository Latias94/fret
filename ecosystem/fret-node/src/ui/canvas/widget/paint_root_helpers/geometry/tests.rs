use super::append_static_scene_geometry_style_key;
use crate::ui::style::NodeGraphStyle;
use fret_canvas::cache::TileCacheKeyBuilder;

#[test]
fn append_static_scene_geometry_style_key_changes_when_scale_factor_changes() {
    let style = NodeGraphStyle::default();

    let mut a = TileCacheKeyBuilder::new("paint-root.geometry");
    append_static_scene_geometry_style_key(&mut a, &style, 1.0);
    let a = a.finish();

    let mut b = TileCacheKeyBuilder::new("paint-root.geometry");
    append_static_scene_geometry_style_key(&mut b, &style, 2.0);
    let b = b.finish();

    assert_ne!(a, b);
}

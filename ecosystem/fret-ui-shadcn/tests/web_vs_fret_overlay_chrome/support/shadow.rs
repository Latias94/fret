use super::*;

#[path = "../../support/shadow_insets.rs"]
mod shadow_insets_shared;
pub(crate) use shadow_insets_shared::*;

pub(crate) fn web_drop_shadow_insets(node: &WebNode) -> Vec<ShadowInsets> {
    let box_shadow = node
        .computed_style
        .get("boxShadow")
        .map(String::as_str)
        .unwrap_or("");
    shadow_insets_from_box_shadow(box_shadow, |color| {
        parse_css_color(color).map(|rgba| rgba.a)
    })
}

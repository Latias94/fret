//! Material token coverage audit helpers used by crate-level conformance tests.

use fret_ui::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MaterialTokenUse {
    pub(crate) source: &'static str,
    pub(crate) key: &'static str,
}

pub(crate) fn literal_md_token_uses() -> Vec<MaterialTokenUse> {
    let mut uses = Vec::new();
    for (source, text) in material_token_sources() {
        uses.extend(
            extract_md_literal_keys(text)
                .into_iter()
                .map(|key| MaterialTokenUse { source, key }),
        );
    }
    uses.sort_unstable_by(|a, b| a.key.cmp(b.key).then_with(|| a.source.cmp(b.source)));
    uses.dedup_by(|a, b| a.key == b.key);
    uses
}

pub(crate) fn token_resolves(theme: &Theme, key: &str) -> bool {
    theme.color_by_key(key).is_some()
        || theme.metric_by_key(key).is_some()
        || theme.number_by_key(key).is_some()
        || theme.duration_ms_by_key(key).is_some()
        || theme.easing_by_key(key).is_some()
        || theme.corners_by_key(key).is_some()
        || theme.text_style_by_key(key).is_some()
}

fn material_token_sources() -> &'static [(&'static str, &'static str)] {
    &[
        ("button.rs", include_str!("../button.rs")),
        ("card.rs", include_str!("../card.rs")),
        ("checkbox.rs", include_str!("../checkbox.rs")),
        ("chip.rs", include_str!("../chip.rs")),
        ("date_picker.rs", include_str!("../date_picker.rs")),
        ("dialog.rs", include_str!("../dialog.rs")),
        ("dropdown_menu.rs", include_str!("../dropdown_menu.rs")),
        ("icon_button.rs", include_str!("../icon_button.rs")),
        ("list.rs", include_str!("../list.rs")),
        ("menu.rs", include_str!("../menu.rs")),
        (
            "modal_navigation_drawer.rs",
            include_str!("../modal_navigation_drawer.rs"),
        ),
        ("navigation_bar.rs", include_str!("../navigation_bar.rs")),
        (
            "navigation_drawer.rs",
            include_str!("../navigation_drawer.rs"),
        ),
        ("navigation_rail.rs", include_str!("../navigation_rail.rs")),
        ("slider.rs", include_str!("../slider.rs")),
        ("radio.rs", include_str!("../radio.rs")),
        ("search_bar.rs", include_str!("../search_bar.rs")),
        ("search_view.rs", include_str!("../search_view.rs")),
        ("select.rs", include_str!("../select.rs")),
        ("snackbar.rs", include_str!("../snackbar.rs")),
        ("switch.rs", include_str!("../switch.rs")),
        ("tabs.rs", include_str!("../tabs.rs")),
        ("text_field.rs", include_str!("../text_field.rs")),
        ("tooltip.rs", include_str!("../tooltip.rs")),
        (
            "foundation/indication.rs",
            include_str!("../foundation/indication.rs"),
        ),
        (
            "foundation/focus_ring.rs",
            include_str!("../foundation/focus_ring.rs"),
        ),
        (
            "foundation/geometry.rs",
            include_str!("../foundation/geometry.rs"),
        ),
        (
            "foundation/interaction.rs",
            include_str!("../foundation/interaction.rs"),
        ),
        (
            "foundation/tokens.rs",
            include_str!("../foundation/tokens.rs"),
        ),
        ("tokens/icon_button.rs", include_str!("icon_button.rs")),
        ("tokens/button.rs", include_str!("button.rs")),
        ("tokens/card.rs", include_str!("card.rs")),
        ("tokens/checkbox.rs", include_str!("checkbox.rs")),
        ("tokens/chip.rs", include_str!("chip.rs")),
        ("tokens/date_picker.rs", include_str!("date_picker.rs")),
        ("tokens/switch.rs", include_str!("switch.rs")),
        ("tokens/radio.rs", include_str!("radio.rs")),
        ("tokens/dialog.rs", include_str!("dialog.rs")),
        ("tokens/snackbar.rs", include_str!("snackbar.rs")),
        ("tokens/tabs.rs", include_str!("tabs.rs")),
        ("tokens/menu.rs", include_str!("menu.rs")),
        ("tokens/text_field.rs", include_str!("text_field.rs")),
        ("tokens/list.rs", include_str!("list.rs")),
        ("tokens/dropdown_menu.rs", include_str!("dropdown_menu.rs")),
        ("tokens/select.rs", include_str!("select.rs")),
        ("tokens/search_bar.rs", include_str!("search_bar.rs")),
        ("tokens/search_view.rs", include_str!("search_view.rs")),
        ("tokens/tooltip.rs", include_str!("tooltip.rs")),
        ("tokens/slider.rs", include_str!("slider.rs")),
    ]
}

fn extract_md_literal_keys(source: &'static str) -> Vec<&'static str> {
    let mut out = Vec::new();
    let mut cursor: usize = 0;
    while let Some(idx) = source[cursor..].find("\"md.") {
        let start = cursor + idx + 1;
        let rest = &source[start..];
        let Some(end) = rest.find('"') else {
            break;
        };
        let key = &source[start..start + end];
        cursor = start + end + 1;
        if key.contains('{') || key.contains('}') || key.contains(' ') || key.contains('\n') {
            continue;
        }
        // Skip namespace/prefix strings like `md.comp.button` / `md.comp.checkbox.selected`
        // that are used to build other keys.
        // - `md.sys.*` tokens can be as short as `md.sys.color.primary` (3 dots).
        // - `md.comp.*` tokens are always deeper (at least 4 dots).
        let dot_count = key.matches('.').count();
        if key.starts_with("md.comp.") {
            if dot_count < 4 {
                continue;
            }
        } else if dot_count < 3 {
            continue;
        }
        out.push(key);
    }
    out
}

#[path = "web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

const DATE_PICKER_KEYS: &[&str] = &[
    "date-picker-demo",
    "date-picker-with-presets",
    "date-picker-with-presets.preset-tomorrow",
    "date-picker-with-range",
];

#[test]
fn shadcn_date_picker_goldens_are_targeted_gates() {
    for &key in DATE_PICKER_KEYS {
        let web = read_web_golden_open_fallback(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| class_contains(n, "lucide-calendar"))
            .expect("missing calendar icon");
        find_first(&theme.root, &|n| {
            class_contains(n, "justify-start") && class_contains(n, "text-left")
        })
        .expect("missing date picker trigger button recipe markers");
    }
}

#[test]
fn shadcn_date_picker_with_presets_preset_tomorrow_has_selected_day_and_trigger_text() {
    let web = read_web_golden_open_fallback("date-picker-with-presets.preset-tomorrow");
    let theme = web.themes.get("light").expect("missing light theme");

    find_first_in_theme(theme, &|n| {
        n.tag == "button" && contains_text(n, "January 16th, 2026")
    })
    .expect("missing updated trigger button text");

    find_first_in_theme(theme, &|n| {
        n.attrs
            .get("aria-label")
            .is_some_and(|v| v.contains("January 16th, 2026") && v.contains("selected"))
    })
    .expect("missing selected day aria-label marker");
}

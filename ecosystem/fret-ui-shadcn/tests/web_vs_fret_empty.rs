#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const EMPTY_KEYS: &[&str] = &[
    "empty-avatar",
    "empty-avatar-group",
    "empty-background",
    "empty-demo",
    "empty-icon",
    "empty-input-group",
    "empty-outline",
];

#[test]
fn shadcn_empty_goldens_are_targeted_gates() {
    for &key in EMPTY_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| {
            n.tag == "div" && class_has_all_tokens(n, &["border-dashed", "rounded-lg", "p-6"])
        })
        .expect("missing empty container (border-dashed rounded-lg p-6)");
    }
}

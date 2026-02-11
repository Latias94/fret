#[path = "web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const RESIZABLE_KEYS: &[&str] = &[
    "resizable-demo",
    "resizable-demo-with-handle",
    "resizable-handle",
    "resizable-vertical",
];

#[test]
fn shadcn_resizable_goldens_are_targeted_gates() {
    for &key in RESIZABLE_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| class_contains(n, "panel-group-direction"))
            .expect("missing resizable panel-group recipe markers");
        find_first(&theme.root, &|n| {
            class_contains(n, "bg-border") && class_contains(n, "w-px")
        })
        .expect("missing resizable handle recipe node");
    }
}

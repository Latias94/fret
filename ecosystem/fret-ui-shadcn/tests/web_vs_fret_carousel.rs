#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const CAROUSEL_KEYS: &[&str] = &[
    "carousel-api",
    "carousel-demo",
    "carousel-orientation",
    "carousel-plugin",
    "carousel-size",
    "carousel-spacing",
];

#[test]
fn shadcn_carousel_goldens_are_targeted_gates() {
    for &key in CAROUSEL_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| {
            n.tag == "div" && class_has_token(n, "overflow-hidden")
        })
        .expect("missing carousel viewport (overflow-hidden)");

        find_first(&theme.root, &|n| {
            n.tag == "div" && class_has_token(n, "min-w-0") && class_has_token(n, "shrink-0")
        })
        .expect("missing carousel item basis (min-w-0 shrink-0)");

        let has_horizontal_controls = find_first(&theme.root, &|n| {
            n.tag == "button"
                && n.class_name
                    .as_deref()
                    .is_some_and(|c| c.contains("-left-12") || c.contains("-right-12"))
        })
        .is_some();

        let has_vertical_controls = find_first(&theme.root, &|n| {
            n.tag == "button"
                && n.class_name
                    .as_deref()
                    .is_some_and(|c| c.contains("-top-12") || c.contains("-bottom-12"))
        })
        .is_some();

        assert!(
            has_horizontal_controls || has_vertical_controls,
            "missing carousel nav controls (expected -left/-right or -top/-bottom buttons) in {key}"
        );
    }
}

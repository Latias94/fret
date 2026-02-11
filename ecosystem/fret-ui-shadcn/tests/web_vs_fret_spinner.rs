#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const SPINNER_KEYS: &[&str] = &[
    "spinner-badge",
    "spinner-basic",
    "spinner-button",
    "spinner-color",
    "spinner-custom",
    "spinner-demo",
    "spinner-empty",
    "spinner-input-group",
    "spinner-item",
    "spinner-size",
];

#[test]
fn shadcn_spinner_goldens_are_targeted_gates() {
    for &key in SPINNER_KEYS {
        let web = read_web_golden(key);
        let theme = web_theme(&web);

        let spinner = find_first(&theme.root, &|n| {
            n.tag == "svg" && class_has_token(n, "animate-spin")
        })
        .expect("missing web spinner svg (animate-spin)");

        assert_eq!(spinner.tag, "svg", "expected <svg> spinner node");
    }
}

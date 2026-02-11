#[path = "web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const TEXTAREA_KEYS: &[&str] = &[
    "textarea-disabled",
    "textarea-with-button",
    "textarea-with-label",
    "textarea-with-text",
];

#[test]
fn shadcn_textarea_goldens_are_targeted_gates() {
    for &key in TEXTAREA_KEYS {
        let web = read_web_golden(key);
        let theme = web_theme(&web);

        find_first(&theme.root, &|n| n.tag == "textarea").expect("missing <textarea> element");
        find_first(&theme.root, &|n| {
            n.tag == "textarea"
                && class_contains(n, "field-sizing-content")
                && class_contains(n, "min-h-16")
        })
        .expect("missing textarea recipe class markers");
    }
}

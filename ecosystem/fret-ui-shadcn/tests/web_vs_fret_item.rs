#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const ITEM_KEYS: &[&str] = &[
    "item-avatar",
    "item-demo",
    "item-group",
    "item-header",
    "item-icon",
    "item-image",
    "item-link",
    "item-size",
    "item-variant",
];

#[test]
fn shadcn_item_goldens_are_targeted_gates() {
    for &key in ITEM_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| {
            (n.tag == "div" || n.tag == "a") && class_has_token(n, "group/item")
        })
        .expect("missing item row node (group/item)");
    }
}

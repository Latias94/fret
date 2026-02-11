#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const NATIVE_SELECT_KEYS: &[&str] = &[
    "native-select-demo",
    "native-select-disabled",
    "native-select-groups",
    "native-select-invalid",
];

#[test]
fn shadcn_native_select_goldens_are_targeted_gates() {
    for &key in NATIVE_SELECT_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| class_has_token(n, "group/native-select"))
            .expect("missing group/native-select wrapper");
        find_first(&theme.root, &|n| {
            n.tag == "select" && class_has_token(n, "appearance-none")
        })
        .expect("missing <select> appearance-none recipe node");
    }
}

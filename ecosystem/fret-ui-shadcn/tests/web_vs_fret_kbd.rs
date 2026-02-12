#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const KBD_KEYS: &[&str] = &["kbd-button", "kbd-group", "kbd-input-group", "kbd-tooltip"];

#[test]
fn shadcn_kbd_goldens_are_targeted_gates() {
    for &key in KBD_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        if key == "kbd-tooltip" {
            find_first(&theme.root, &|n| {
                class_contains(n, "has-[>[data-slot=button-group]]:gap-2")
            })
            .expect("missing tooltip trigger input-group recipe markers");
        } else {
            find_first(&theme.root, &|n| n.tag == "kbd").expect("missing <kbd> element");
            find_first(&theme.root, &|n| {
                n.tag == "kbd" && class_contains(n, "min-w-5")
            })
            .expect("missing <kbd> recipe markers");
        }
    }
}

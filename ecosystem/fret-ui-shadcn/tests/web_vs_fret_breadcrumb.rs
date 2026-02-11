#[path = "web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const BREADCRUMB_KEYS: &[&str] = &[
    "breadcrumb-ellipsis",
    "breadcrumb-link",
    "breadcrumb-separator",
];

#[test]
fn shadcn_breadcrumb_goldens_are_targeted_gates() {
    for &key in BREADCRUMB_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| {
            class_has_token(n, "break-words") && class_has_token(n, "flex-wrap")
        })
        .expect("missing breadcrumb root recipe markers");

        if key == "breadcrumb-separator" {
            find_first(&theme.root, &|n| class_contains(n, "lucide-slash"))
                .expect("missing breadcrumb slash separator icon");
        } else {
            find_first(&theme.root, &|n| class_contains(n, "lucide-chevron-right"))
                .expect("missing breadcrumb chevron separator icon");
        }

        if key == "breadcrumb-ellipsis" {
            find_first(&theme.root, &|n| class_contains(n, "lucide-ellipsis"))
                .expect("missing breadcrumb ellipsis icon");
        }
    }
}

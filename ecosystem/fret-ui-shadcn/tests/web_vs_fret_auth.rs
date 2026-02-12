#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

const LOGIN_KEYS: &[&str] = &["login-01", "login-02", "login-03", "login-04", "login-05"];
const SIGNUP_KEYS: &[&str] = &[
    "signup-01",
    "signup-02",
    "signup-03",
    "signup-04",
    "signup-05",
];
const OTP_KEYS: &[&str] = &["otp-01", "otp-02", "otp-03", "otp-04", "otp-05"];

#[test]
fn shadcn_auth_template_goldens_are_targeted_gates() {
    for &key in LOGIN_KEYS.iter().chain(SIGNUP_KEYS).chain(OTP_KEYS) {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| class_has_token(n, "min-h-svh"))
            .expect("missing min-h-svh layout wrapper");

        if key.starts_with("otp-") {
            let has_slot_list_recipe =
                find_first(&theme.root, &|n| class_contains(n, "input-otp-slot")).is_some();
            let has_slot_node_recipe = find_first(&theme.root, &|n| {
                class_contains(n, "data-[active=true]:border-ring")
                    && class_contains(n, "first:rounded-l-md")
            })
            .is_some();
            assert!(
                has_slot_list_recipe || has_slot_node_recipe,
                "missing input-otp recipe markers"
            );
        } else {
            let has_max_w = find_first(&theme.root, &|n| {
                class_has_token(n, "max-w-sm") || class_has_token(n, "max-w-xs")
            })
            .is_some();
            assert!(has_max_w, "missing max-w-sm/max-w-xs content container");
        }
    }
}

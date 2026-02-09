use super::*;

#[test]
fn web_vs_fret_sonner_open_toast_rect_matches_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sonner_toast_open_cases_v1.json"
    ));
    let suite: FixtureSuite<SonnerToastCase> =
        serde_json::from_str(raw).expect("sonner toast fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("sonner toast open case={}", case.id);
        let web = read_web_golden_open(&case.web_name);
        assert_sonner_toast_rect_matches_web(&case.web_name, &web, move |sonner, host, window| {
            let mut opts = fret_ui_shadcn::ToastMessageOptions::new();
            if let Some(desc) = &case.description {
                opts = opts.description(desc.clone());
            }
            if let (Some(label), Some(cmd)) = (&case.action_label, &case.action_command) {
                opts = opts.action(label.clone(), fret_runtime::CommandId::new(cmd.clone()));
            }

            match case.kind {
                SonnerToastKind::Message => {
                    let _ = sonner.toast_message(host, window, case.title.clone(), opts);
                }
                SonnerToastKind::Success => {
                    let _ = sonner.toast_success_message(host, window, case.title.clone(), opts);
                }
                SonnerToastKind::Info => {
                    let _ = sonner.toast_info_message(host, window, case.title.clone(), opts);
                }
                SonnerToastKind::Warning => {
                    let _ = sonner.toast_warning_message(host, window, case.title.clone(), opts);
                }
                SonnerToastKind::Error => {
                    let _ = sonner.toast_error_message(host, window, case.title.clone(), opts);
                }
                SonnerToastKind::Promise => {
                    let _promise = sonner.toast_promise(host, window, case.title.clone());
                }
            }
        });
    }
}

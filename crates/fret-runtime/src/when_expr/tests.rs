use super::WhenExpr;
use crate::{ExternalDragPayloadKind, ExternalDragPositionQuality, InputContext};

#[test]
fn when_expr_identifier_contract_matches_capability_tables() {
    use crate::capabilities::{KNOWN_BOOL_CAPABILITY_KEYS, KNOWN_STR_CAPABILITY_KEYS};

    for &key in KNOWN_BOOL_CAPABILITY_KEYS {
        WhenExpr::parse(key)
            .unwrap_or_else(|_| panic!("expected bool key to parse as identifier: {key}"))
            .validate()
            .unwrap_or_else(|_| panic!("expected bool key to validate as boolean expr: {key}"));

        let prefixed = format!("cap.{key}");
        WhenExpr::parse(&prefixed)
            .unwrap_or_else(|_| panic!("expected cap-prefixed bool key to parse: {prefixed}"))
            .validate()
            .unwrap_or_else(|_| panic!("expected cap-prefixed bool key to validate: {prefixed}"));
    }

    for &key in KNOWN_STR_CAPABILITY_KEYS {
        let expr = WhenExpr::parse(key)
            .unwrap_or_else(|_| panic!("expected str key to parse as identifier: {key}"));
        assert!(
            expr.validate().is_err(),
            "expected string key to fail boolean validation: {key}"
        );

        let eq = format!("{key} == \"__test__\"");
        WhenExpr::parse(&eq)
            .unwrap_or_else(|_| panic!("expected string comparison to parse: {eq}"))
            .validate()
            .unwrap_or_else(|_| panic!("expected string comparison to validate: {eq}"));

        let prefixed_eq = format!("cap.{key} != \"__test__\"");
        WhenExpr::parse(&prefixed_eq)
            .unwrap_or_else(|_| {
                panic!("expected cap-prefixed string comparison to parse: {prefixed_eq}")
            })
            .validate()
            .unwrap_or_else(|_| {
                panic!("expected cap-prefixed string comparison to validate: {prefixed_eq}")
            });
    }
}

#[test]
fn when_expr_identifier_contract_covers_builtin_identifiers() {
    let bool_idents = [
        "ui.has_modal",
        "focus.is_text_input",
        "edit.can_undo",
        "edit.can_redo",
    ];
    for ident in bool_idents {
        WhenExpr::parse(ident)
            .unwrap_or_else(|_| panic!("expected identifier to parse: {ident}"))
            .validate()
            .unwrap_or_else(|_| panic!("expected identifier to validate as bool: {ident}"));
    }

    let platform_eq = "platform != \"web\"";
    WhenExpr::parse(platform_eq)
        .unwrap_or_else(|_| panic!("expected platform comparison to parse: {platform_eq}"))
        .validate()
        .unwrap_or_else(|_| panic!("expected platform comparison to validate: {platform_eq}"));
}

#[test]
fn when_expr_can_eval_capability_bools() {
    let mut ctx = InputContext::default();
    ctx.caps.ui.multi_window = false;

    assert!(!WhenExpr::parse("ui.multi_window").unwrap().eval(&ctx));
    assert!(
        WhenExpr::parse("cap.ui.multi_window == false")
            .unwrap()
            .eval(&ctx)
    );
}

#[test]
fn when_expr_can_eval_capability_strings() {
    let mut ctx = InputContext::default();
    ctx.caps.dnd.external_payload = ExternalDragPayloadKind::FileToken;
    ctx.caps.dnd.external_position = ExternalDragPositionQuality::BestEffort;

    assert!(
        WhenExpr::parse("dnd.external_payload == \"file_token\"")
            .unwrap()
            .eval(&ctx)
    );
    assert!(
        WhenExpr::parse("cap.dnd.external_payload != \"none\"")
            .unwrap()
            .eval(&ctx)
    );

    assert!(
        WhenExpr::parse("dnd.external_position == \"best_effort\"")
            .unwrap()
            .eval(&ctx)
    );
}

#[test]
fn when_expr_validation_rejects_unknown_identifier() {
    let expr = WhenExpr::parse("ui.multi_windo").unwrap();
    assert!(expr.validate().is_err());
}

#[test]
fn when_expr_validation_rejects_string_key_used_as_bool() {
    let expr = WhenExpr::parse("dnd.external_payload").unwrap();
    assert!(expr.validate().is_err());
}

#[test]
fn when_expr_validation_rejects_type_mismatched_comparison() {
    let expr = WhenExpr::parse("ui.multi_window == \"true\"").unwrap();
    assert!(expr.validate().is_err());
}

#[test]
fn when_expr_validation_accepts_valid_expressions() {
    let expr = WhenExpr::parse("cap.ui.multi_window && platform != \"web\"").unwrap();
    expr.validate().unwrap();
}

use super::WhenExpr;
use crate::{ExternalDragPayloadKind, ExternalDragPositionQuality, InputContext};

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

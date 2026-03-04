use super::WhenEvalContext;
use super::ast::Value;

#[derive(Debug, Clone)]
pub(super) enum Lit {
    Bool(bool),
    Str(String),
}

pub(super) fn eval_value(cx: &WhenEvalContext<'_>, v: &Value) -> Option<Lit> {
    match v {
        Value::Bool(b) => Some(Lit::Bool(*b)),
        Value::Str(s) => Some(Lit::Str(s.clone())),
        Value::Ident(id) => {
            if let Some(b) = eval_ident_bool_opt(cx, id) {
                return Some(Lit::Bool(b));
            }
            if let Some(s) = eval_ident_str_opt(cx, id) {
                return Some(Lit::Str(s.to_string()));
            }
            None
        }
    }
}

pub(super) fn eval_ident_bool(cx: &WhenEvalContext<'_>, name: &str) -> bool {
    eval_ident_bool_opt(cx, name).unwrap_or(false)
}

fn eval_ident_bool_opt(cx: &WhenEvalContext<'_>, name: &str) -> Option<bool> {
    if let Some(query) = name.strip_prefix("keyctx.") {
        return Some(cx.key_contexts.iter().any(|active| {
            let active = active.as_ref();
            active == query
                || (active.starts_with(query) && active.as_bytes().get(query.len()) == Some(&b'.'))
        }));
    }

    match name {
        "ui.has_modal" => Some(cx.input.ui_has_modal),
        "focus.is_text_input" => Some(cx.input.focus_is_text_input),
        "edit.can_undo" => Some(cx.input.edit_can_undo),
        "edit.can_redo" => Some(cx.input.edit_can_redo),
        "router.can_back" => Some(cx.input.router_can_back),
        "router.can_forward" => Some(cx.input.router_can_forward),
        _ => {
            let key = name.strip_prefix("cap.").unwrap_or(name);
            cx.input.caps.bool_key(key)
        }
    }
}

fn eval_ident_str_opt<'a>(cx: &'a WhenEvalContext<'a>, name: &str) -> Option<&'a str> {
    match name {
        "platform" => Some(cx.input.platform.as_str()),
        _ => {
            let key = name.strip_prefix("cap.").unwrap_or(name);
            cx.input.caps.str_key(key)
        }
    }
}

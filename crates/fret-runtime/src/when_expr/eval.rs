use crate::InputContext;

use super::ast::Value;

#[derive(Debug, Clone)]
pub(super) enum Lit {
    Bool(bool),
    Str(String),
}

pub(super) fn eval_value(ctx: &InputContext, v: &Value) -> Option<Lit> {
    match v {
        Value::Bool(b) => Some(Lit::Bool(*b)),
        Value::Str(s) => Some(Lit::Str(s.clone())),
        Value::Ident(id) => {
            if let Some(b) = eval_ident_bool_opt(ctx, id) {
                return Some(Lit::Bool(b));
            }
            if let Some(s) = eval_ident_str_opt(ctx, id) {
                return Some(Lit::Str(s.to_string()));
            }
            None
        }
    }
}

pub(super) fn eval_ident_bool(ctx: &InputContext, name: &str) -> bool {
    eval_ident_bool_opt(ctx, name).unwrap_or(false)
}

fn eval_ident_bool_opt(ctx: &InputContext, name: &str) -> Option<bool> {
    match name {
        "ui.has_modal" => Some(ctx.ui_has_modal),
        "focus.is_text_input" => Some(ctx.focus_is_text_input),
        "edit.can_undo" => Some(ctx.edit_can_undo),
        "edit.can_redo" => Some(ctx.edit_can_redo),
        _ => {
            let key = name.strip_prefix("cap.").unwrap_or(name);
            ctx.caps.bool_key(key)
        }
    }
}

fn eval_ident_str_opt<'a>(ctx: &'a InputContext, name: &str) -> Option<&'a str> {
    match name {
        "platform" => Some(ctx.platform.as_str()),
        _ => {
            let key = name.strip_prefix("cap.").unwrap_or(name);
            ctx.caps.str_key(key)
        }
    }
}

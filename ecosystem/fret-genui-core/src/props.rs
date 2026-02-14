//! Dynamic prop expressions and bindings.
//!
//! This intentionally mirrors json-render-style expression objects:
//! - `{ "$state": "/path" }`
//! - `{ "$item": "field" }` (repeat scopes only)
//! - `{ "$index": true }` (repeat scopes only)
//! - `{ "$bindState": "/path" }` / `{ "$bindItem": "field" }`
//! - `{ "$cond": <VisibilityCondition>, "$then": <expr>, "$else": <expr> }`

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::visibility::{RepeatScope, VisibilityConditionV1, VisibilityContext};

#[derive(Debug, Clone)]
pub struct PropResolutionContext<'a> {
    pub state: &'a Value,
    pub repeat: RepeatScope<'a>,
}

fn is_obj(v: &Value) -> Option<&Map<String, Value>> {
    match v {
        Value::Object(o) => Some(o),
        _ => None,
    }
}

fn get_str<'a>(o: &'a Map<String, Value>, k: &str) -> Option<&'a str> {
    o.get(k).and_then(|v| v.as_str())
}

fn get_bool(o: &Map<String, Value>, k: &str) -> Option<bool> {
    o.get(k).and_then(|v| v.as_bool())
}

fn resolve_item_path<'a>(field: &str, ctx: &PropResolutionContext<'a>) -> Option<Value> {
    let item = ctx.repeat.item?;
    if field.is_empty() {
        return Some(item.clone());
    }
    let ptr = format!("/{}", field);
    crate::json_pointer::get_opt(item, &ptr).cloned()
}

fn resolve_bind_item_path(field: &str, ctx: &PropResolutionContext<'_>) -> Option<String> {
    let base = ctx.repeat.base_path?;
    if field.is_empty() {
        return Some(base.to_string());
    }
    Some(format!("{base}/{field}"))
}

pub fn resolve_value(value: &Value, ctx: &PropResolutionContext<'_>) -> Value {
    let Some(obj) = is_obj(value) else {
        return match value {
            Value::Array(a) => Value::Array(a.iter().map(|v| resolve_value(v, ctx)).collect()),
            Value::Object(o) => Value::Object(
                o.iter()
                    .map(|(k, v)| (k.clone(), resolve_value(v, ctx)))
                    .collect(),
            ),
            other => other.clone(),
        };
    };

    if let Some(path) = get_str(obj, "$state") {
        return crate::json_pointer::get_opt(ctx.state, path)
            .cloned()
            .unwrap_or(Value::Null);
    }

    if let Some(field) = get_str(obj, "$item") {
        return resolve_item_path(field, ctx).unwrap_or(Value::Null);
    }

    if get_bool(obj, "$index") == Some(true) {
        return ctx
            .repeat
            .index
            .map(|i| Value::from(i as i64))
            .unwrap_or(Value::Null);
    }

    if let Some(path) = get_str(obj, "$bindState") {
        return crate::json_pointer::get_opt(ctx.state, path)
            .cloned()
            .unwrap_or(Value::Null);
    }

    if let Some(field) = get_str(obj, "$bindItem") {
        let Some(abs) = resolve_bind_item_path(field, ctx) else {
            return Value::Null;
        };
        return crate::json_pointer::get_opt(ctx.state, &abs)
            .cloned()
            .unwrap_or(Value::Null);
    }

    if obj.contains_key("$cond") && obj.contains_key("$then") && obj.contains_key("$else") {
        let cond_raw = obj.get("$cond").cloned().unwrap_or(Value::Bool(false));
        let cond: Option<VisibilityConditionV1> = serde_json::from_value(cond_raw).ok();
        let vcx = VisibilityContext {
            state: ctx.state,
            repeat: ctx.repeat,
        };
        let pass = cond
            .as_ref()
            .is_some_and(|c| crate::visibility::evaluate(c, &vcx));
        let chosen = if pass { "$then" } else { "$else" };
        return obj
            .get(chosen)
            .map(|v| resolve_value(v, ctx))
            .unwrap_or(Value::Null);
    }

    // Literal object: resolve recursively.
    Value::Object(
        obj.iter()
            .map(|(k, v)| (k.clone(), resolve_value(v, ctx)))
            .collect(),
    )
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedProps {
    pub props: Map<String, Value>,
    pub bindings: BTreeMap<String, String>,
}

pub fn resolve_bindings(
    props: &Map<String, Value>,
    ctx: &PropResolutionContext<'_>,
) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in props.iter() {
        let Some(obj) = is_obj(v) else {
            continue;
        };
        if let Some(path) = get_str(obj, "$bindState") {
            out.insert(k.clone(), path.to_string());
        } else if let Some(field) = get_str(obj, "$bindItem") {
            if let Some(abs) = resolve_bind_item_path(field, ctx) {
                out.insert(k.clone(), abs);
            }
        }
    }
    out
}

pub fn resolve_props(props: &Map<String, Value>, ctx: &PropResolutionContext<'_>) -> ResolvedProps {
    let bindings = resolve_bindings(props, ctx);
    let mut resolved: Map<String, Value> = Map::new();
    for (k, v) in props.iter() {
        resolved.insert(k.clone(), resolve_value(v, ctx));
    }
    ResolvedProps {
        props: resolved,
        bindings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolves_state_ref() {
        let state = json!({"name": "Ada"});
        let ctx = PropResolutionContext {
            state: &state,
            repeat: RepeatScope::default(),
        };
        let v = resolve_value(&json!({"$state": "/name"}), &ctx);
        assert_eq!(v, json!("Ada"));
    }

    #[test]
    fn resolves_conditional() {
        let state = json!({"flag": true});
        let ctx = PropResolutionContext {
            state: &state,
            repeat: RepeatScope::default(),
        };
        let v = resolve_value(
            &json!({
                "$cond": {"$state": "/flag"},
                "$then": "yes",
                "$else": "no"
            }),
            &ctx,
        );
        assert_eq!(v, json!("yes"));
    }

    #[test]
    fn extracts_bindings() {
        let state = json!({"form": {"email": "a@b.com"}});
        let ctx = PropResolutionContext {
            state: &state,
            repeat: RepeatScope::default(),
        };
        let props = serde_json::from_value::<Map<String, Value>>(json!({
            "value": {"$bindState": "/form/email"},
            "label": "Email"
        }))
        .unwrap();
        let bindings = resolve_bindings(&props, &ctx);
        assert_eq!(
            bindings.get("value").map(|s| s.as_str()),
            Some("/form/email")
        );
    }
}

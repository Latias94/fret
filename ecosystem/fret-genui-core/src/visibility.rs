//! Visibility conditions (json-render compatible semantics).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VisibilityConditionV1 {
    Bool(bool),
    Single(SingleConditionV1),
    // Implicit AND list (of single conditions).
    ImplicitAnd(Vec<SingleConditionV1>),
    And {
        #[serde(rename = "$and")]
        and: Vec<VisibilityConditionV1>,
    },
    Or {
        #[serde(rename = "$or")]
        or: Vec<VisibilityConditionV1>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SingleConditionV1 {
    State(StateConditionV1),
    Item(ItemConditionV1),
    Index(IndexConditionV1),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateConditionV1 {
    #[serde(rename = "$state")]
    pub state: String,
    #[serde(flatten)]
    #[serde(default)]
    pub ops: ComparisonOpsV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemConditionV1 {
    #[serde(rename = "$item")]
    pub item: String,
    #[serde(flatten)]
    #[serde(default)]
    pub ops: ComparisonOpsV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IndexConditionV1 {
    #[serde(rename = "$index")]
    pub index: bool,
    #[serde(flatten)]
    #[serde(default)]
    pub ops: ComparisonOpsV1,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ComparisonOpsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eq: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neq: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gt: Option<CompareOperandV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gte: Option<CompareOperandV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lt: Option<CompareOperandV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lte: Option<CompareOperandV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompareOperandV1 {
    Number(f64),
    StateRef {
        #[serde(rename = "$state")]
        state: String,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RepeatScope<'a> {
    pub item: Option<&'a Value>,
    pub index: Option<usize>,
    pub base_path: Option<&'a str>,
}

#[derive(Debug)]
pub struct VisibilityContext<'a> {
    pub state: &'a Value,
    pub repeat: RepeatScope<'a>,
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn resolve_compare_operand<'a>(
    op: &'a CompareOperandV1,
    ctx: &VisibilityContext<'a>,
) -> Option<f64> {
    match op {
        CompareOperandV1::Number(v) => Some(*v),
        CompareOperandV1::StateRef { state } => {
            crate::json_pointer::get_opt(ctx.state, state).and_then(|v| v.as_f64())
        }
    }
}

fn deep_equal(a: &Value, b: &Value) -> bool {
    a == b
}

fn eval_ops<'a>(
    value: Option<&'a Value>,
    ops: &'a ComparisonOpsV1,
    ctx: &VisibilityContext<'a>,
) -> bool {
    let mut out = if let Some(eq) = ops.eq.as_ref() {
        value.is_some_and(|v| deep_equal(v, eq))
    } else if let Some(neq) = ops.neq.as_ref() {
        !value.is_some_and(|v| deep_equal(v, neq))
    } else if let Some(gt) = ops.gt.as_ref() {
        value
            .and_then(|v| v.as_f64())
            .zip(resolve_compare_operand(gt, ctx))
            .is_some_and(|(l, r)| l > r)
    } else if let Some(gte) = ops.gte.as_ref() {
        value
            .and_then(|v| v.as_f64())
            .zip(resolve_compare_operand(gte, ctx))
            .is_some_and(|(l, r)| l >= r)
    } else if let Some(lt) = ops.lt.as_ref() {
        value
            .and_then(|v| v.as_f64())
            .zip(resolve_compare_operand(lt, ctx))
            .is_some_and(|(l, r)| l < r)
    } else if let Some(lte) = ops.lte.as_ref() {
        value
            .and_then(|v| v.as_f64())
            .zip(resolve_compare_operand(lte, ctx))
            .is_some_and(|(l, r)| l <= r)
    } else {
        value.is_some_and(truthy)
    };

    if ops.not == Some(true) {
        out = !out;
    }
    out
}

fn eval_single<'a>(cond: &'a SingleConditionV1, ctx: &VisibilityContext<'a>) -> bool {
    match cond {
        SingleConditionV1::State(c) => {
            let value = crate::json_pointer::get_opt(ctx.state, &c.state);
            eval_ops(value, &c.ops, ctx)
        }
        SingleConditionV1::Item(c) => {
            let Some(item) = ctx.repeat.item else {
                return false;
            };
            let value = if c.item.is_empty() {
                Some(item)
            } else {
                crate::json_pointer::get_opt(item, &format!("/{}", c.item))
            };
            eval_ops(value, &c.ops, ctx)
        }
        SingleConditionV1::Index(c) => {
            if !c.index {
                return false;
            }
            let idx = ctx.repeat.index.map(|i| Value::from(i as i64));
            eval_ops(idx.as_ref(), &c.ops, ctx)
        }
    }
}

pub fn evaluate(condition: &VisibilityConditionV1, ctx: &VisibilityContext<'_>) -> bool {
    match condition {
        VisibilityConditionV1::Bool(v) => *v,
        VisibilityConditionV1::Single(v) => eval_single(v, ctx),
        VisibilityConditionV1::ImplicitAnd(list) => list.iter().all(|c| eval_single(c, ctx)),
        VisibilityConditionV1::And { and } => and.iter().all(|c| evaluate(c, ctx)),
        VisibilityConditionV1::Or { or } => or.iter().any(|c| evaluate(c, ctx)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn state_eq() {
        let state = json!({"tab": "home"});
        let cond: VisibilityConditionV1 = serde_json::from_value(json!({
            "$state": "/tab",
            "eq": "home"
        }))
        .unwrap();
        let ctx = VisibilityContext {
            state: &state,
            repeat: RepeatScope::default(),
        };
        assert!(evaluate(&cond, &ctx));
    }

    #[test]
    fn implicit_and() {
        let state = json!({"a": 1, "b": 2});
        let cond: VisibilityConditionV1 = serde_json::from_value(json!([
            {"$state": "/a", "eq": 1},
            {"$state": "/b", "eq": 2}
        ]))
        .unwrap();
        let ctx = VisibilityContext {
            state: &state,
            repeat: RepeatScope::default(),
        };
        assert!(evaluate(&cond, &ctx));
    }

    #[test]
    fn not_inverts() {
        let state = json!({"x": 0});
        let cond: VisibilityConditionV1 = serde_json::from_value(json!({
            "$state": "/x",
            "not": true
        }))
        .unwrap();
        let ctx = VisibilityContext {
            state: &state,
            repeat: RepeatScope::default(),
        };
        assert!(evaluate(&cond, &ctx));
    }
}

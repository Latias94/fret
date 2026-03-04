mod ast;
mod eval;
mod parser;
mod validate;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::InputContext;

use ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhenExpr(Expr);

/// Evaluation context for `when` expressions.
///
/// This is intentionally a small, data-only view that can be extended without forcing new
/// `InputContext` fields (which would require widespread struct literal churn).
#[derive(Debug, Clone, Copy)]
pub struct WhenEvalContext<'a> {
    pub input: &'a InputContext,
    pub key_contexts: &'a [Arc<str>],
}

impl<'a> WhenEvalContext<'a> {
    pub fn new(input: &'a InputContext) -> Self {
        Self {
            input,
            key_contexts: &[],
        }
    }

    pub fn with_key_contexts(input: &'a InputContext, key_contexts: &'a [Arc<str>]) -> Self {
        Self {
            input,
            key_contexts,
        }
    }
}

impl WhenExpr {
    pub fn parse(input: &str) -> Result<Self, String> {
        Ok(Self(parser::parse(input)?))
    }

    pub fn validate(&self) -> Result<(), WhenExprValidationError> {
        self.0.validate_bool_expr()
    }

    pub fn eval(&self, ctx: &InputContext) -> bool {
        self.eval_in(&WhenEvalContext::new(ctx))
    }

    pub fn eval_with_key_contexts(&self, ctx: &InputContext, key_contexts: &[Arc<str>]) -> bool {
        self.eval_in(&WhenEvalContext::with_key_contexts(ctx, key_contexts))
    }

    pub fn eval_in(&self, cx: &WhenEvalContext<'_>) -> bool {
        self.0.eval(cx)
    }
}

pub use validate::{WhenExprValidationError, WhenValueKind};

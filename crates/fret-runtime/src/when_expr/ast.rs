use crate::InputContext;

use super::eval::{Lit, eval_ident_bool, eval_value};
use super::validate::{WhenExprValidationError, WhenValueKind, ident_kind, value_kind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum Expr {
    Bool(bool),
    Str(String),
    Ident(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Eq(Value, Value, bool /* is_equal */),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum Value {
    Bool(bool),
    Str(String),
    Ident(String),
}

impl Expr {
    pub(super) fn validate_bool_expr(&self) -> Result<(), WhenExprValidationError> {
        match self {
            Expr::Bool(_) => Ok(()),
            Expr::Str(_) => Err(WhenExprValidationError::StrUsedAsBool),
            Expr::Ident(name) => match ident_kind(name)? {
                WhenValueKind::Bool => Ok(()),
                WhenValueKind::Str => {
                    Err(WhenExprValidationError::IdentifierNotBool { name: name.clone() })
                }
            },
            Expr::Not(e) => e.validate_bool_expr(),
            Expr::And(a, b) | Expr::Or(a, b) => {
                a.validate_bool_expr()?;
                b.validate_bool_expr()?;
                Ok(())
            }
            Expr::Eq(a, b, _) => {
                let left = value_kind(a)?;
                let right = value_kind(b)?;
                if left != right {
                    return Err(WhenExprValidationError::ComparisonTypeMismatch { left, right });
                }
                Ok(())
            }
        }
    }

    pub(super) fn eval(&self, ctx: &InputContext) -> bool {
        match self {
            Expr::Bool(v) => *v,
            Expr::Str(_) => false,
            Expr::Ident(name) => eval_ident_bool(ctx, name),
            Expr::Not(e) => !e.eval(ctx),
            Expr::And(a, b) => a.eval(ctx) && b.eval(ctx),
            Expr::Or(a, b) => a.eval(ctx) || b.eval(ctx),
            Expr::Eq(a, b, is_equal) => {
                let left = eval_value(ctx, a);
                let right = eval_value(ctx, b);
                match (left, right) {
                    (Some(Lit::Bool(a)), Some(Lit::Bool(b))) => (a == b) == *is_equal,
                    (Some(Lit::Str(a)), Some(Lit::Str(b))) => (a == b) == *is_equal,
                    _ => false,
                }
            }
        }
    }
}

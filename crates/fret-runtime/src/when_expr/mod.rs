mod ast;
mod eval;
mod parser;
mod validate;

#[cfg(test)]
mod tests;

use crate::InputContext;

use ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhenExpr(Expr);

impl WhenExpr {
    pub fn parse(input: &str) -> Result<Self, String> {
        Ok(Self(parser::parse(input)?))
    }

    pub fn validate(&self) -> Result<(), WhenExprValidationError> {
        self.0.validate_bool_expr()
    }

    pub fn eval(&self, ctx: &InputContext) -> bool {
        self.0.eval(ctx)
    }
}

pub use validate::{WhenExprValidationError, WhenValueKind};

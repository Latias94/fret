use crate::InputContext;
use crate::capabilities::{CapabilityValueKind, capability_key_kind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhenExpr(Expr);

impl WhenExpr {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut p = Parser::new(input);
        let expr = p.parse_expr()?;
        p.skip_ws();
        if !p.eof() {
            return Err(format!("unexpected trailing input at byte {}", p.pos));
        }
        Ok(Self(expr))
    }

    pub fn validate(&self) -> Result<(), WhenExprValidationError> {
        self.0.validate_bool_expr()
    }

    pub fn eval(&self, ctx: &InputContext) -> bool {
        self.0.eval(ctx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenValueKind {
    Bool,
    Str,
}

impl From<CapabilityValueKind> for WhenValueKind {
    fn from(value: CapabilityValueKind) -> Self {
        match value {
            CapabilityValueKind::Bool => Self::Bool,
            CapabilityValueKind::Str => Self::Str,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WhenExprValidationError {
    #[error("unknown identifier: {name}")]
    UnknownIdentifier { name: String },
    #[error("identifier must be boolean in this context: {name}")]
    IdentifierNotBool { name: String },
    #[error("string literal is not a boolean expression")]
    StrUsedAsBool,
    #[error("type mismatch in comparison: left={left:?} right={right:?}")]
    ComparisonTypeMismatch {
        left: WhenValueKind,
        right: WhenValueKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Expr {
    Bool(bool),
    Str(String),
    Ident(String),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Eq(Value, Value, bool /* is_equal */),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Value {
    Bool(bool),
    Str(String),
    Ident(String),
}

impl Expr {
    fn validate_bool_expr(&self) -> Result<(), WhenExprValidationError> {
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

    fn eval(&self, ctx: &InputContext) -> bool {
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

#[derive(Debug, Clone)]
enum Lit {
    Bool(bool),
    Str(String),
}

fn ident_kind(name: &str) -> Result<WhenValueKind, WhenExprValidationError> {
    match name {
        "ui.has_modal" | "focus.is_text_input" => return Ok(WhenValueKind::Bool),
        "edit.can_undo" | "edit.can_redo" => return Ok(WhenValueKind::Bool),
        "router.can_back" | "router.can_forward" => return Ok(WhenValueKind::Bool),
        "platform" => return Ok(WhenValueKind::Str),
        _ => {}
    }

    let key = name.strip_prefix("cap.").unwrap_or(name);
    match capability_key_kind(key) {
        Some(kind) => Ok(kind.into()),
        None => Err(WhenExprValidationError::UnknownIdentifier {
            name: name.to_string(),
        }),
    }
}

fn value_kind(v: &Value) -> Result<WhenValueKind, WhenExprValidationError> {
    match v {
        Value::Bool(_) => Ok(WhenValueKind::Bool),
        Value::Str(_) => Ok(WhenValueKind::Str),
        Value::Ident(id) => ident_kind(id),
    }
}

fn eval_value(ctx: &InputContext, v: &Value) -> Option<Lit> {
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

fn eval_ident_bool(ctx: &InputContext, name: &str) -> bool {
    eval_ident_bool_opt(ctx, name).unwrap_or(false)
}

fn eval_ident_bool_opt(ctx: &InputContext, name: &str) -> Option<bool> {
    match name {
        "ui.has_modal" => Some(ctx.ui_has_modal),
        "focus.is_text_input" => Some(ctx.focus_is_text_input),
        "edit.can_undo" => Some(ctx.edit_can_undo),
        "edit.can_redo" => Some(ctx.edit_can_redo),
        "router.can_back" => Some(ctx.router_can_back),
        "router.can_forward" => Some(ctx.router_can_forward),
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

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn eat(&mut self, s: &str) -> bool {
        if self.input[self.pos..].starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if self.eat("||") {
                let right = self.parse_and()?;
                left = Expr::Or(Box::new(left), Box::new(right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            self.skip_ws();
            if self.eat("&&") {
                let right = self.parse_unary()?;
                left = Expr::And(Box::new(left), Box::new(right));
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat("!") {
            let inner = self.parse_unary()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let left_expr = self.parse_primary()?;
        self.skip_ws();

        if self.eat("==") {
            let right_expr = self.parse_primary()?;
            let left = expr_to_value(left_expr)?;
            let right = expr_to_value(right_expr)?;
            return Ok(Expr::Eq(left, right, true));
        }
        if self.eat("!=") {
            let right_expr = self.parse_primary()?;
            let left = expr_to_value(left_expr)?;
            let right = expr_to_value(right_expr)?;
            return Ok(Expr::Eq(left, right, false));
        }

        Ok(left_expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        self.skip_ws();
        if self.eat("(") {
            let expr = self.parse_expr()?;
            self.skip_ws();
            if !self.eat(")") {
                return Err(format!("expected ')' at byte {}", self.pos));
            }
            return Ok(expr);
        }

        if self.eat("true") {
            return Ok(Expr::Bool(true));
        }
        if self.eat("false") {
            return Ok(Expr::Bool(false));
        }

        if self.peek_char() == Some('"') {
            return self.parse_string();
        }

        self.parse_ident()
    }

    fn parse_string(&mut self) -> Result<Expr, String> {
        if !self.eat("\"") {
            return Err(format!("expected string '\"' at byte {}", self.pos));
        }
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c == '"' {
                let s = self.input[start..self.pos].to_string();
                self.pos += 1;
                return Ok(Expr::Str(s));
            }
            self.pos += c.len_utf8();
        }
        Err("unterminated string".into())
    }

    fn parse_ident(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' || c == '.' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(format!("expected identifier at byte {}", self.pos));
        }
        Ok(Expr::Ident(self.input[start..self.pos].to_string()))
    }
}

fn expr_to_value(expr: Expr) -> Result<Value, String> {
    match expr {
        Expr::Bool(b) => Ok(Value::Bool(b)),
        Expr::Str(s) => Ok(Value::Str(s)),
        Expr::Ident(s) => Ok(Value::Ident(s)),
        _ => Err("expected literal or identifier".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::WhenExpr;
    use crate::ExternalDragPayloadKind;
    use crate::ExternalDragPositionQuality;
    use crate::InputContext;

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
}

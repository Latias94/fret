use super::ast::{Expr, Value};

pub(super) fn parse(input: &str) -> Result<Expr, String> {
    let mut p = Parser::new(input);
    let expr = p.parse_expr()?;
    p.skip_ws();
    if !p.eof() {
        return Err(format!("unexpected trailing input at byte {}", p.pos));
    }
    Ok(expr)
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

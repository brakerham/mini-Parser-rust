use std::{env, fmt, process::ExitCode};

enum Expr {
    Integer(i64),
    Negate(Box<Expr>),
    Binary {
        left: Box<Expr>,
        op: char,
        right: Box<Expr>,
    },
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Negate(expr) => write!(f, "(-{expr})"),
            Self::Binary { left, op, right } => write!(f, "({left} {op} {right})"),
        }
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn parse(input: &str) -> Result<Expr, String> {
        let mut parser = Parser { input, pos: 0 };
        let expr = parser.expression()?;
        if let Some(ch) = parser.peek() {
            return Err(parser.error(&format!("unexpected character {ch:?}")));
        }
        Ok(expr)
    }

    // expression = product (('+' | '-') product)*
    fn expression(&mut self) -> Result<Expr, String> {
        self.binary(Self::product, &['+', '-'])
    }

    // product = unary (('*' | '/') unary)*
    fn product(&mut self) -> Result<Expr, String> {
        self.binary(Self::unary, &['*', '/'])
    }

    fn binary(
        &mut self,
        operand: fn(&mut Self) -> Result<Expr, String>,
        operators: &[char],
    ) -> Result<Expr, String> {
        let mut left = operand(self)?;
        while let Some(op) = self.peek().filter(|ch| operators.contains(ch)) {
            self.pos += 1;
            let right = operand(self)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // unary = '-' unary | '(' expression ')' | integer
    fn unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some('-') => {
                self.pos += 1;
                Ok(Expr::Negate(Box::new(self.unary()?)))
            }
            Some('(') => {
                self.pos += 1;
                let expr = self.expression()?;
                if self.peek() != Some(')') {
                    return Err(self.error("expected ')'"));
                }
                self.pos += 1;
                Ok(expr)
            }
            Some('0'..='9') => {
                let start = self.pos;
                while self
                    .input
                    .as_bytes()
                    .get(self.pos)
                    .is_some_and(u8::is_ascii_digit)
                {
                    self.pos += 1;
                }
                self.input[start..self.pos]
                    .parse()
                    .map(Expr::Integer)
                    .map_err(|_| format!("column {}: integer literal is too large", start + 1))
            }
            _ => Err(self.error("expected an integer, '-' or '('")),
        }
    }

    fn peek(&mut self) -> Option<char> {
        for ch in self.input[self.pos..].chars() {
            if !ch.is_whitespace() {
                return Some(ch);
            }
            self.pos += ch.len_utf8();
        }
        None
    }

    fn error(&self, message: &str) -> String {
        let column = self.input[..self.pos].chars().count() + 1;
        format!("column {column}: {message}")
    }
}

fn main() -> ExitCode {
    let input = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if input.is_empty() {
        eprintln!("Usage: mini-parser \"1 + 2 * (3 - 4)\"");
        return ExitCode::FAILURE;
    }

    match Parser::parse(&input) {
        Ok(tree) => {
            println!("{tree}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

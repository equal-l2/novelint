use crate::parser::Rule;

#[derive(Debug)]
pub enum ExprRuntimeError {
    IdentNotFound(String),
    OverFlow,
}

impl std::fmt::Display for ExprRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::IdentNotFound(s) => write!(f, "Ident \"{}\" was not found", s),
            Self::OverFlow => write!(f, "Expr overflowed"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    IdentOrNum(IdentOrNum),
    TrueExpr(TrueExpr),
}

#[derive(Debug, Clone)]
pub enum ExprOp {
    Add, // +
}

#[derive(Debug, Clone)]
pub enum IdentOrNum {
    Ident(String),
    Num(usize),
}

#[derive(Debug, Clone)]
pub struct TrueExpr {
    lhs: IdentOrNum,
    op: ExprOp,
    rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum CompOp {
    LessThan,     // <
    GreaterThan,  // >
    Equal,        // ==
    NotEqual,     // !=
    LessEqual,    // <=
    GreaterEqual, // >=
}

#[derive(Debug)]
pub struct CompOpParseError(String);

impl std::fmt::Display for CompOpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "cannot parse as CompOp: {}", self.0)
    }
}

impl std::str::FromStr for CompOp {
    type Err = CompOpParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(Self::LessThan),
            ">" => Ok(Self::GreaterThan),
            "==" => Ok(Self::Equal),
            "!=" => Ok(Self::NotEqual),
            "<=" => Ok(Self::LessEqual),
            ">=" => Ok(Self::GreaterEqual),
            _ => Err(CompOpParseError(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompExpr {
    lhs: Expr,
    op: CompOp,
    rhs: Expr,
}

pub trait FromStmt {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self;
}

impl FromStmt for Expr {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self {
        let stmt = stmt.into_inner().next().unwrap();
        match stmt.as_rule() {
            Rule::IdentOrNum => Self::IdentOrNum(IdentOrNum::parse_stmt(stmt)),
            Rule::TrueExpr => Self::TrueExpr(TrueExpr::parse_stmt(stmt)),
            other => {
                panic!("Semantic error: unexpected rule : {:?}", other);
            }
        }
    }
}

impl FromStmt for ExprOp {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self {
        match stmt.as_str() {
            "+" => Self::Add,
            _ => unreachable!(),
        }
    }
}

impl FromStmt for IdentOrNum {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self {
        let it = stmt.into_inner().next().unwrap();
        match it.as_rule() {
            Rule::Ident => Self::Ident(it.as_str().to_owned()),
            Rule::Num => Self::Num(it.as_str().parse().unwrap()),
            other => {
                panic!("Semantic error: unexpected rule : {:?}", other);
            }
        }
    }
}

impl FromStmt for TrueExpr {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self {
        let mut it = stmt.into_inner();
        Self {
            lhs: IdentOrNum::parse_stmt(it.next().unwrap()),
            op: ExprOp::parse_stmt(it.next().unwrap()),
            rhs: Box::new(Expr::parse_stmt(it.next().unwrap())),
        }
    }
}

impl FromStmt for CompOp {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self {
        use std::str::FromStr;
        Self::from_str(stmt.as_str()).unwrap()
    }
}

impl FromStmt for CompExpr {
    fn parse_stmt(stmt: pest::iterators::Pair<Rule>) -> Self {
        let mut it = stmt.into_inner();
        Self {
            lhs: Expr::parse_stmt(it.next().unwrap()),
            op: CompOp::parse_stmt(it.next().unwrap()),
            rhs: Expr::parse_stmt(it.next().unwrap()),
        }
    }
}

pub trait Eval {
    type T;
    fn eval(&self, call_stack: &crate::runner::CallStack) -> Result<Self::T, ExprRuntimeError>;
}

impl Eval for CompExpr {
    type T = bool;
    fn eval(&self, call_stack: &crate::runner::CallStack) -> Result<Self::T, ExprRuntimeError> {
        let lhs = self.lhs.eval(call_stack)?;
        let rhs = self.rhs.eval(call_stack)?;
        Ok(match self.op {
            CompOp::LessThan => lhs < rhs,
            CompOp::GreaterThan => lhs > rhs,
            CompOp::Equal => lhs == rhs,
            CompOp::NotEqual => lhs != rhs,
            CompOp::LessEqual => lhs <= rhs,
            CompOp::GreaterEqual => lhs >= rhs,
        })
    }
}

impl Eval for Expr {
    type T = usize;
    fn eval(&self, call_stack: &crate::runner::CallStack) -> Result<Self::T, ExprRuntimeError> {
        match self {
            Self::IdentOrNum(ion) => ion.eval(call_stack),
            Self::TrueExpr(x) => x.eval(call_stack),
        }
    }
}

impl Eval for IdentOrNum {
    type T = usize;
    fn eval(&self, call_stack: &crate::runner::CallStack) -> Result<Self::T, ExprRuntimeError> {
        Ok(match self {
            Self::Ident(name) => {
                call_stack
                    .get_var(name)
                    .ok_or_else(|| ExprRuntimeError::IdentNotFound(name.clone()))?
                    .value
            }
            Self::Num(num) => *num,
        })
    }
}

impl Eval for TrueExpr {
    type T = usize;
    fn eval<'a>(&self, call_stack: &crate::runner::CallStack) -> Result<Self::T, ExprRuntimeError> {
        let lhs = self.lhs.eval(call_stack)?;
        let rhs = self.rhs.eval(call_stack)?;
        match self.op {
            ExprOp::Add => lhs.checked_add(rhs).ok_or(ExprRuntimeError::OverFlow),
        }
    }
}
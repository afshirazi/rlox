use crate::{
    expr::{Expr, Literal},
    tokens::Token,
};

pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var(Var),
}

impl Stmt {
    pub fn interpret_stmt(self) -> Result<(), String> {
        match self {
            Stmt::Expr(expr) => {
                Self::evaluate(expr)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = Self::evaluate(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Var(var) => {
                Self::evaluate(var.initializer)?;
                Ok(())
            }
        }
    }

    fn evaluate(expr: Expr) -> Result<Literal, String> {
        expr.interpret_ast()
    }
}

pub struct Var {
    token: Token,
    initializer: Expr,
}

impl Var {
    pub fn new(token: Token, initializer: Expr) -> Self {
        Self { token, initializer }
    }
}


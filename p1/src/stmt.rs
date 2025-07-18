use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    expr::{Expr, Literal},
    tokens::Token,
};

pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    Var(Var, Rc<RefCell<Environment>>),
}

impl Stmt {
    pub fn interpret_stmt(self) -> Result<(), String> {
        match self {
            Stmt::Expr(expr) => {
                expr.interpret_ast()?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = expr.interpret_ast()?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Block(stmts) => {
                stmts
                    .into_iter()
                    .try_for_each(|stmt| stmt.interpret_stmt())?;
                Ok(())
            }
            Stmt::Var(var, env) => {
                match var.initializer {
                    Some(expr) => {
                        let val = expr.interpret_ast()?;
                        env.borrow_mut().define(var.token.lexeme, val)
                    }
                    None => env.borrow_mut().define(var.token.lexeme, Literal::Nil),
                };
                Ok(())
            }
        }
    }
}

pub struct Var {
    token: Token,
    initializer: Option<Expr>,
}

impl Var {
    pub fn new(token: Token) -> Self {
        Self {
            token,
            initializer: None,
        }
    }

    pub fn with_init(token: Token, initializer: Expr) -> Self {
        Self {
            token,
            initializer: Some(initializer),
        }
    }
}

use crate::expr::{Expr, Literal};


pub enum Stmt {
    Expr(Expr),
    Print(Expr)
}

impl Stmt {
    pub fn interpret_stmt(self) -> Result<(), String> {
        match self {
            Stmt::Expr(expr) => {
                Self::evaluate(expr)?;
                Ok(())
            },
            Stmt::Print(expr) => {
                let value = Self::evaluate(expr)?;
                println!("{}", value);
                Ok(())
            },
        }
    }
    
    fn evaluate(expr: Expr) -> Result<Literal, String> {
        expr.interpret_ast()
    }
}
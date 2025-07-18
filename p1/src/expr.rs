use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    environment::Environment,
    tokens::{self, Token},
};

pub enum Expr {
    Literal(Literal),
    Unary(Unary),
    Binary(Binary),
    Grouping(Grouping),
    Variable(Token, Rc<RefCell<Environment>>),
    Assign(Assign, Rc<RefCell<Environment>>),
}

impl Expr {
    // pub fn print_ast(&self) -> String {
    //     match self {
    //         Expr::Literal(literal) => match literal {
    //             Literal::Number(n) => n.to_string(),
    //             Literal::String(s) => s.clone(),
    //             Literal::Boolean(b) => b.to_string(),
    //             Literal::Nil => "nil".to_owned(),
    //         },
    //         Expr::Unary(unary) => match unary.op {
    //             UnaryOp::Minus => format!("(- {})", unary.expr.print_ast()),
    //             UnaryOp::Bang => format!("(! {})", unary.expr.print_ast()),
    //         },
    //         Expr::Binary(binary) => match binary.op {
    //             BinaryOp::EqualEqual => format!(
    //                 "(== {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::BangEqual => format!(
    //                 "(!= {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::Less => format!(
    //                 "(< {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::LessEqual => format!(
    //                 "(<= {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::Greater => format!(
    //                 "(> {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::GreaterEqual => format!(
    //                 "(>= {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::Plus => format!(
    //                 "(+ {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::Minus => format!(
    //                 "(- {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::Star => format!(
    //                 "(* {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //             BinaryOp::Slash => format!(
    //                 "(/ {} {})",
    //                 binary.l_expr.print_ast(),
    //                 binary.r_expr.print_ast()
    //             ),
    //         },
    //         Expr::Grouping(grouping) => format!("(group {})", grouping.expr.print_ast()),
    //         Expr::Identifier(token, _) => match token.token_type {
    //             tokens::TokenType::Identifier => match token.literal.as_ref().unwrap() {
    //                 tokens::Literal::Identifier(i) => format!("({})", i),
    //                 _ => "not allowed".to_owned(),
    //             },
    //             _ => "not allowed".to_owned(),
    //         },
    //     }
    // }

    pub fn interpret_ast(self) -> Result<Literal, String> {
        match self {
            Expr::Literal(literal) => Ok(literal),
            Expr::Unary(unary) => {
                match unary.op {
                    UnaryOp::Minus => match unary.expr.interpret_ast()? {
                        Literal::Number(n) => Ok(Literal::Number(-n)),
                        other => Err(format!("Expected a number but got {}", other)),
                    },
                    UnaryOp::Bang => {
                        match unary.expr.interpret_ast()? {
                            // much stricter than the book's implementation
                            Literal::Boolean(b) => Ok(Literal::Boolean(!b)),
                            other => Err(format!("Expected a boolean value but got {}", other)),
                        }
                    }
                }
            }
            Expr::Binary(binary) => {
                let l = binary.l_expr.interpret_ast()?;
                let r = binary.r_expr.interpret_ast()?;
                match (l, r) {
                    (Literal::Number(ln), Literal::Number(rn)) => match binary.op {
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(ln == rn)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(ln != rn)),
                        BinaryOp::Less => Ok(Literal::Boolean(ln < rn)),
                        BinaryOp::LessEqual => Ok(Literal::Boolean(ln <= rn)),
                        BinaryOp::Greater => Ok(Literal::Boolean(ln > rn)),
                        BinaryOp::GreaterEqual => Ok(Literal::Boolean(ln >= rn)),
                        BinaryOp::Plus => Ok(Literal::Number(ln + rn)),
                        BinaryOp::Minus => Ok(Literal::Number(ln - rn)),
                        BinaryOp::Star => Ok(Literal::Number(ln * rn)),
                        BinaryOp::Slash => Ok(Literal::Number(ln / rn)),
                    },
                    (Literal::String(ls), Literal::String(rs)) => match binary.op {
                        BinaryOp::Plus => Ok(Literal::String(ls + &rs)),
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(ls == rs)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(ls != rs)),
                        bad_op => Err(format!("Operation {} not supported for Strings", bad_op)),
                    },
                    (Literal::Boolean(lb), Literal::Boolean(rb)) => match binary.op {
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(lb == rb)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(lb != rb)),
                        bad_op => Err(format!("Operation {} not supported for Booleans", bad_op)),
                    },
                    (Literal::Nil, Literal::Nil) => match binary.op {
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(true)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(false)),
                        bad_op => Err(format!("Operation {} not supported for Booleans", bad_op)),
                    },
                    (mismatch_l, mismatch_r) => match binary.op {
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(false)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(true)),
                        _ => Err(format!(
                            "Mismatched types: left was {} while right was {}",
                            mismatch_l, mismatch_r
                        )),
                    },
                }
            }
            Expr::Grouping(grouping) => Ok(grouping.expr.interpret_ast()?),
            Expr::Variable(token, map) => match token.token_type {
                tokens::TokenType::Identifier => match token.literal.as_ref().unwrap() {
                    tokens::Literal::Identifier(i) => map
                        .borrow()
                        .get(i)
                        .ok_or("Couldn't find the variable".to_owned()),
                    _ => unreachable!("shouldn't ever be a number/string"),
                },
                ttype => Err(format!(
                    "Somehow this Identifier was of type {:?} instead",
                    ttype
                )),
            },
            Expr::Assign(assign, map) => {
                let val = assign.expr.interpret_ast()?;
                map.borrow_mut().assign(assign.name.lexeme, val.clone())?;
                Ok(val)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Number(n) => write!(f, "{}", n),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

pub enum UnaryOp {
    Minus,
    Bang,
}

pub struct Unary {
    op: UnaryOp,
    expr: Box<Expr>,
}

impl Unary {
    pub fn new(op: UnaryOp, expr: Box<Expr>) -> Self {
        Self { op, expr }
    }
}

#[derive(Debug)]
pub enum BinaryOp {
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::EqualEqual => write!(f, "{}", "=="),
            BinaryOp::BangEqual => write!(f, "{}", "!="),
            BinaryOp::Less => write!(f, "{}", "<"),
            BinaryOp::LessEqual => write!(f, "{}", "<="),
            BinaryOp::Greater => write!(f, "{}", ">"),
            BinaryOp::GreaterEqual => write!(f, "{}", ">="),
            BinaryOp::Plus => write!(f, "{}", "+"),
            BinaryOp::Minus => write!(f, "{}", "-"),
            BinaryOp::Star => write!(f, "{}", "*"),
            BinaryOp::Slash => write!(f, "{}", "/"),
        }
    }
}

pub struct Binary {
    l_expr: Box<Expr>,
    op: BinaryOp,
    r_expr: Box<Expr>,
}

impl Binary {
    pub fn new(l_expr: Box<Expr>, op: BinaryOp, r_expr: Box<Expr>) -> Self {
        Self { l_expr, op, r_expr }
    }
}

pub struct Grouping {
    expr: Box<Expr>,
}

impl Grouping {
    pub fn new(expr: Box<Expr>) -> Self {
        Self { expr }
    }
}

pub struct Assign {
    name: Token,
    expr: Box<Expr>,
}

impl Assign {
    pub fn new(name: Token, expr: Box<Expr>) -> Self {
        Self { name, expr }
    }
}

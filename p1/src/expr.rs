pub enum Expr {
    Literal(Literal),
    Unary(Unary),
    Binary(Binary),
    Grouping(Grouping),
}

impl Expr {
    pub fn print_ast(&self) -> String {
        match self {
            Expr::Literal(literal) => match literal {
                Literal::Number(n) => n.to_string(),
                Literal::String(s) => s.clone(),
                Literal::Boolean(b) => b.to_string(),
                Literal::Nil => "nil".to_owned(),
            },
            Expr::Unary(unary) => match unary.op {
                UnaryOp::Minus => format!("(- {})", unary.expr.print_ast()),
                UnaryOp::Bang => format!("(! {})", unary.expr.print_ast()),
            },
            Expr::Binary(binary) => match binary.op {
                BinaryOp::EqualEqual => format!(
                    "(== {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::BangEqual => format!(
                    "(!= {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::Less => format!(
                    "(< {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::LessEqual => format!(
                    "(<= {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::Greater => format!(
                    "(> {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::GreaterEqual => format!(
                    "(>= {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::Plus => format!(
                    "(+ {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::Minus => format!(
                    "(- {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::Star => format!(
                    "(* {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
                BinaryOp::Slash => format!(
                    "(/ {} {})",
                    binary.l_expr.print_ast(),
                    binary.r_expr.print_ast()
                ),
            },
            Expr::Grouping(grouping) => format!("(group {})", grouping.expr.print_ast()),
        }
    }

    pub fn interpret_ast(self) -> Result<Literal, String> {
        match self {
            Expr::Literal(literal) => {
                Ok(literal)
            },
            Expr::Unary(unary) => {
                match unary.op {
                    UnaryOp::Minus => {
                        match unary.expr.interpret_ast()? {
                            Literal::Number(n) => Ok(Literal::Number(-n)),
                            other => Err(format!("Expected a number but got {:?}", other)) 
                        }
                    },
                    UnaryOp::Bang => {
                        match unary.expr.interpret_ast()? { // much stricter than the book's implementation
                            Literal::Boolean(b) => Ok(Literal::Boolean(!b)),
                            other => Err(format!("Expected a boolean value but got {:?}", other)) 
                        }
                    },
                }
            },
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
                        bad_op => Err(format!("Operation {:?} not supported for Strings", bad_op))
                    },
                    (Literal::Boolean(lb), Literal::Boolean(rb)) => match binary.op {
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(lb == rb)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(lb != rb)),
                        bad_op => Err(format!("Operation {:?} not supported for Booleans", bad_op))
                    },
                    (Literal::Nil, Literal::Nil) => match binary.op {
                        BinaryOp::EqualEqual => Ok(Literal::Boolean(true)),
                        BinaryOp::BangEqual => Ok(Literal::Boolean(false)),
                        bad_op => Err(format!("Operation {:?} not supported for Booleans", bad_op))
                    },
                    (mismatch_l, mismatch_r) => {
                        match binary.op {
                            BinaryOp::EqualEqual => Ok(Literal::Boolean(false)),
                            BinaryOp::BangEqual => Ok(Literal::Boolean(true)),
                            _ => Err(format!("Mismatched types: left was {:?} while right was {:?}", mismatch_l, mismatch_r))
                        }
                    }
                }
            },
            Expr::Grouping(grouping) => {
                Ok(grouping.expr.interpret_ast()?)
            },
        }
    }
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
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

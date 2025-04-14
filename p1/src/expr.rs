pub enum Expr {
    Literal(Literal),
    Unary(Unary),
    Binary(Binary),
    Grouping(Grouping),
}

impl Expr {
    fn print_ast(&self) -> String {
        match self {
            Expr::Literal(literal) => match literal {
                Literal::Number(n) => n.to_string(),
                Literal::String(s) => s.clone(),
                Literal::True => "true".to_owned(),
                Literal::False => "false".to_owned(),
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
}

enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}

enum UnaryOp {
    Minus,
    Bang,
}

struct Unary {
    op: UnaryOp,
    expr: Box<Expr>,
}

enum BinaryOp {
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

struct Binary {
    l_expr: Box<Expr>,
    op: BinaryOp,
    r_expr: Box<Expr>,
}

struct Grouping {
    expr: Box<Expr>,
}

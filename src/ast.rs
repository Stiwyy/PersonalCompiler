pub enum Expr {
    Number(i32),
    Float(f64),
    Boolean(bool),
    StringLiteral(String),
    Array(Vec<Box<Expr>>),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Print(Box<Expr>),
    Exit(Box<Expr>),
    Const {
        name: String,
        value: Box<Expr>,
    },
    Variable(String),
    Null,
}

#[derive(PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Equal,    // ==
    NotEqual, // !=
    Lt,       // <
    Gt,       // >
    Lte,      // <=
    Gte,      // >=
}

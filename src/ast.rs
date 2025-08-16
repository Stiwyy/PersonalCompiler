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
    Let {
        name: String,
        value: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Variable(String),
    Null,
    If {
        condition: Box<Expr>,
        then_branch: Vec<Box<Expr>>,
        else_branch: Option<Vec<Box<Expr>>>,
    },
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

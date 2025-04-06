#[derive(Debug)]
pub enum Expr {
    Number(i32),
    StringLiteral(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Print(Box<Expr>), // Represents a print statement
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

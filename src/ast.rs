#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Str(String),
    Ident(String),
    Bool(bool),
    Array(Vec<Expr>),
    BinaryOp(Box<Expr>, String, Box<Expr>),
    // Новое: индексация массива [индекс]
    Index { array: Box<Expr>, index: Box<Expr> },
    // Новое: вызов метода .method()
    MethodCall { receiver: Box<Expr>, method: String },
}

#[derive(Debug)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Print(Expr),
    #[allow(dead_code)]
    Import(String),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
}

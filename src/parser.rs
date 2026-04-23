use crate::ast::{Expr, Stmt};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            }
        }
        stmts
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.peek() {
            Some(Token::Let) => Some(self.parse_let()),
            Some(Token::Print) => Some(self.parse_print()),
            Some(Token::If) => Some(self.parse_if()),
            Some(Token::Import) => {
                self.advance();
                if let Some(Token::Str(path)) = self.advance() {
                    Some(Stmt::Import(path))
                } else {
                    panic!("Ожидалась строка после import");
                }
            }
            Some(Token::Newline) => {
                self.advance();
                None
            }
            _ => {
                self.advance();
                None
            }
        }
    }

    fn parse_let(&mut self) -> Stmt {
        self.advance(); // let
        let name = match self.advance() {
            Some(Token::Ident(n)) => n,
            _ => panic!("Ожидалось имя переменной"),
        };
        self.consume(Token::Assign);
        let value = self.parse_expr();
        Stmt::Let { name, value }
    }

    fn parse_print(&mut self) -> Stmt {
        self.advance(); // print
        Stmt::Print(self.parse_expr())
    }

    fn parse_if(&mut self) -> Stmt {
        self.advance(); // if
        let condition = self.parse_expr();
        self.consume(Token::LBrace);
        let then_branch = self.parse_block();

        let mut else_branch = None;
        if self.peek() == Some(Token::Else) {
            self.advance();
            self.consume(Token::LBrace);
            else_branch = Some(self.parse_block());
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.peek() != Some(Token::RBrace) && !self.is_at_end() {
            if let Some(s) = self.parse_statement() {
                statements.push(s);
            }
        }
        self.consume(Token::RBrace);
        statements
    }

    // --- Выражения и приоритеты ---

    fn parse_expr(&mut self) -> Expr {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_addition();
        while let Some(op) = self.get_binary_op() {
            if op == "==" || op == ">" || op == "<" {
                self.advance();
                let right = self.parse_addition();
                left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_addition(&mut self) -> Expr {
        let mut left = self.parse_primary_with_suffix();
        while let Some(op) = self.get_binary_op() {
            if op == "+" || op == "-" {
                self.advance();
                let right = self.parse_primary_with_suffix();
                left = Expr::BinaryOp(Box::new(left), op, Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    // Поддержка суффиксов: индексация [] и вызов методов .
    fn parse_primary_with_suffix(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        loop {
            match self.peek() {
                Some(Token::LBrack) => {
                    self.advance(); // [
                    let index = self.parse_expr();
                    self.consume(Token::RBrack);
                    expr = Expr::Index {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Some(Token::Dot) => {
                    self.advance(); // .
                    if let Some(Token::Ident(method)) = self.advance() {
                        expr = Expr::MethodCall {
                            receiver: Box::new(expr),
                            method,
                        };
                    } else {
                        panic!("Ожидалось имя метода после '.'");
                    }
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let token = self.advance().expect("Неожиданный конец файла");
        match token {
            Token::Number(n) => Expr::Number(n),
            Token::Str(s) => Expr::Str(s),
            Token::Ident(i) => Expr::Ident(i),
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::LBrack => {
                let mut elements = Vec::new();
                while self.peek() != Some(Token::RBrack) {
                    elements.push(self.parse_expr());
                    if self.peek() == Some(Token::Comma) {
                        self.advance();
                    }
                }
                self.advance(); // ]
                Expr::Array(elements)
            }
            _ => panic!("Неподдерживаемое выражение: {:?}", token),
        }
    }

    // --- Утилиты ---

    fn get_binary_op(&self) -> Option<String> {
        match self.peek() {
            Some(Token::Plus) => Some("+".into()),
            Some(Token::Minus) => Some("-".into()),
            Some(Token::Eq) => Some("==".into()),
            Some(Token::Greater) => Some(">".into()),
            Some(Token::Less) => Some("<".into()),
            _ => None,
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.peek();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn consume(&mut self, expected: Token) {
        if self.peek() == Some(expected.clone()) {
            self.advance();
        } else {
            panic!("Ожидалось {:?}, но найдено {:?}", expected, self.peek());
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

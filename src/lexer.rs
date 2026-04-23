use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\f]+")] // Пропускаем пробелы и табуляцию
pub enum Token {
    // Ключевые слова
    #[token("let")]
    Let,
    #[token("print")]
    Print,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("import")]
    Import,

    // Идентификаторы и литералы
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex("[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Number(i64),

    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    Str(String),

    // Операторы
    #[token("=")]
    Assign,
    #[token("==")]
    Eq,
    #[token(">")]
    Greater,
    #[token("<")]
    Less,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,

    // Разделители и знаки пунктуации
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("[")]
    LBrack,
    #[token("]")]
    RBrack,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // Перенос строки (важен для завершения выражений)
    #[regex(r"\n|(\r\n)")]
    Newline,
}

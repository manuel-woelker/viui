use crate::expression::span::Span;
use logos::Logos;

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Span,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut lexer = TokenKind::lexer(input);
    let mut tokens = Vec::new();
    while let Some(result) = lexer.next() {
        if let Ok(kind) = result {
            tokens.push(Token {
                kind,
                lexeme: lexer.slice(),
                span: lexer.span().into(),
            });
        } else {
            tokens.push(Token {
                kind: TokenKind::Unexpected,
                lexeme: lexer.slice(),
                span: lexer.span().into(),
            })
        }
    }
    tokens.push(Token {
        kind: TokenKind::EOF,
        lexeme: "",
        span: Span::new(input.len(), input.len()),
    });
    tokens
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
#[logos(skip r"[ \t\n\r\f]+")] // Ignore this regex pattern between tokens
pub enum TokenKind {
    #[regex("[a-zA-Z]+")]
    Identifier,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    Number,

    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#)]
    String,

    EOF,

    Unexpected,
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};
    use std::fmt::Write;

    fn test_lex(input: &str, expected: Expect) {
        let tokens = lex(input);
        let mut string = String::new();
        for token in tokens {
            writeln!(
                string,
                "<{:?}> '{}' {}+{}",
                token.kind,
                token.lexeme,
                token.span.start,
                token.span.end - token.span.start,
            )
            .unwrap();
        }
        expected.assert_eq(&string);
    }

    macro_rules! test_lex {
        ($($name:ident, $input:expr, $expected:expr;)+) => {
            $(#[test]
            fn $name() {
                test_lex($input, $expected);
            })+
        };
    }

    test_lex!(
        test_identifier, "foo", expect![[r#"
            <Identifier> 'foo' 0+3
            <EOF> '' 3+0
        "#]];

        test_number, "123", expect![[r#"
            <Number> '123' 0+3
            <EOF> '' 3+0
        "#]];
        test_number_negative, "-123", expect![[r#"
            <Number> '-123' 0+4
            <EOF> '' 4+0
        "#]];
        test_number_decimal, "0.123", expect![[r#"
            <Number> '0.123' 0+5
            <EOF> '' 5+0
        "#]];

        test_string_empty, "\"\"", expect![[r#"
            <String> '""' 0+2
            <EOF> '' 2+0
        "#]];
        test_string_foo, "\"foo\"", expect![[r#"
            <String> '"foo"' 0+5
            <EOF> '' 5+0
        "#]];

        test_unexpected, "=", expect![[r#"
            <Unexpected> '=' 0+1
            <EOF> '' 1+0
        "#]];

        test_multi, "-123 foo 3.141 \"bar\"", expect![[r#"
            <Number> '-123' 0+4
            <Identifier> 'foo' 5+3
            <Number> '3.141' 9+5
            <String> '"bar"' 15+5
            <EOF> '' 20+0
        "#]];

        test_whitespace, "a\t\n\r b", expect![[r#"
            <Identifier> 'a' 0+1
            <Identifier> 'b' 5+1
            <EOF> '' 6+0
        "#]];
    );
}

use crate::bail;
use crate::expression::span::Span;
use crate::result::ViuiResult;
use logos::Logos;
use unscanny::Scanner;

#[derive(Debug)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Span,
}

pub fn lex(input: &str) -> ViuiResult<Vec<Token>> {
    let mut lexer = Lexer::new(input);
    lexer.lex()?;
    Ok(lexer.tokens)
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LexerState {
    Code,
    TemplateLiteral,
}

pub struct Lexer<'a> {
    state_stack: Vec<LexerState>,
    current_state: LexerState,
    scanner: Scanner<'a>,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            scanner: Scanner::new(input),
            tokens: Vec::new(),
            state_stack: Vec::new(),
            current_state: LexerState::Code,
        }
    }
    fn lex(&mut self) -> ViuiResult<()> {
        while !self.scanner.done() {
            match self.current_state {
                LexerState::Code => {
                    self.lex_code()?;
                }
                LexerState::TemplateLiteral => {
                    self.lex_template_literal()?;
                }
            }
        }
        self.create_token(self.scanner.cursor(), TokenKind::EOF);
        Ok(())
    }

    fn lex_code(&mut self) -> ViuiResult<()> {
        self.scanner.eat_whitespace();
        let start = self.scanner.cursor();
        let Some(char) = self.scanner.eat() else {
            return Ok(());
        };
        match char {
            '}' => {
                self.pop_state();
                self.create_token(start, TokenKind::CloseBrace);
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.scanner.eat_while(char::is_alphanumeric);
                self.create_token(start, TokenKind::Identifier);
            }
            '0'..='9' | '-' => {
                self.scanner
                    .eat_while(|c: char| c.is_ascii_digit() || c == '.');
                self.create_token(start, TokenKind::Number);
            }
            '"' => {
                self.scanner.eat_until('"');
                self.create_token(start + 1, TokenKind::String);
                self.scanner.eat();
            }
            '`' => {
                self.push_state(LexerState::TemplateLiteral);
            }
            _ => {
                self.create_token(start, TokenKind::Unexpected);
                self.create_token(self.scanner.cursor(), TokenKind::EOF);
            }
        }
        Ok(())
    }

    fn push_state(&mut self, state: LexerState) {
        self.state_stack.push(self.current_state);
        self.current_state = state;
    }

    fn pop_state(&mut self) {
        self.current_state = self.state_stack.pop().unwrap();
    }

    fn lex_template_literal(&mut self) -> ViuiResult<()> {
        let start = self.scanner.cursor();
        while !self.scanner.done() {
            let Some(char) = self.scanner.eat() else {
                bail!("Unexpected end of input");
            };
            match char {
                '`' => {
                    self.pop_state();
                    self.scanner.uneat();
                    self.create_token(start, TokenKind::TemplateString);
                    self.scanner.eat();
                    return Ok(());
                }
                '$' => {
                    if self.scanner.eat_if('{') {
                        self.push_state(LexerState::Code);
                        self.scanner.uneat();
                        self.scanner.uneat();
                        self.create_token(start, TokenKind::TemplateString);
                        let start = self.scanner.cursor();
                        self.scanner.eat();
                        self.scanner.eat();
                        self.create_token(start, TokenKind::StartTemplateLiteralExpression);
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn create_token(&mut self, start: usize, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            lexeme: &self.scanner.from(start),
            span: Span::new(start, self.scanner.cursor()),
        });
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenKind {
    Identifier,
    Number,
    String,
    TemplateString,
    EOF,
    Unexpected,
    StartTemplateLiteralExpression,
    CloseBrace,
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};
    use std::fmt::Write;

    fn test_lex(input: &str, expected: Expect) {
        let tokens = lex(input).unwrap();
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
            <String> '' 1+0
            <EOF> '' 2+0
        "#]];
        test_string_foo, "\"foo\"", expect![[r#"
            <String> 'foo' 1+3
            <EOF> '' 5+0
        "#]];

        test_unexpected, "=", expect![[r#"
            <Unexpected> '=' 0+1
            <EOF> '' 1+0
            <EOF> '' 1+0
        "#]];

        test_multi, "-123 foo 3.141 \"bar\"", expect![[r#"
            <Number> '-123' 0+4
            <Identifier> 'foo' 5+3
            <Number> '3.141' 9+5
            <String> 'bar' 16+3
            <EOF> '' 20+0
        "#]];

        test_template_literal_empty, "``", expect![[r#"
            <TemplateString> '' 1+0
            <EOF> '' 2+0
        "#]];
        test_template_literal_non_empty, "`foo`", expect![[r#"
            <TemplateString> 'foo' 1+3
            <EOF> '' 5+0
        "#]];
        test_template_literal_placeholder, "`${foo}`", expect![[r#"
            <TemplateString> '' 1+0
            <StartTemplateLiteralExpression> '${' 1+2
            <Identifier> 'foo' 3+3
            <CloseBrace> '}' 6+1
            <TemplateString> '' 7+0
            <EOF> '' 8+0
        "#]];
        test_template_literal_identifier, "`foo${x}bar`", expect![[r#"
            <TemplateString> 'foo' 1+3
            <StartTemplateLiteralExpression> '${' 4+2
            <Identifier> 'x' 6+1
            <CloseBrace> '}' 7+1
            <TemplateString> 'bar' 8+3
            <EOF> '' 12+0
        "#]];
        test_template_literal_number, "`foo${1.0}bar`", expect![[r#"
            <TemplateString> 'foo' 1+3
            <StartTemplateLiteralExpression> '${' 4+2
            <Number> '1.0' 6+3
            <CloseBrace> '}' 9+1
            <TemplateString> 'bar' 10+3
            <EOF> '' 14+0
        "#]];
        test_template_literal_string, "`foo${\"baz\"}bar`", expect![[r#"
            <TemplateString> 'foo' 1+3
            <StartTemplateLiteralExpression> '${' 4+2
            <String> 'baz' 7+3
            <CloseBrace> '}' 11+1
            <TemplateString> 'bar' 12+3
            <EOF> '' 16+0
        "#]];

        test_template_literal_nested, "`foo${`${fizz}`}bar`", expect![[r#"
            <TemplateString> 'foo' 1+3
            <StartTemplateLiteralExpression> '${' 4+2
            <TemplateString> '' 7+0
            <StartTemplateLiteralExpression> '${' 7+2
            <Identifier> 'fizz' 9+4
            <CloseBrace> '}' 13+1
            <TemplateString> '' 14+0
            <CloseBrace> '}' 15+1
            <TemplateString> 'bar' 16+3
            <EOF> '' 20+0
        "#]];

        test_whitespace, "a\t\n\r b", expect![[r#"
            <Identifier> 'a' 0+1
            <Identifier> 'b' 5+1
            <EOF> '' 6+0
        "#]];
    );
}

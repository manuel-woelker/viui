use crate::bail;
use crate::component::ast::{ExpressionAst, ExpressionKind};
use crate::component::lexer::{lex, Token, TokenKind};
use crate::component::span::Span;
use crate::component::value::ExpressionValue;
use crate::result::ViuiResult;

pub fn parse_expression(expression_string: &str) -> ViuiResult<ExpressionAst> {
    let tokens = lex(expression_string)?;
    let mut parser = Parser::new(&tokens[..]);
    let ast = parser.parse_expression()?;
    Ok(ast)
}

pub struct Parser<'a> {
    current_index: usize,
    tokens: &'a [Token<'a>],
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            current_index: 0,
            tokens,
        }
    }

    fn parse_expression(&mut self) -> ViuiResult<ExpressionAst> {
        self.parse_primary()
    }
    fn parse_primary(&mut self) -> ViuiResult<ExpressionAst> {
        let kind = match self.current_token().kind {
            TokenKind::Number => ExpressionKind::Literal(ExpressionValue::Float(
                self.current_token().lexeme.parse()?,
            )),
            TokenKind::String => ExpressionKind::Literal(ExpressionValue::String(
                self.current_token().lexeme.to_string(),
            )),
            TokenKind::Identifier => {
                ExpressionKind::VarUse(self.current_token().lexeme.to_string())
            }
            TokenKind::TemplateString => {
                return self.parse_template_literal();
            }
            //TokenKind::String => {}
            _ => bail!("Unexpected token: {:?}", self.current_token()),
        };
        let ast = ExpressionAst::new(self.current_token().span, kind);
        self.advance_token();
        Ok(ast)
    }

    fn parse_template_literal(&mut self) -> ViuiResult<ExpressionAst> {
        let mut strings = Vec::new();
        let mut expressions = Vec::new();
        let start_span = self.current_token().span;
        strings.push(self.current_token().lexeme.to_string());
        self.consume(TokenKind::TemplateString, "Expected Template String")?;

        while self.current_token().kind == TokenKind::StartTemplateLiteralExpression {
            self.advance_token();
            let expression = self.parse_expression()?;
            expressions.push(expression);
            self.consume(
                TokenKind::CloseBrace,
                "Expected '}' after expression in template string",
            )?;
            strings.push(self.current_token().lexeme.to_string());
            self.consume(TokenKind::TemplateString, "Expected Template String")?;
        }
        let end_span = self.current_token().span;
        let span = Span::new(start_span.start, end_span.end);
        Ok(ExpressionAst::new(
            span,
            ExpressionKind::StringTemplate {
                strings,
                expressions,
            },
        ))
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> ViuiResult<()> {
        if self.current_token().kind == kind {
            self.advance_token();
            Ok(())
        } else {
            bail!(
                "Found {:?} {}, but {}",
                self.current_token().kind,
                self.current_token().lexeme,
                message
            )
        }
    }

    fn current_token(&self) -> &Token<'a> {
        &self.tokens[self.current_index]
    }

    fn advance_token(&mut self) {
        self.current_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::component::ast::print_expression_ast;
    use assertables::assert_contains;
    use expect_test::{expect, Expect};

    fn test_parse(input: &str, expected_output: Expect) {
        let result = super::parse_expression(input).unwrap();
        let output = print_expression_ast(&result);
        expected_output.assert_eq(&output);
    }

    macro_rules! test_parse {
        ($($name:ident, $input:expr, $expected:expr;)+) => {
            $(#[test]
            fn $name() {
                test_parse($input, $expected);
            })+
        };
    }

    test_parse!(
        parse_number, "123.456",
            expect![[r#"
            Literal Float(123.456)
        "#]];

        parse_string, "\"foo\"",
            expect![[r#"
                Literal String("foo")
            "#]];

        parse_string_template, "`foo`",
            expect![[r#"
                StringTemplate
                └── String foo
            "#]];
        parse_string_template_placeholder, "`a${ foo }b`",
            expect![[r#"
                StringTemplate
                ├── String a
                ├── VarUse foo
                └── String b
            "#]];
        parse_string_template_placeholder_number, "`a${1.0}b`",
            expect![[r#"
                StringTemplate
                ├── String a
                ├── Literal Float(1.0)
                └── String b
            "#]];
        parse_string_template_placeholder_nested, "`a${`x${foo}y`}b`",
            expect![[r#"
                StringTemplate
                ├── String a
                ├── StringTemplate
                |   ├── String x
                |   ├── VarUse foo
                |   └── String y
                └── String b
            "#]];
    );

    #[test]
    fn test_parse_empty() {
        let error = super::parse_expression("").unwrap_err();
        assert_contains!(error.to_string(), "Unexpected token");
    }

    #[test]
    fn test_parse_invalid_symbols() {
        let error = super::parse_expression("#?").unwrap_err();
        assert_contains!(error.to_string(), "Unexpected token");
    }
}

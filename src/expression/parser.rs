use crate::bail;
use crate::expression::ast::{ExpressionAst, ExpressionKind};
use crate::expression::lexer::{lex, Token, TokenKind};
use crate::expression::value::ExpressionValue;
use crate::result::ViuiResult;

pub fn parse_expression(expression_string: &str) -> ViuiResult<ExpressionAst> {
    let tokens = lex(expression_string);
    let mut parser = Parser::new(expression_string, &tokens[..]);
    let ast = parser.parse_expression()?;
    Ok(ast)
}

pub struct Parser<'a> {
    expression_string: &'a str,
    current_index: usize,
    tokens: &'a [Token<'a>],
}

impl<'a> Parser<'a> {
    fn new(expression_string: &'a str, tokens: &'a [Token<'a>]) -> Self {
        Self {
            expression_string,
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
            //TokenKind::String => {}
            _ => bail!("Unexpected token: {:?}", self.current_token()),
        };
        let ast = ExpressionAst::new(self.current_token().span, kind);
        self.advance_token();
        Ok(ast)
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
    use crate::expression::ast::print_expression_ast;
    use expect_test::{expect, Expect};

    fn test_parse(input: &str, expected_output: Expect) {
        let result = super::parse_expression(input).unwrap();
        let output = print_expression_ast(&result);
        expected_output.assert_eq(&output);
    }

    #[test]
    fn test_parse_number() {
        test_parse("123.456", expect![[r#"
            Literal Float(123.456)
        "#]]);
    }

    #[test]
    fn test_parse_empty() {
        let result = super::parse_expression("").unwrap();
        dbg!(result);
    }

    #[test]
    fn test_parse_invalid_symbols() {
        let result = super::parse_expression("#?").unwrap();
        dbg!(result);
    }
}

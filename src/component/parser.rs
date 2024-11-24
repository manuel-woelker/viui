use crate::bail;
use crate::component::ast::{
    ComponentAst, ComponentDefinition, ExpressionAst, ExpressionKind, IfItemDefinition, ItemAst,
    ItemDefinition, NodeAst, NodeDefinition, PropAst, PropDefinition, UIAst, UIDefinition,
};
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

pub fn parse_ui(ui_string: &str) -> ViuiResult<UIAst> {
    let tokens = lex(ui_string)?;
    let mut parser = Parser::new(&tokens[..]);
    parser.parse_ui()
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
    fn parse_ui(&mut self) -> ViuiResult<UIAst> {
        let mut components = vec![];
        while !self.at_end() {
            components.push(self.parse_component()?);
        }
        Ok(UIAst::new(
            Span::new(0, self.previous_token().span.end),
            UIDefinition { components },
        ))
    }

    fn parse_component(&mut self) -> ViuiResult<ComponentAst> {
        let start = self
            .consume(TokenKind::Component, "Expected component")?
            .span
            .start;
        let component_name = self
            .consume(TokenKind::Identifier, "Expected component name")?
            .lexeme
            .to_string();
        self.consume(TokenKind::OpenBrace, "Expected '{'")?;
        let mut children = vec![];
        while !self.is_at(TokenKind::CloseBrace) {
            children.push(self.parse_item()?);
        }
        self.consume(TokenKind::CloseBrace, "Expected '}'")?;
        Ok(ComponentAst::new(
            Span::new(start, self.previous_token().span.end),
            ComponentDefinition {
                name: component_name,
                children,
            },
        ))
    }

    fn parse_item(&mut self) -> ViuiResult<ItemAst> {
        let start = self.current_token().span.start;
        let item = match self.current_token().kind {
            TokenKind::Identifier => ItemDefinition::Node {
                node: self.parse_node()?,
            },
            TokenKind::If => ItemDefinition::If(Box::new(self.parse_if()?)),
            _ => {
                bail!(
                    "Found {:?} {}, but expected node, if or for",
                    self.current_token().kind,
                    self.current_token().lexeme,
                )
            }
        };
        Ok(ItemAst::new(
            Span::new(start, self.previous_token().span.end),
            item,
        ))
    }

    fn parse_if(&mut self) -> ViuiResult<IfItemDefinition> {
        self.consume(TokenKind::If, "Expected 'if'")?;
        self.consume(TokenKind::OpenParen, "Expected '('")?;
        let condition = self.parse_expression()?;
        self.consume(TokenKind::CloseParen, "Expected ')'")?;
        self.consume(TokenKind::OpenBrace, "Expected '{'")?;
        let start = self.current_token().span.start;
        let mut then_items = vec![];
        while !self.is_at(TokenKind::CloseBrace) {
            then_items.push(self.parse_item()?);
        }
        let then_end = self.previous_token().span.end;
        self.consume(TokenKind::CloseBrace, "Expected '}'")?;
        Ok(IfItemDefinition {
            condition,
            then_item: ItemAst::new(
                Span::new(start, then_end),
                ItemDefinition::Block { items: then_items },
            ),
            else_item: None,
        })
    }

    fn parse_node(&mut self) -> ViuiResult<NodeAst> {
        let start = self.current_token().span.start;
        let tag = self
            .consume(TokenKind::Identifier, "Expected ídentifier")?
            .lexeme
            .to_string();
        let mut props = vec![];
        let mut events = vec![];
        let mut children = vec![];
        if self.is_at(TokenKind::OpenParen) {
            self.advance_token();
            while !self.is_at(TokenKind::CloseParen) {
                if self.is_at(TokenKind::At) {
                    self.advance_token();
                    events.push(self.parse_node_prop()?);
                } else {
                    props.push(self.parse_node_prop()?);
                }
            }
            self.consume(TokenKind::CloseParen, "Expected ')'")?;
        }
        if self.is_at(TokenKind::OpenBrace) {
            self.advance_token();
            while !self.is_at(TokenKind::CloseBrace) {
                children.push(self.parse_item()?);
            }
            self.consume(TokenKind::CloseBrace, "Expected '}'")?;
        }
        Ok(NodeAst::new(
            Span::new(start, self.previous_token().span.end),
            NodeDefinition {
                tag,
                props,
                events,
                children,
            },
        ))
    }

    fn parse_node_prop(&mut self) -> ViuiResult<PropAst> {
        let start = self.current_token().span.start;
        let name = self
            .consume(TokenKind::Identifier, "Expected identifier")?
            .lexeme
            .to_string();
        self.consume(TokenKind::Equal, "Expected '='")?;
        let expression = self.parse_expression()?;
        Ok(PropAst::new(
            Span::new(start, self.previous_token().span.end),
            PropDefinition { name, expression },
        ))
    }

    fn parse_expression(&mut self) -> ViuiResult<ExpressionAst> {
        self.parse_call()
    }

    fn parse_call(&mut self) -> ViuiResult<ExpressionAst> {
        let start_span = self.current_token().span.start;
        let mut expr = self.parse_primary()?;
        loop {
            if self.is_at(TokenKind::OpenParen) {
                self.advance_token();
                expr = self.finish_call(expr, start_span)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: ExpressionAst, start: usize) -> ViuiResult<ExpressionAst> {
        let arguments = self.parse_separated(|parser| {
            if parser.is_at(TokenKind::CloseParen) {
                return Ok(None);
            }
            Ok(Some(parser.parse_expression()?))
        })?;
        if arguments.len() >= 255 {
            bail!("Too many arguments in call expression");
        }
        self.consume(
            TokenKind::CloseParen,
            "Expected ')' after function call arguments",
        )?;
        Ok(ExpressionAst::new(
            Span::new(start, self.previous_token().span.end),
            ExpressionKind::Call {
                callee: Box::new(callee),
                arguments,
            },
        ))
    }

    fn parse_separated<T>(
        &mut self,
        parse_fn: impl Fn(&mut Parser) -> ViuiResult<Option<T>>,
    ) -> ViuiResult<Vec<T>> {
        let mut result: Vec<T> = Vec::new();
        loop {
            if let Some(item) = parse_fn(self)? {
                result.push(item);
            } else {
                break;
            }
            if !self.is_at(TokenKind::Comma) {
                break;
            }
            self.advance_token();
        }
        Ok(result)
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

    fn consume(&mut self, kind: TokenKind, message: &str) -> ViuiResult<&Token> {
        if self.current_token().kind == kind {
            self.advance_token();
            Ok(self.previous_token())
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
    fn previous_token(&self) -> &Token<'a> {
        if self.current_index == 0 {
            return self.current_token();
        }
        &self.tokens[self.current_index - 1]
    }

    fn advance_token(&mut self) {
        self.current_index += 1;
    }

    fn at_end(&self) -> bool {
        self.is_at(TokenKind::EOF)
    }

    fn is_at(&self, token_kind: TokenKind) -> bool {
        self.current_token().kind == token_kind
    }
}

#[cfg(test)]
mod tests {
    use crate::component::ast::{print_expression_ast, print_ui_ast};
    use assertables::assert_contains;
    use expect_test::{expect, Expect};

    fn test_parse_expression(input: &str, expected_output: Expect) {
        let result = super::parse_expression(input).unwrap();
        let output = print_expression_ast(&result);
        expected_output.assert_eq(&output);
    }

    macro_rules! test_parse_expression {
        ($($name:ident, $input:expr, $expected:expr;)+) => {
            $(#[test]
            fn $name() {
                test_parse_expression($input, $expected);
            })+
        };
    }

    test_parse_expression!(
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
                │   ├── String x
                │   ├── VarUse foo
                │   └── String y
                └── String b
            "#]];
        parse_call_empty, "foo()",
            expect![[r#"
                Call
                └── VarUse foo
            "#]];
        parse_call_args, "foo(\"foo\", 3)",
            expect![[r#"
                Call
                ├── VarUse foo
                ├── Literal String("foo")
                └── Literal Float(3.0)
            "#]];
    );

    fn test_parse_ui(input: &str, expected_output: Expect) {
        let result = super::parse_ui(input).unwrap();
        let output = print_ui_ast(&result);
        expected_output.assert_eq(&output);
    }

    macro_rules! test_parse_ui {
        ($($name:ident, $input:expr, $expected:expr;)+) => {
            $(#[test]
            fn $name() {
                test_parse_ui($input, $expected);
            })+
        };
    }

    test_parse_ui!(
        parse_empty, "",
            expect![[r#"
                UIDefinition
            "#]];
        parse_component_empty, "component empty {}",
            expect![[r#"
                UIDefinition
                └── Component empty
            "#]];
        parse_component_simple, "component simple {label button}",
            expect![[r#"
                UIDefinition
                └── Component simple
                    ├── Node label
                    └── Node button
            "#]];
        parse_component_with_props, "component simple {label(text=\"foo\")}",
            expect![[r#"
                UIDefinition
                └── Component simple
                    └── Node label
                        └── text=
                            └── Literal String("foo")
            "#]];
        parse_component_with_events, "component event {button(label=\"+1\" @click=Increment)}",
            expect![[r#"
                UIDefinition
                └── Component event
                    └── Node button
                        ├── label=
                        │   └── Literal String("+1")
                        └── @click=
                            └── VarUse Increment
            "#]];

        parse_component_with_children, "component event {button{label}}",
            expect![[r#"
                UIDefinition
                └── Component event
                    └── Node button
                        └── child: Node label
            "#]];

        parse_if, "component simple {if(true) {label}}",
            expect![[r#"
                UIDefinition
                └── Component simple
                    └── if VarUse true
                        └── then
                            └── Node label
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

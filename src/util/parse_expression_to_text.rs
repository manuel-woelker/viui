use crate::bail;
use crate::model::{Text, TextPart};
use crate::result::ViuiResult;
use regex_lite::Regex;

pub fn parse_expression_to_text(original_expression: &str) -> ViuiResult<Text> {
    let mut parts = vec![];
    let string_regex = Regex::new(r#"^([^$]+)"#)?;
    let placeholder_regex = Regex::new(r#"^\$\{([^}]+)}"#)?;
    let mut matched = true;
    let mut expression = original_expression;
    while !expression.is_empty() {
        if !matched {
            bail!(
                "Failed to parse placeholder expression: '{}' at '{}'",
                original_expression,
                expression
            );
        }
        matched = false;
        if let Some(found) = string_regex.find(expression) {
            parts.push(TextPart::FixedText(found.as_str().to_string()));
            expression = &expression[found.end()..];
            matched = true;
        }
        if let Some(found) = placeholder_regex.find(expression) {
            parts.push(TextPart::VariableText(
                expression[found.start() + 2..found.end() - 1].to_string(),
            ));
            expression = &expression[found.end()..];
            matched = true;
        }
    }
    Ok(Text { parts })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expression_to_text() -> ViuiResult<()> {
        let cases = vec![
            (
                "Hello, ${name}!",
                vec![
                    TextPart::FixedText("Hello, ".to_string()),
                    TextPart::VariableText("name".to_string()),
                    TextPart::FixedText("!".to_string()),
                ],
            ),
            (
                "${greeting}, ${name}!",
                vec![
                    TextPart::VariableText("greeting".to_string()),
                    TextPart::FixedText(", ".to_string()),
                    TextPart::VariableText("name".to_string()),
                    TextPart::FixedText("!".to_string()),
                ],
            ),
            (
                "No variables here",
                vec![TextPart::FixedText("No variables here".to_string())],
            ),
            (
                "${var1}${var2}",
                vec![
                    TextPart::VariableText("var1".to_string()),
                    TextPart::VariableText("var2".to_string()),
                ],
            ),
        ];

        for (input, expected) in cases {
            let result = parse_expression_to_text(input)?;
            assert_eq!(result.parts, expected);
        }

        Ok(())
    }

    #[test]
    fn test_parse_expression_to_text_error() {
        let result = parse_expression_to_text("Invalid ${placeholder");
        assert!(result.is_err());
    }
}

use crate::bail;
use crate::component::ast::{ExpressionAst, ExpressionKind};
use crate::component::value::ExpressionValue;
use crate::result::ViuiResult;

type VarLookUp<'a> = dyn Fn(&str) -> ViuiResult<ExpressionValue> + 'a;

pub fn eval(expr: &ExpressionAst, var_lookup: &VarLookUp) -> ViuiResult<ExpressionValue> {
    let evaluator = Evaluator { var_lookup };
    evaluator.eval(expr)
}

struct Evaluator<'a> {
    var_lookup: &'a VarLookUp<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn eval(&self, expression: &ExpressionAst) -> ViuiResult<ExpressionValue> {
        match expression.data() {
            ExpressionKind::Literal(value) => Ok(value.clone()),
            ExpressionKind::VarUse(name) => (self.var_lookup)(name),
            ExpressionKind::StringTemplate {
                strings,
                expressions,
            } => {
                let mut result = String::new();
                for (string, expression) in strings.iter().zip(expressions.iter()) {
                    result.push_str(string);
                    result.push_str(&eval(expression, self.var_lookup)?.to_string().to_string());
                }
                result.push_str(strings.iter().last().unwrap());
                Ok(ExpressionValue::String(result))
            }
            ExpressionKind::Call { callee, arguments } => {
                let mut callee = self.eval(callee)?;
                let ExpressionValue::Function(function) = callee else {
                    bail!("Not a function: {}", callee);
                };
                let arguments = arguments
                    .iter()
                    .map(|arg| self.eval(arg))
                    .collect::<ViuiResult<Vec<ExpressionValue>>>()?;
                function.invoke(arguments)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bail;
    use crate::component::parser::parse_expression;
    #[test]
    fn test_eval() {
        fn var_lookup(name: &str) -> ViuiResult<ExpressionValue> {
            match name {
                "a" => Ok(ExpressionValue::Float(1.234)),
                "b" => Ok(ExpressionValue::String("Foo".to_string())),
                _ => bail!("Unknown variable: {}", name),
            }
        }
        let ast = parse_expression("a").unwrap();
        let result = eval(&ast, &var_lookup).unwrap();
        assert_eq!(result, ExpressionValue::Float(1.234));
    }

    #[test]
    fn test_eval2() {
        fn var_lookup(name: &str) -> ViuiResult<ExpressionValue> {
            match name {
                "a" => Ok(ExpressionValue::Float(1.234)),
                "b" => Ok(ExpressionValue::String("Foo".to_string())),
                _ => bail!("Unknown variable: {}", name),
            }
        }
        let ast = parse_expression("`foo${a}bar`").unwrap();
        let result = eval(&ast, &var_lookup).unwrap();
        assert_eq!(result, ExpressionValue::String("foo1.234bar".to_string()));
    }
}

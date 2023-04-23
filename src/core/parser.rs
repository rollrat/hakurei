use std::error::Error;

use super::{
    semantic::SemanticType,
    tokenizer::{TokenType, Tokenizer},
};

pub struct Parser {
    tokenizer: Tokenizer,
}

pub enum NodeType {
    CommandExpression,
    ExpressionAnd,
    ExpressionAndRight,
    ExpressionOr,
    ExpressionOrRight,
    ExpressionCase,
    FunctionExpression,
    Arguments,
}

#[derive(Debug)]
pub struct CommandExpressionNode {
    pub expr_and: Box<ExpressionAndNode>,
    pub semantic_type: Option<SemanticType>,
}

#[derive(Debug)]
pub struct ExpressionAndNode {
    pub expr_ors: Vec<Box<ExpressionOrNode>>,
    pub semantic_type: Option<SemanticType>,
}

#[derive(Debug)]
pub struct ExpressionOrNode {
    pub expr_cases: Vec<Box<ExpressionCaseNode>>,
    pub semantic_type: Option<SemanticType>,
}

#[derive(Debug)]
pub struct ExpressionCaseNode {
    pub expr_and: Option<Box<ExpressionAndNode>>,
    pub func: Option<Box<FunctionExpressionNode>>,
    pub semantic_type: Option<SemanticType>,
}

#[derive(Debug)]
pub struct FunctionExpressionNode {
    pub name: String,
    pub is_use: bool, // A function is used as an argument to another function.
    pub args: Option<Box<ArgumentsNode>>,
    pub semantic_type: Option<SemanticType>,
}

#[derive(Debug)]
pub struct ArgumentsNode {
    pub args: Vec<Box<ArgumentNode>>,
    pub semantic_type: Option<SemanticType>,
}

#[derive(Debug)]
pub struct ArgumentNode {
    pub value: Option<String>,
    pub expr_and: Option<Box<ExpressionAndNode>>,
}

impl Parser {
    pub fn from(target: &str) -> Parser {
        Parser {
            tokenizer: Tokenizer::from(target),
        }
    }

    pub fn parse(&mut self) -> Result<CommandExpressionNode, Box<dyn Error>> {
        let result = self.parse_expr_and()?;

        let nt = self.tokenizer.next();

        if nt.token_type != TokenType::Eof {
            Err(format!("Unexpected token \"{}\".", nt.content.unwrap()).into())
        } else {
            Ok(CommandExpressionNode {
                expr_and: result,
                semantic_type: None,
            })
        }
    }

    fn parse_expr_and(&mut self) -> Result<Box<ExpressionAndNode>, Box<dyn Error>> {
        let mut or_nodes: Vec<Box<ExpressionOrNode>> = Vec::new();

        or_nodes.push(self.parse_expr_or()?);

        while self.tokenizer.lookup() == TokenType::And {
            // consume &
            self.tokenizer.next();
            or_nodes.push(self.parse_expr_or()?);
        }

        Ok(Box::new(ExpressionAndNode {
            expr_ors: or_nodes,
            semantic_type: None,
        }))
    }

    fn parse_expr_or(&mut self) -> Result<Box<ExpressionOrNode>, Box<dyn Error>> {
        let mut case_nodes: Vec<Box<ExpressionCaseNode>> = Vec::new();

        case_nodes.push(self.parse_expr_case()?);

        while self.tokenizer.lookup() == TokenType::Or {
            // consume |
            self.tokenizer.next();
            case_nodes.push(self.parse_expr_case()?);
        }

        Ok(Box::new(ExpressionOrNode {
            semantic_type: None,
            expr_cases: case_nodes,
        }))
    }

    fn parse_expr_case(&mut self) -> Result<Box<ExpressionCaseNode>, Box<dyn Error>> {
        if self.tokenizer.lookup() == TokenType::BraceStart {
            // consume (
            self.tokenizer.next();

            let _result = Box::new(ExpressionCaseNode {
                expr_and: Some(self.parse_expr_and()?),
                func: None,
                semantic_type: None,
            });

            // consume )
            if self.tokenizer.next().token_type != TokenType::BraceEnd {
                return Err("expect )".into());
            }

            return Ok(_result);
        }

        if self.tokenizer.lookup() != TokenType::Name {
            return Err("expect name".into());
        }

        Ok(Box::new(ExpressionCaseNode {
            expr_and: None,
            func: Some(self.parse_func()?),
            semantic_type: None,
        }))
    }

    fn parse_func(&mut self) -> Result<Box<FunctionExpressionNode>, Box<dyn Error>> {
        // consume name
        let name = self.tokenizer.next();

        if self.tokenizer.lookup() != TokenType::BraceStart {
            return Ok(Box::new(FunctionExpressionNode {
                name: name.content.unwrap(),
                is_use: true,
                args: None,
                semantic_type: None,
            }));
        }

        // consume (
        self.tokenizer.next();

        if self.tokenizer.lookup() == TokenType::BraceEnd {
            // consume )
            self.tokenizer.next();

            return Ok(Box::new(FunctionExpressionNode {
                name: name.content.unwrap(),
                is_use: false,
                args: None,
                semantic_type: None,
            }));
        }

        let args = self.parse_args()?;

        if self.tokenizer.lookup() != TokenType::BraceEnd {
            return Err("expect )".into());
        }

        // consume )
        self.tokenizer.next();

        Ok(Box::new(FunctionExpressionNode {
            name: name.content.unwrap(),
            is_use: false,
            args: Some(args),
            semantic_type: None,
        }))
    }

    fn parse_args(&mut self) -> Result<Box<ArgumentsNode>, Box<dyn Error>> {
        let mut args: Vec<Box<ArgumentNode>> = Vec::new();

        args.push(self.parse_arg()?);

        while self.tokenizer.lookup() == TokenType::Comma {
            // comsume ,
            self.tokenizer.next();
            args.push(self.parse_arg()?);
        }

        Ok(Box::new(ArgumentsNode {
            semantic_type: None,
            args,
        }))
    }

    fn parse_arg(&mut self) -> Result<Box<ArgumentNode>, Box<dyn Error>> {
        if self.tokenizer.lookup() == TokenType::Const {
            // consume const
            let co = self.tokenizer.next();

            if self.tokenizer.lookup() != TokenType::Comma {
                return Ok(Box::new(ArgumentNode {
                    value: Some(co.content.unwrap()),
                    expr_and: None,
                }));
            }

            // consume ,
            self.tokenizer.next();

            return Ok(Box::new(ArgumentNode {
                value: Some(co.content.unwrap()),
                expr_and: None,
            }));
        }

        let expr_and = self.parse_expr_and()?;

        return Ok(Box::new(ArgumentNode {
            value: None,
            expr_and: Some(expr_and),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn parser_test() {
        let mut p = Parser::from("title:startswith(\"abcd\")");
        let root = p.parse().unwrap();

        let n = &root.expr_and.expr_ors[0].expr_cases[0]
            .func
            .as_ref()
            .unwrap()
            .name;

        let s = root.expr_and.expr_ors[0].expr_cases[0]
            .func
            .as_ref()
            .unwrap()
            .clone()
            .args
            .as_ref()
            .unwrap()
            .args[0]
            .value
            .as_ref()
            .unwrap();

        assert_eq!(n, "title:startswith");
        assert_eq!(s, "abcd");
    }

    #[test]
    fn parse_test_2() {
        let mut p = Parser::from("group_sum(reduce(title:contains(\"동방\"), category))");
        p.parse().unwrap();
        let mut p = Parser::from("count(set(reduce(title:contains(\"동방\"), category)))");
        p.parse().unwrap();
        let mut p = Parser::from("map(reduce(title:startswith(\"서든\") | title:endswith(\"어택\"), category), select_max_len)");
        p.parse().unwrap();
    }
}

use std::error::Error;

use super::tokenizer::{TokenType, Tokenizer};

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

pub trait Node {
    fn get_type(&self) -> NodeType;
}

#[derive(Debug)]
pub struct CommandExpressionNode {
    pub expr_and: Box<ExpressionAndNode>,
}

#[derive(Debug)]
pub struct ExpressionAndNode {
    pub expr_or: Option<Box<ExpressionOrNode>>,
    pub expr_and: Option<Box<ExpressionAndRightNode>>,
}

#[derive(Debug)]
pub struct ExpressionAndRightNode {
    pub expr_or: Option<Box<ExpressionOrNode>>,
    pub expr_and: Option<Box<ExpressionAndRightNode>>,
}

#[derive(Debug)]
pub struct ExpressionOrNode {
    pub expr_case: Option<Box<ExpressionCaseNode>>,
    pub expr_or: Option<Box<ExpressionOrRightNode>>,
}

#[derive(Debug)]
pub struct ExpressionOrRightNode {
    pub expr_case: Option<Box<ExpressionCaseNode>>,
    pub expr_or: Option<Box<ExpressionOrRightNode>>,
}

#[derive(Debug)]
pub struct ExpressionCaseNode {
    pub expr_and: Option<Box<ExpressionAndNode>>,
    pub func: Option<Box<FunctionExpressionNode>>,
}

#[derive(Debug)]
pub struct FunctionExpressionNode {
    pub name: String,
    pub is_use: bool, // A function is used as an argument to another function.
    pub args: Option<Box<ArgumentsNode>>,
}

#[derive(Debug)]
pub struct ArgumentsNode {
    pub value: Option<String>,
    pub expr_and: Option<Box<ExpressionAndNode>>,
    pub next_args: Option<Box<ArgumentsNode>>,
}

impl Node for CommandExpressionNode {
    fn get_type(&self) -> NodeType {
        NodeType::CommandExpression
    }
}

impl Node for ExpressionAndNode {
    fn get_type(&self) -> NodeType {
        NodeType::ExpressionAnd
    }
}

impl Node for ExpressionAndRightNode {
    fn get_type(&self) -> NodeType {
        NodeType::ExpressionAndRight
    }
}

impl Node for ExpressionOrNode {
    fn get_type(&self) -> NodeType {
        NodeType::ExpressionOr
    }
}

impl Node for ExpressionOrRightNode {
    fn get_type(&self) -> NodeType {
        NodeType::ExpressionOrRight
    }
}

impl Node for ExpressionCaseNode {
    fn get_type(&self) -> NodeType {
        NodeType::ExpressionCase
    }
}

impl Node for FunctionExpressionNode {
    fn get_type(&self) -> NodeType {
        NodeType::FunctionExpression
    }
}

impl Node for ArgumentsNode {
    fn get_type(&self) -> NodeType {
        NodeType::Arguments
    }
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
            Ok(CommandExpressionNode { expr_and: result })
        }
    }

    fn parse_expr_and(&mut self) -> Result<Box<ExpressionAndNode>, Box<dyn Error>> {
        Ok(Box::new(ExpressionAndNode {
            expr_or: self.parse_expr_or()?,
            expr_and: self.parse_expr_and_lr()?,
        }))
    }

    fn parse_expr_and_lr(&mut self) -> Result<Option<Box<ExpressionAndRightNode>>, Box<dyn Error>> {
        if self.tokenizer.lookup() != TokenType::And {
            return Ok(None);
        }

        // consume &
        self.tokenizer.next();

        Ok(Some(Box::new(ExpressionAndRightNode {
            expr_or: self.parse_expr_or()?,
            expr_and: self.parse_expr_and_lr()?,
        })))
    }

    fn parse_expr_or(&mut self) -> Result<Option<Box<ExpressionOrNode>>, Box<dyn Error>> {
        Ok(Some(Box::new(ExpressionOrNode {
            expr_case: self.parse_expr_case()?,
            expr_or: self.parse_expr_or_lr()?,
        })))
    }

    fn parse_expr_or_lr(&mut self) -> Result<Option<Box<ExpressionOrRightNode>>, Box<dyn Error>> {
        if self.tokenizer.lookup() != TokenType::Or {
            return Ok(None);
        }

        // consume |
        self.tokenizer.next();

        Ok(Some(Box::new(ExpressionOrRightNode {
            expr_case: self.parse_expr_case()?,
            expr_or: self.parse_expr_or_lr()?,
        })))
    }

    fn parse_expr_case(&mut self) -> Result<Option<Box<ExpressionCaseNode>>, Box<dyn Error>> {
        if self.tokenizer.lookup() == TokenType::BraceStart {
            // consume (
            self.tokenizer.next();

            let _result = Box::new(ExpressionCaseNode {
                expr_and: Some(self.parse_expr_and()?),
                func: None,
            });

            // consume )
            if self.tokenizer.next().token_type != TokenType::BraceEnd {
                return Err("expect )".into());
            }

            return Ok(Some(_result));
        }

        if self.tokenizer.lookup() != TokenType::Name {
            return Err("expect name".into());
        }

        Ok(Some(Box::new(ExpressionCaseNode {
            expr_and: None,
            func: Some(self.parse_func()?),
        })))
    }

    fn parse_func(&mut self) -> Result<Box<FunctionExpressionNode>, Box<dyn Error>> {
        // consume name
        let name = self.tokenizer.next();

        if self.tokenizer.lookup() != TokenType::BraceStart {
            return Ok(Box::new(FunctionExpressionNode {
                name: name.content.unwrap(),
                is_use: true,
                args: None,
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
        }))
    }

    fn parse_args(&mut self) -> Result<Box<ArgumentsNode>, Box<dyn Error>> {
        if self.tokenizer.lookup() == TokenType::Const {
            // consume const
            let co = self.tokenizer.next();

            if self.tokenizer.lookup() != TokenType::Comma {
                return Ok(Box::new(ArgumentsNode {
                    value: Some(co.content.unwrap()),
                    expr_and: None,
                    next_args: None,
                }));
            }

            // consume ,
            self.tokenizer.next();

            return Ok(Box::new(ArgumentsNode {
                value: Some(co.content.unwrap()),
                expr_and: None,
                next_args: Some(self.parse_args()?),
            }));
        }

        let expr_and = self.parse_expr_and()?;

        if self.tokenizer.lookup() != TokenType::Comma {
            return Ok(Box::new(ArgumentsNode {
                value: None,
                expr_and: Some(expr_and),
                next_args: None,
            }));
        }

        // comsume ,
        self.tokenizer.next();

        Ok(Box::new(ArgumentsNode {
            value: None,
            expr_and: Some(expr_and),
            next_args: Some(self.parse_args()?),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn parser_test() {
        let mut p = Parser::from("title:startswith(\"abcd\")");
        let root = p.parse().unwrap();

        let n = &root
            .expr_and
            .expr_or
            .as_ref()
            .unwrap()
            .expr_case
            .as_ref()
            .unwrap()
            .func
            .as_ref()
            .unwrap()
            .name;

        let s = root
            .expr_and
            .expr_or
            .as_ref()
            .unwrap()
            .expr_case
            .as_ref()
            .unwrap()
            .func
            .as_ref()
            .unwrap()
            .clone()
            .args
            .as_ref()
            .unwrap()
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

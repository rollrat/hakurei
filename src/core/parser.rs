use std::error::Error;

use super::tokenizer::{TokenType, Tokenizer};

pub struct Parser {
    tokenizer: Tokenizer,
}

pub enum NodeType {
    Ellipsion,
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

pub struct EllipsionNode {}

pub struct CommandExpressionNode {
    pub expr_and: Box<ExpressionAndNode>,
}

pub struct ExpressionAndNode {
    pub expr_or: Box<ExpressionOrNode>,
    pub expr_and: Box<dyn Node>,
}

pub struct ExpressionAndRightNode {
    pub expr_or: Box<ExpressionOrNode>,
    pub expr_and: Box<dyn Node>,
}

pub struct ExpressionOrNode {
    pub expr_case: Box<ExpressionCaseNode>,
    pub expr_or: Box<dyn Node>,
}

pub struct ExpressionOrRightNode {
    pub expr_case: Box<dyn Node>,
    pub expr_or: Box<dyn Node>,
}

pub struct ExpressionCaseNode {
    pub expr_and: Option<Box<ExpressionAndNode>>,
    pub func: Option<Box<FunctionExpressionNode>>,
}

pub struct FunctionExpressionNode {
    pub name: String,
    pub args: Option<Box<ArgumentsNode>>,
}

pub struct ArgumentsNode {
    pub value: Option<String>,
    pub expr_and: Option<Box<ExpressionAndNode>>,
    pub next_args: Option<Box<ArgumentsNode>>,
}

impl Node for EllipsionNode {
    fn get_type(&self) -> NodeType {
        NodeType::Ellipsion
    }
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
        Ok(CommandExpressionNode {
            expr_and: self.parse_expr_and()?,
        })
    }

    fn parse_expr_and(&mut self) -> Result<Box<ExpressionAndNode>, Box<dyn Error>> {
        Ok(Box::new(ExpressionAndNode {
            expr_or: self.parse_expr_or()?,
            expr_and: self.parse_expr_and_lr()?,
        }))
    }

    fn parse_expr_and_lr(&mut self) -> Result<Box<dyn Node>, Box<dyn Error>> {
        if self.tokenizer.lookup() != TokenType::And {
            return Ok(Box::new(EllipsionNode {}));
        }

        // consume &
        self.tokenizer.next();

        Ok(Box::new(ExpressionAndRightNode {
            expr_or: self.parse_expr_or()?,
            expr_and: self.parse_expr_and_lr()?,
        }))
    }

    fn parse_expr_or(&mut self) -> Result<Box<ExpressionOrNode>, Box<dyn Error>> {
        Ok(Box::new(ExpressionOrNode {
            expr_case: self.parse_expr_case()?,
            expr_or: self.parse_expr_or_lr(),
        }))
    }

    fn parse_expr_or_lr(&mut self) -> Box<dyn Node> {
        if self.tokenizer.lookup() != TokenType::And {
            return Box::new(EllipsionNode {});
        }

        // consume |
        self.tokenizer.next();

        Box::new(ExpressionOrRightNode {
            expr_case: self.parse_expr_or_lr(),
            expr_or: self.parse_expr_or_lr(),
        })
    }

    fn parse_expr_case(&mut self) -> Result<Box<ExpressionCaseNode>, Box<dyn Error>> {
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

            return Ok(_result);
        }

        if self.tokenizer.lookup() != TokenType::Name {
            return Err("expect name".into());
        }

        Ok(Box::new(ExpressionCaseNode {
            expr_and: None,
            func: Some(self.parse_func()?),
        }))
    }

    fn parse_func(&mut self) -> Result<Box<FunctionExpressionNode>, Box<dyn Error>> {
        // consume name
        let name = self.tokenizer.next();

        if self.tokenizer.next().token_type != TokenType::BraceStart {
            return Ok(Box::new(FunctionExpressionNode {
                name: name.content.unwrap(),
                args: None,
            }));
        }

        if self.tokenizer.lookup() == TokenType::BraceEnd {
            // consume )
            self.tokenizer.next();

            return Ok(Box::new(FunctionExpressionNode {
                name: name.content.unwrap(),
                args: None,
            }));
        }

        Ok(Box::new(FunctionExpressionNode {
            name: name.content.unwrap(),
            args: Some(self.parse_args()?),
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

        let n = &root.expr_and.expr_or.expr_case.func.as_ref().unwrap().name;
        let s = root
            .expr_and
            .expr_or
            .expr_case
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
}

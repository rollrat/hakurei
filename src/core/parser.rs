use std::{error::Error, process::Child, slice::SliceIndex};

struct Tokenizer {
    target: Vec<char>,
    ptr: usize,
}

#[derive(Debug, PartialEq)]
enum TokenType {
    Error,
    Eof,
    And,        // &
    Or,         // |
    BraceStart, // (
    BraceEnd,   // )
    Comma,      // ,
    Name,       // [_a-zA-Z$][_:a-zA-Z0-9$]**
    Const,      // number ([0-9]+), string ("([^\\"]|\\")*")
}

struct Token {
    token_type: TokenType,
    content: Option<String>,
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

impl Tokenizer {
    fn from<'a>(target: &'a str) -> Tokenizer {
        Tokenizer {
            target: target.chars().collect::<Vec<char>>(),
            ptr: 0,
        }
    }

    fn next(&mut self) -> Token {
        while !self.check_end() && self.target[self.ptr] == ' ' {
            self.ptr += 1;
        }

        if self.check_end() {
            return Token {
                token_type: TokenType::Eof,
                content: None,
            };
        }

        match self.target[self.ptr] {
            '(' | ')' | '&' | '|' | ',' => {
                self.ptr += 1;
                Token {
                    token_type: match self.target[self.ptr - 1] {
                        '(' => TokenType::BraceStart,
                        ')' => TokenType::BraceEnd,
                        '&' => TokenType::And,
                        '|' => TokenType::Or,
                        ',' => TokenType::Comma,
                        _ => panic!(),
                    },
                    content: None,
                }
            }
            '0'..='9' => {
                let mut num = String::new();

                while !self.check_end() {
                    let ch = self.target[self.ptr];

                    if '0' <= ch && ch <= '9' {
                        num.push(ch);
                    } else {
                        break;
                    }

                    self.ptr += 1;
                }

                Token {
                    token_type: TokenType::Const,
                    content: Some(num),
                }
            }
            'a'..='z' | '_' | '$' => {
                let mut name = String::new();

                while !self.check_end() {
                    let ch = self.target[self.ptr];

                    if ('a' <= ch && ch <= 'z')
                        || ('0' <= ch && ch <= '9')
                        || ch == '_'
                        || ch == '$'
                        || ch == ':'
                    {
                        name.push(ch);
                    } else {
                        break;
                    }

                    self.ptr += 1;
                }

                Token {
                    token_type: TokenType::Name,
                    content: Some(name),
                }
            }
            '"' => {
                let mut string = String::new();

                self.ptr += 1;

                while !self.check_end() {
                    let ch = self.target[self.ptr];

                    if ch == '"' {
                        self.ptr += 1;

                        return Token {
                            token_type: TokenType::Const,
                            content: Some(string),
                        };
                    }

                    if ch == '\\' {
                        self.ptr += 1;

                        if self.check_end() {
                            return Token {
                                token_type: TokenType::Error,
                                content: None,
                            };
                        }

                        match self.target[self.ptr] {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            _ => string.push(ch),
                        }
                    }

                    string.push(ch);

                    self.ptr += 1;
                }

                Token {
                    token_type: TokenType::Error,
                    content: None,
                }
            }
            _ => Token {
                token_type: TokenType::Error,
                content: None,
            },
        }
    }

    fn check_end(&self) -> bool {
        self.ptr >= self.target.len()
    }

    fn lookup(&mut self) -> TokenType {
        let last_pos = self.ptr;
        let lookup = self.next();
        self.ptr = last_pos;

        lookup.token_type
    }
}

struct Parser {
    tokenizer: Tokenizer,
}

enum NodeType {
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

trait Node {
    fn get_type(&self) -> NodeType;
}

struct EllipsionNode {}

struct CommandExpressionNode {
    expr_and: Box<ExpressionAndNode>,
}

struct ExpressionAndNode {
    expr_or: Box<ExpressionOrNode>,
    expr_and: Box<dyn Node>,
}

struct ExpressionAndRightNode {
    expr_or: Box<ExpressionOrNode>,
    expr_and: Box<dyn Node>,
}

struct ExpressionOrNode {
    expr_case: Box<ExpressionCaseNode>,
    expr_or: Box<dyn Node>,
}

struct ExpressionOrRightNode {
    expr_case: Box<dyn Node>,
    expr_or: Box<dyn Node>,
}

struct ExpressionCaseNode {
    expr_and: Option<Box<ExpressionAndNode>>,
    func: Option<Box<FunctionExpressionNode>>,
}

struct FunctionExpressionNode {
    name: String,
    args: Option<Box<ArgumentsNode>>,
}

struct ArgumentsNode {
    value: Option<String>,
    expr_and: Option<Box<ExpressionAndNode>>,
    next_args: Option<Box<ArgumentsNode>>,
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
    fn from(target: &str) -> Parser {
        Parser {
            tokenizer: Tokenizer::from(target),
        }
    }

    fn parse(&mut self) -> Result<CommandExpressionNode, Box<dyn Error>> {
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
            return Err("expect (".into());
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
    use crate::core::parser::TokenType;

    use super::{Parser, Tokenizer};

    fn token_type(target: &str) -> TokenType {
        Tokenizer::from(target).next().token_type
    }

    #[test]
    fn tokenizer_unit_test() {
        assert_eq!(token_type(""), TokenType::Eof);
        assert_eq!(token_type("123"), TokenType::Const);
        assert_eq!(token_type("asdf"), TokenType::Name);
        assert_eq!(token_type("\"zxcjklv\\\"zxbxcvb\""), TokenType::Const);
    }

    #[test]
    fn tokenizer_test() {
        let mut tok = Tokenizer::from("title:startswith(\"abcd\")");
        assert_eq!(tok.next().token_type, TokenType::Name);
        assert_eq!(tok.next().token_type, TokenType::BraceStart);
        assert_eq!(tok.next().token_type, TokenType::Const);
        assert_eq!(tok.next().token_type, TokenType::BraceEnd);
        assert_eq!(tok.next().token_type, TokenType::Eof);
    }

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

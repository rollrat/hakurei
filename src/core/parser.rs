use std::{process::Child, slice::SliceIndex};

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
            '(' | ')' | '&' | '|' => {
                self.ptr += 1;
                Token {
                    token_type: match self.target[self.ptr - 1] {
                        '(' => TokenType::BraceStart,
                        ')' => TokenType::BraceEnd,
                        '&' => TokenType::And,
                        '|' => TokenType::Or,
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
}

struct Parser {
    tokenizer: Tokenizer,
}

enum NodeType {
    CommandExpression,
    ExpressionAnd,
    ExpressionAndRight,
    ExpressionOr,
    ExpressionOrRight,
    ExpressionCase,
    FunctionExpression,
}

trait Node {
    fn get_type(&self) -> NodeType;
}

struct CommandExpressionNode {}

struct ExpressionAndNode {}

struct ExpressionAndRightNode {}

struct ExpressionOrNode {}

struct ExpressionOrRightNode {}

struct ExpressionCaseNode {}

struct FunctionExpressionNode {}

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

impl Parser {
    fn from(target: &str) -> Tokenizer {
        Tokenizer::from(target)
    }

    fn parse() -> CommandExpressionNode {
        CommandExpressionNode {}
    }
}

#[cfg(test)]
mod tests {
    use crate::core::parser::TokenType;

    use super::Tokenizer;

    fn token_type(target: &str) -> TokenType {
        Tokenizer::from(target).next().token_type
    }

    #[test]
    fn parse_unit_test() {
        assert_eq!(token_type(""), TokenType::Eof);
        assert_eq!(token_type("123"), TokenType::Const);
        assert_eq!(token_type("asdf"), TokenType::Name);
        assert_eq!(token_type("\"zxcjklv\\\"zxbxcvb\""), TokenType::Const);
    }

    #[test]
    fn parse_test() {
        let mut tok = Tokenizer::from("title:startswith(\"abcd\")");
        assert_eq!(tok.next().token_type, TokenType::Name);
        assert_eq!(tok.next().token_type, TokenType::BraceStart);
        assert_eq!(tok.next().token_type, TokenType::Const);
        assert_eq!(tok.next().token_type, TokenType::BraceEnd);
        assert_eq!(tok.next().token_type, TokenType::Eof);
    }
}

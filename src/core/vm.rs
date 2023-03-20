use std::{error::Error, process::Command};

use crate::index::{category::CategoryIndex, title::TitleIndex};

use super::{
    parser::{
        ArgumentsNode, CommandExpressionNode, ExpressionAndNode, ExpressionAndRightNode,
        ExpressionCaseNode, ExpressionOrNode, ExpressionOrRightNode, FunctionExpressionNode,
        Parser,
    },
    semantic::{check_semantic, SemanticType},
};

#[derive(PartialEq, Debug)]
pub enum RuntimePrimitiveObject {
    Article(String),  // article title
    Category(String), // category name
    Integer(usize),
    String(String),
    Function,
}

#[derive(Debug)]
pub enum RuntimeObject {
    None,
    Primitive(RuntimePrimitiveObject),
    Array(Vec<Box<RuntimeObject>>),
    Set(Vec<Box<RuntimeObject>>),
    Tuple(Vec<Box<RuntimeObject>>),
}

impl RuntimeObject {
    fn eq(&self, other: &RuntimeObject) -> bool {
        match self {
            RuntimeObject::None => match other {
                RuntimeObject::None => true,
                _ => false,
            },
            RuntimeObject::Primitive(e) => match other {
                RuntimeObject::Primitive(o) => true,
                _ => false,
            },
            RuntimeObject::Array(e) => match other {
                RuntimeObject::Array(o) => true,
                _ => false,
            },
            RuntimeObject::Set(e) => match other {
                RuntimeObject::Set(o) => true,
                _ => false,
            },
            RuntimeObject::Tuple(e) => match other {
                RuntimeObject::Tuple(o) => {
                    e.len() == o.len() && e.iter().zip(o).all(|(x, y)| x.eq(y))
                }
                _ => false,
            },
        }
    }
}

pub struct VirtualMachine {
    root: CommandExpressionNode,
    title_index: TitleIndex,
    category_index: CategoryIndex,
    return_type: SemanticType,
}

impl RuntimeObject {
    pub fn print_type(&self) {
        println!("{:?}", self);
    }

    pub fn print(&self) {
        match self {
            RuntimeObject::None => todo!(),
            RuntimeObject::Primitive(_) => todo!(),
            RuntimeObject::Array(_) => todo!(),
            RuntimeObject::Set(_) => todo!(),
            RuntimeObject::Tuple(_) => todo!(),
        }
    }
}

impl VirtualMachine {
    pub fn from(
        command: &str,
        title_index: TitleIndex,
        category_index: CategoryIndex,
    ) -> Result<Self, Box<dyn Error>> {
        let root = Parser::from(command).parse()?;
        let semantic_type = check_semantic(&root)?;

        Ok(VirtualMachine {
            root: root,
            title_index: title_index,
            category_index: category_index,
            return_type: semantic_type,
        })
    }

    // this operation do not ocurred error, except memory allocation
    pub fn run(&self) -> RuntimeObject {
        self.visit_root(&self.root)
    }

    fn visit_root(&self, node: &CommandExpressionNode) -> RuntimeObject {
        self.visit_expr_and(&node.expr_and)
    }

    fn visit_expr_and(&self, node: &ExpressionAndNode) -> RuntimeObject {
        let l_value = match &node.expr_or {
            Some(node) => self.visit_expr_or(&node),
            None => RuntimeObject::None,
        };

        let r_value = match &node.expr_and {
            Some(node) => self.visit_expr_and_lr(&node),
            None => RuntimeObject::None,
        };

        if r_value.eq(&RuntimeObject::None) {
            r_value
        } else {
            let mut l_value = match l_value {
                RuntimeObject::Array(e) => e,
                _ => panic!("unreachable"),
            };

            let mut r_value = match r_value {
                RuntimeObject::Array(e) => e,
                _ => panic!("unreachable"),
            };

            l_value.append(&mut r_value);

            RuntimeObject::Array(l_value)
        }
    }

    fn visit_expr_and_lr(&self, node: &ExpressionAndRightNode) -> RuntimeObject {
        todo!()
    }

    fn visit_expr_or(&self, node: &ExpressionOrNode) -> RuntimeObject {
        todo!()
    }

    fn visit_expr_or_lr(&self, node: &ExpressionOrRightNode) -> RuntimeObject {
        todo!()
    }

    fn visit_expr_case(&self, node: &ExpressionCaseNode) -> RuntimeObject {
        todo!()
    }

    fn visit_func(&self, node: &FunctionExpressionNode) -> RuntimeObject {
        todo!()
    }

    fn visit_args(&self, node: &ArgumentsNode) -> RuntimeObject {
        todo!()
    }
}

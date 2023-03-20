use std::error::Error;

use crate::index::{category::CategoryIndex, title::TitleIndex};

use super::{
    parser::{CommandExpressionNode, Parser},
    semantic::{check_semantic, SemanticType},
};

#[derive(Debug)]
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
    Array(Box<SemanticType>),
    Set(Box<SemanticType>),
    Tuple(Vec<Box<SemanticType>>),
}

pub struct VirtualMachine {
    root: CommandExpressionNode,
    title_index: TitleIndex,
    category_index: CategoryIndex,
    return_type: SemanticType,
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

    pub fn run(&self) -> RuntimeObject {
        todo!()
    }
}

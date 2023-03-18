use std::error::Error;

use super::parser::{
    CommandExpressionNode, ExpressionAndNode, ExpressionAndRightNode, ExpressionCaseNode,
    ExpressionOrNode, ExpressionOrRightNode, FunctionExpressionNode,
};

#[derive(PartialEq)]
pub enum SemanticType {
    None,
    ArticleArray,
    ArticleSet,
    ArticleWithCountArray,
    CategorySet,
    CategoryArray,
    CategoryWithCountArray,
    Integer,
    String,
}

impl SemanticType {
    fn is_article(&self) -> bool {
        *self == SemanticType::ArticleArray
            || *self == SemanticType::ArticleSet
            || *self == SemanticType::ArticleWithCountArray
    }

    fn is_category(&self) -> bool {
        *self == SemanticType::CategoryArray
            || *self == SemanticType::CategorySet
            || *self == SemanticType::CategoryWithCountArray
    }

    fn is_array(&self) -> bool {
        *self == SemanticType::ArticleArray || *self == SemanticType::CategoryArray
    }

    fn is_tuple_array(&self) -> bool {
        *self == SemanticType::ArticleWithCountArray
            || *self == SemanticType::CategoryWithCountArray
    }

    fn is_set(&self) -> bool {
        *self == SemanticType::ArticleSet || *self == SemanticType::CategorySet
    }

    fn is_const(&self) -> bool {
        *self == SemanticType::Integer || *self == SemanticType::String
    }

    fn can_concat(&self, other: &SemanticType) -> bool {
        // none and/or is possible
        !self.is_const() && *self == *other
    }
}

pub fn check_semantic(root: &CommandExpressionNode) -> Result<SemanticType, Box<dyn Error>> {
    visit_expr_and(&root.expr_and)
}

fn visit_expr_and(node: &ExpressionAndNode) -> Result<SemanticType, Box<dyn Error>> {
    let l_type = match &node.expr_or {
        Some(node) => visit_expr_or(&node)?,
        None => SemanticType::None,
    };

    let r_type = match &node.expr_and {
        Some(node) => visit_expr_and_lr(&node)?,
        None => SemanticType::None,
    };

    if l_type.can_concat(&r_type) {
        return Ok(l_type);
    }

    Err("".into())
}

fn visit_expr_and_lr(node: &ExpressionAndRightNode) -> Result<SemanticType, Box<dyn Error>> {
    todo!()
}

fn visit_expr_or(node: &ExpressionOrNode) -> Result<SemanticType, Box<dyn Error>> {
    todo!()
}

fn visit_expr_or_lr(node: &ExpressionOrRightNode) -> Result<SemanticType, Box<dyn Error>> {
    todo!()
}

fn visit_expr_case(node: &ExpressionCaseNode) -> Result<SemanticType, Box<dyn Error>> {
    todo!()
}

fn visit_func(node: &FunctionExpressionNode) -> Result<SemanticType, Box<dyn Error>> {
    todo!()
}

fn function_name_check(func: &str) -> bool {
    match func {
        "title:contains" | "title:startswith" | "title:endswith" => true,
        "reduce" | "set" | "count" | "map" | "group_sum" | "ref" => true,
        "category" | "select_max_len" | "select_min_len" => true,
        _ => false,
    }
}

fn param_check(node: &FunctionExpressionNode) -> bool {
    match &node.name[..] {
        "title:contains" | "title:startswith" | "title:endswith" => {
            node.args.as_ref().unwrap().value.is_some()
        }
        "reduce" => false,
        _ => false,
    }
}

fn infer_type(node: &FunctionExpressionNode) -> SemanticType {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::core::semantic::function_name_check;

    #[test]
    fn function_name_check_test() {
        assert_eq!(function_name_check("title:contains"), true);
    }
}

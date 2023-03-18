use std::{any::Any, error::Error};

use crate::core::parser::NodeType;

use super::parser::{
    CommandExpressionNode, ExpressionAndNode, ExpressionAndRightNode, ExpressionCaseNode,
    ExpressionOrNode, ExpressionOrRightNode, FunctionExpressionNode,
};

enum SemanticType {
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

pub fn check_semantic(root: &CommandExpressionNode) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn visit_expr_and(node: &ExpressionAndNode) -> Result<SemanticType, Box<dyn Error + '_>> {
    let l_type = visit_expr_or(node.expr_or.as_ref())?;

    let r_node = node.expr_and.as_ref();

    if r_node.get_type() == NodeType::Ellipsion {}

    let r_type = visit_expr_and_lr(
        r_node
            .as_any()
            .downcast_ref::<ExpressionAndRightNode>()
            .unwrap(),
    )?;

    todo!()
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

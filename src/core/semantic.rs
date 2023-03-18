use std::error::Error;

use super::parser::{
    CommandExpressionNode, ExpressionAndNode, ExpressionAndRightNode, ExpressionCaseNode,
    ExpressionOrNode, ExpressionOrRightNode, FunctionExpressionNode,
};

#[derive(PartialEq, Clone, Debug)]
pub enum SemanticPrimitiveType {
    Article,
    Category,
    Integer,
    String,
}

#[derive(Clone, Debug)]
pub enum SemanticType {
    None,
    Primitive(SemanticPrimitiveType),
    Array(Box<SemanticType>),
    Set(Box<SemanticType>),
    Tuple(Vec<Box<SemanticType>>),
}

impl SemanticType {
    fn eq(&self, other: &SemanticType) -> bool {
        match self {
            SemanticType::None => match other {
                SemanticType::None => true,
                _ => false,
            },
            SemanticType::Primitive(e) => match other {
                SemanticType::Primitive(o) => e == o,
                _ => false,
            },
            SemanticType::Array(e) => match other {
                SemanticType::Array(o) => e.eq(o),
                _ => false,
            },
            SemanticType::Set(e) => match other {
                SemanticType::Set(o) => e.eq(o),
                _ => false,
            },
            SemanticType::Tuple(e) => match other {
                SemanticType::Tuple(o) => {
                    e.len() == o.len() && e.iter().zip(o).all(|(x, y)| x.eq(y))
                }
                _ => false,
            },
        }
    }

    fn infer_concat(&self, other: &SemanticType) -> Result<SemanticType, Box<dyn Error>> {
        match self {
            SemanticType::None => Ok(other.clone()),
            SemanticType::Primitive(e) => match other {
                SemanticType::None => Ok(Self::None),
                SemanticType::Primitive(o) => {
                    if e == o {
                        Ok(self.clone())
                    } else {
                        Err(format!("Types {:#?} and {:#?} do not match! The two elements have different types and cannot be merged.", e, o).into())
                    }
                }
                SemanticType::Array(o) => match o.as_ref() {
                    SemanticType::None => Ok(o.as_ref().clone()),
                    SemanticType::Primitive(p) => {
                        if e.eq(p) {
                            Ok(other.clone())
                        } else {
                            Err(format!("Types {:#?} and {:#?} do not match! To concaterate an element to a array, the type of the element in the array must match the type of the element.", e, p).into())
                        }
                    }
                    _ => Err(format!("To concaterate a primitive type and an array, the element type of the array must match the primitive type.").into()),
                },
                SemanticType::Set(o) => match o.as_ref() {
                    SemanticType::None => Ok(o.as_ref().clone()),
                    SemanticType::Primitive(p) => {
                        if e.eq(p) {
                            Ok(other.clone())
                        } else {
                            Err(format!("Types {:#?} and {:#?} do not match! To merge an element to a set, the type of the element in the set must match the type of the element.", e, p).into())
                        }
                    }
                    _ => Err(format!("To merge a primitive type and an set, the element type of the set must match the primitive type.").into()),
                },
                SemanticType::Tuple(_) => Err(format!("Primitive types and tuples cannot be concatenated").into()),
            },
            SemanticType::Array(e) => match e.as_ref() {
                SemanticType::None => Ok(self.clone()),
                SemanticType::Primitive(_) => other.infer_concat(self),
                SemanticType::Array(o) => {
                    if e.eq(o) {
                        Ok(self.clone())
                    } else {
                        Err(format!("Types {:#?} and {:#?} do not match! To concaterate two arrays, the array elements must have the same type.", e.as_ref(), o.as_ref()).into())
                    }
                }
                _ => Err(format!("Arrays can only be concaterated arrays or primitives.").into()),
            },
            SemanticType::Set(e) => match e.as_ref() {
                SemanticType::None => Ok(self.clone()),
                SemanticType::Primitive(_) => other.infer_concat(self),
                SemanticType::Set(o) => {
                    if e.eq(o) {
                        Ok(self.clone())
                    } else {
                        Err(format!("Types {:#?} and {:#?} do not match! To merge two sets, they must have the same element type.", e.as_ref(), o.as_ref()).into())
                    }
                }
                _ => Err(format!("Sets can only be merged sets or primitives.").into()),
            },
            SemanticType::Tuple(_) => {
                if self.eq(other) {
                    Ok(Self::Array(Box::new(self.clone())))
                } else {
                    Err(format!("If you want to merge tuples to create an array, both mergers must have the same tuple type.").into())
                }
            }
        }
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

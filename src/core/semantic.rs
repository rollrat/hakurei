use std::error::Error;

use super::parser::{
    ArgumentsNode, CommandExpressionNode, ExpressionAndNode, ExpressionAndRightNode,
    ExpressionCaseNode, ExpressionOrNode, ExpressionOrRightNode, FunctionExpressionNode,
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
                        Err(format!("Types {:?} and {:?} do not match! The two elements have different types and cannot be merged.", e, o).into())
                    }
                }
                SemanticType::Array(o) => match o.as_ref() {
                    SemanticType::None => Ok(o.as_ref().clone()),
                    SemanticType::Primitive(p) => {
                        if e.eq(p) {
                            Ok(other.clone())
                        } else {
                            Err(format!("Types {:?} and {:?} do not match! To concaterate an element to a array, the type of the element in the array must match the type of the element.", e, p).into())
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
                            Err(format!("Types {:?} and {:?} do not match! To merge an element to a set, the type of the element in the set must match the type of the element.", e, p).into())
                        }
                    }
                    _ => Err(format!("To merge a primitive type and an set, the element type of the set must match the primitive type.").into()),
                },
                SemanticType::Tuple(_) => Err(format!("Primitive types and tuples cannot be concatenated").into()),
            },
            SemanticType::Array(e) => match other {
                SemanticType::None => Ok(self.clone()),
                SemanticType::Primitive(_) => other.infer_concat(self),
                SemanticType::Array(o) => {
                    if e.eq(o) {
                        Ok(self.clone())
                    } else {
                        Err(format!("Types {:?} and {:?} do not match! To concaterate two arrays, the array elements must have the same type.", e.as_ref(), o.as_ref()).into())
                    }
                }
                _ => Err(format!("Arrays can only be concaterated arrays or primitives.").into()),
            },
            SemanticType::Set(e) => match other {
                SemanticType::None => Ok(self.clone()),
                SemanticType::Primitive(_) => other.infer_concat(self),
                SemanticType::Set(o) => {
                    if e.eq(o) {
                        Ok(self.clone())
                    } else {
                        Err(format!("Types {:?} and {:?} do not match! To merge two sets, they must have the same element type.", e.as_ref(), o.as_ref()).into())
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

    fn infer_intercross(&self, other: &SemanticType) -> Result<SemanticType, Box<dyn Error>> {
        match self {
            SemanticType::None => Ok(Self::None),
            SemanticType::Primitive(_) => {
                Err("An intercrossing subject cannot be a primitive type.".into())
            }
            SemanticType::Array(_) => {
                if self.eq(other) {
                    Ok(self.clone())
                } else {
                    Err(format!("Types {:?} and {:?} do not match! Two arrays with different element types cannot be intercrossed.", self, other).into())
                }
            }
            SemanticType::Set(_) => {
                if self.eq(other) {
                    Ok(self.clone())
                } else {
                    Err(format!("Types {:?} and {:?} do not match! Two arrays with different element types cannot be intercrossed.", self, other).into())
                }
            }
            SemanticType::Tuple(_) => {
                Err("An intercrossing subject cannot be a tuple type.".into())
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

    if r_type.eq(&SemanticType::None) {
        return Ok(l_type);
    }

    l_type.infer_intercross(&r_type)
}

fn visit_expr_and_lr(node: &ExpressionAndRightNode) -> Result<SemanticType, Box<dyn Error>> {
    let l_type = match &node.expr_or {
        Some(node) => visit_expr_or(&node)?,
        None => SemanticType::None,
    };

    let r_type = match &node.expr_and {
        Some(node) => visit_expr_and_lr(&node)?,
        None => SemanticType::None,
    };

    if r_type.eq(&SemanticType::None) {
        return Ok(l_type);
    }

    l_type.infer_intercross(&r_type)
}

fn visit_expr_or(node: &ExpressionOrNode) -> Result<SemanticType, Box<dyn Error>> {
    let l_type = match &node.expr_case {
        Some(node) => visit_expr_case(&node)?,
        None => SemanticType::None,
    };

    let r_type = match &node.expr_or {
        Some(node) => visit_expr_or_lr(&node)?,
        None => SemanticType::None,
    };

    if r_type.eq(&SemanticType::None) {
        return Ok(l_type);
    }

    l_type.infer_concat(&r_type)
}

fn visit_expr_or_lr(node: &ExpressionOrRightNode) -> Result<SemanticType, Box<dyn Error>> {
    let l_type = match &node.expr_case {
        Some(node) => visit_expr_case(&node)?,
        None => SemanticType::None,
    };

    let r_type = match &node.expr_or {
        Some(node) => visit_expr_or_lr(&node)?,
        None => SemanticType::None,
    };

    if r_type.eq(&SemanticType::None) {
        return Ok(l_type);
    }

    l_type.infer_concat(&r_type)
}

fn visit_expr_case(node: &ExpressionCaseNode) -> Result<SemanticType, Box<dyn Error>> {
    if let Some(expr_and) = &node.expr_and {
        return visit_expr_and(&expr_and);
    }

    visit_func(node.func.as_ref().unwrap())
}

fn visit_func(node: &FunctionExpressionNode) -> Result<SemanticType, Box<dyn Error>> {
    match &node.name[..] {
        "title:contains" | "title:startswith" | "title:endswith" => {
            let p_check = param_check_lazy_1(
                node,
                &SemanticType::Primitive(SemanticPrimitiveType::String),
            )?;

            if !p_check {
                return Err(format!(
                    "{} function accepts only {:?} type!",
                    &node.name,
                    SemanticType::Primitive(SemanticPrimitiveType::String)
                )
                .into());
            }

            return Ok(SemanticType::Array(Box::new(SemanticType::Primitive(
                SemanticPrimitiveType::Article,
            ))));
        }
        _ => Err(format!("'{}' function not found!", &node.name).into()),
    }
}

fn param_check_lazy_1(
    node: &FunctionExpressionNode,
    target_type: &SemanticType,
) -> Result<bool, Box<dyn Error>> {
    if let Some(args) = &node.args {
        match args.next_args {
            Some(_) => Ok(false),
            None => param_type_eq_lazy(args, target_type),
        }
    } else {
        Ok(false)
    }
}

fn param_check_lazy_2(
    node: &FunctionExpressionNode,
    first_target_type: &SemanticType,
    second_target_type: &SemanticType,
) -> Result<bool, Box<dyn Error>> {
    if let Some(args_first) = &node.args {
        match &args_first.next_args {
            Some(args_second) => match &args_second.next_args {
                Some(_) => Ok(false),
                None => Ok(param_type_eq_lazy(args_first, first_target_type)?
                    && param_type_eq_lazy(args_second, second_target_type)?),
            },
            None => Ok(false),
        }
    } else {
        Ok(false)
    }
}

fn param_type_eq_lazy(
    args: &ArgumentsNode,
    target_type: &SemanticType,
) -> Result<bool, Box<dyn Error>> {
    Ok(if let Some(value) = &args.value {
        match target_type {
            SemanticType::Primitive(prim_type) => match prim_type {
                SemanticPrimitiveType::Integer => {
                    if let Ok(_) = value.parse::<usize>() {
                        true
                    } else {
                        false
                    }
                }
                SemanticPrimitiveType::String => true,
                _ => false,
            },
            _ => false,
        }
    } else if let Some(expr_and) = &args.expr_and {
        let l_type = visit_expr_and(&expr_and)?;

        match l_type {
            SemanticType::Primitive(e) => match target_type {
                SemanticType::Primitive(o) => e == *o,
                _ => false,
            },
            SemanticType::Array(_) => match target_type {
                SemanticType::Array(_) => true,
                _ => false,
            },
            SemanticType::Set(_) => match target_type {
                SemanticType::Set(_) => true,
                _ => false,
            },
            _ => false,
        }
    } else {
        false
    })
}

#[cfg(test)]
mod tests {
    use crate::core::semantic::{SemanticPrimitiveType, SemanticType};

    fn get_si() -> SemanticType {
        SemanticType::Array(Box::new(SemanticType::Set(Box::new(SemanticType::Tuple(
            vec![
                Box::new(SemanticType::Primitive(SemanticPrimitiveType::String)),
                Box::new(SemanticType::Primitive(SemanticPrimitiveType::Integer)),
            ],
        )))))
    }

    fn get_ii() -> SemanticType {
        SemanticType::Array(Box::new(SemanticType::Set(Box::new(SemanticType::Tuple(
            vec![
                Box::new(SemanticType::Primitive(SemanticPrimitiveType::Integer)),
                Box::new(SemanticType::Primitive(SemanticPrimitiveType::Integer)),
            ],
        )))))
    }

    #[test]
    fn type_eq_test() {
        assert!(get_si().eq(&get_si()));
        assert!(!get_si().eq(&get_ii()));
    }

    #[test]
    fn type_infer_concat_test() {
        assert!(match get_si().infer_concat(&get_si()) {
            Ok(_) => true,
            Err(_) => false,
        });
        assert!(match get_si().infer_concat(&get_ii()) {
            Ok(_) => false,
            Err(_) => true,
        });
    }
}

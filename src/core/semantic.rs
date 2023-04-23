use std::error::Error;

use super::parser::{
    ArgumentNode, CommandExpressionNode, ExpressionAndNode, ExpressionCaseNode, ExpressionOrNode,
    FunctionExpressionNode,
};

#[derive(PartialEq, Clone, Debug)]
pub enum SemanticPrimitiveType {
    Article,
    Category,
    Integer,
    String,
    Boolean,
}

#[derive(Clone, Debug)]
pub enum SemanticFunctionType {
    None,
    Category,
    Select,
    Redirect,
    UnwrapTuple1,
    UnwrapTuple2,
    CmpArray,
    CmpTuple1,
    CmpTuple2,
}

#[derive(Clone, Debug)]
pub enum SemanticType {
    None,
    Primitive(SemanticPrimitiveType),
    Array(Box<SemanticType>),
    Set(Box<SemanticType>),
    Tuple(Vec<Box<SemanticType>>),
    Function(SemanticFunctionType),
}

impl SemanticType {
    fn eq(&self, other: &SemanticType) -> bool {
        match self {
            SemanticType::None => match other {
                SemanticType::Primitive(_) | SemanticType::None => true,
                _ => false,
            },
            SemanticType::Primitive(e) => match other {
                SemanticType::Primitive(o) => e == o,
                SemanticType::None => true,
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
            SemanticType::Function(_) => unreachable!(),
        }
    }

    fn infer_concat(&self, other: &SemanticType) -> Result<SemanticType, Box<dyn Error>> {
        match self {
            SemanticType::None => Ok(other.clone()),
            SemanticType::Primitive(e) => match other {
                SemanticType::None => Ok(Self::None),
                SemanticType::Primitive(p) => {
                    if e == p {
                        Ok(self.clone())
                    } else {
                        Err(format!("Types {:?} and {:?} do not match! The two elements have different types and cannot be merged.", e, p).into())
                    }
                }
                SemanticType::Array(o) => match o.as_ref() {
                    SemanticType::None => Ok(o.as_ref().clone()),
                    SemanticType::Primitive(p) => {
                        if e == p {
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
                        if e == p {
                            Ok(other.clone())
                        } else {
                            Err(format!("Types {:?} and {:?} do not match! To merge an element to a set, the type of the element in the set must match the type of the element.", e, p).into())
                        }
                    }
                    _ => Err(format!("To merge a primitive type and an set, the element type of the set must match the primitive type.").into()),
                },
                SemanticType::Tuple(_) => Err(format!("Primitive types and tuples cannot be concatenated").into()),
                SemanticType::Function(_) => todo!(),
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
            SemanticType::Function(_) => todo!(),
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
            SemanticType::Function(_) => {
                Err("An intercrossing subject cannot be a functino type.".into())
            }
        }
    }
}

pub fn check_semantic(root: &mut CommandExpressionNode) -> Result<SemanticType, Box<dyn Error>> {
    visit_expr_and(&mut root.expr_and)
}

fn visit_expr_and(node: &mut ExpressionAndNode) -> Result<SemanticType, Box<dyn Error>> {
    let mut return_type = visit_expr_or(&mut node.expr_ors[0])?;

    for expr_or in &mut node.expr_ors.iter_mut().skip(1) {
        let target_type = visit_expr_or(expr_or)?;
        return_type = return_type.infer_intercross(&target_type)?;
    }

    node.semantic_type = Some(return_type.clone());

    Ok(return_type)
}

fn visit_expr_or(node: &mut ExpressionOrNode) -> Result<SemanticType, Box<dyn Error>> {
    let mut return_type = visit_expr_case(&mut node.expr_cases[0])?;

    for expr_case in &mut node.expr_cases.iter_mut().skip(1) {
        let target_type = visit_expr_case(expr_case)?;
        return_type = return_type.infer_concat(&target_type)?;
    }

    node.semantic_type = Some(return_type.clone());

    Ok(return_type)
}

fn visit_expr_case(node: &mut ExpressionCaseNode) -> Result<SemanticType, Box<dyn Error>> {
    if let Some(expr_and) = &mut node.expr_and {
        let result = visit_expr_and(expr_and);

        if let Ok(e) = &result {
            node.semantic_type = Some(e.clone());
        }

        return result;
    }

    let result = visit_func(node.func.as_mut().unwrap());

    if let Ok(e) = &result {
        node.semantic_type = Some(e.clone());
    }

    result
}

fn visit_func(node: &mut FunctionExpressionNode) -> Result<SemanticType, Box<dyn Error>> {
    if node.is_use {
        return visit_func_use(node);
    }

    let result = match &node.name[..] {
        "title:exact" | "title:contains" | "title:startswith" | "title:endswith" => {
            param_check_lazy_1(
                node,
                &SemanticType::Primitive(SemanticPrimitiveType::String),
            )?;

            Ok(SemanticType::Array(Box::new(SemanticType::Primitive(
                SemanticPrimitiveType::Article,
            ))))
        }
        "count" => {
            let check_is_array =
                param_check_lazy_1(node, &SemanticType::Array(Box::new(SemanticType::None)));

            if let Ok(_) = check_is_array {
                node.semantic_type = Some(SemanticType::Primitive(SemanticPrimitiveType::Integer));
                return Ok(SemanticType::Primitive(SemanticPrimitiveType::Integer));
            }

            let check_is_set =
                param_check_lazy_1(node, &SemanticType::Set(Box::new(SemanticType::None)));

            if let Ok(_) = check_is_set {
                node.semantic_type = Some(SemanticType::Primitive(SemanticPrimitiveType::Integer));
                return Ok(SemanticType::Primitive(SemanticPrimitiveType::Integer));
            }

            check_is_array?;

            panic!("unreachable")
        }
        "set" => {
            param_check_lazy_1(node, &SemanticType::Array(Box::new(SemanticType::None)))?;

            let first_param_type = visit_expr_and(node.args[0].expr_and.as_mut().unwrap())?;
            let first_param_uncapsuled = match first_param_type {
                SemanticType::Array(e) => e.clone(),
                _ => panic!("unreachable"),
            };

            Ok(SemanticType::Set(first_param_uncapsuled))
        }
        "group_sum" => {
            param_check_lazy_1(node, &SemanticType::Array(Box::new(SemanticType::None)))?;

            let first_param_type = visit_expr_and(node.args[0].expr_and.as_mut().unwrap())?;
            let first_param_uncapsuled = match first_param_type {
                SemanticType::Array(e) => e.clone(),
                _ => panic!("unreachable"),
            };

            match first_param_uncapsuled.as_ref() {
                SemanticType::Primitive(_) => {
                    let tuple_type = vec![
                        first_param_uncapsuled,
                        Box::new(SemanticType::Primitive(SemanticPrimitiveType::Integer))
                    ];

                    Ok(SemanticType::Array(Box::new(SemanticType::Tuple(tuple_type))))
                }
                _ => Err(format!("The generic reference of the first parameter Array of 'group_sum' must be a primitive type. Current generic type is '{:?}'.", *first_param_uncapsuled).into())
            }
        }
        "map" => {
            param_check_lazy_2(
                node,
                &SemanticType::Array(Box::new(SemanticType::None)),
                &SemanticType::Function(SemanticFunctionType::None),
            )?;

            let first_param_type = visit_expr_and(node.args[0].expr_and.as_mut().unwrap())?;
            let first_param_uncapsuled = match first_param_type {
                SemanticType::Array(e) => e.clone(),
                _ => panic!("unreachable"),
            };

            let second_param_func = get_second_param_func(node);

            let semantic_type = visit_func_use(second_param_func)?;
            let func_type = match semantic_type {
                SemanticType::Function(e) => e,
                _ => unreachable!(),
            };

            Ok(SemanticType::Array(Box::new(visit_infer_func_use(
                &func_type,
                Some(&first_param_uncapsuled),
                None,
            )?)))
        }
        "flatten" => {
            param_check_lazy_1(
                node,
                &SemanticType::Array(Box::new(SemanticType::Array(Box::new(SemanticType::None)))),
            )?;

            let first_param_type = visit_expr_and(node.args[0].expr_and.as_mut().unwrap())?;
            let first_param_uncapsuled = match first_param_type {
                SemanticType::Array(e) => e.clone(),
                _ => panic!("unreachable"),
            };

            Ok(*first_param_uncapsuled)
        }
        "filter" => todo!(),
        "sort" => {
            param_check_lazy_2(
                node,
                &SemanticType::Array(Box::new(SemanticType::None)),
                &SemanticType::Function(SemanticFunctionType::None),
            )?;

            let first_param_type = visit_expr_and(node.args[0].expr_and.as_mut().unwrap())?;
            let first_param_uncapsuled = match &first_param_type {
                SemanticType::Array(e) => e.clone(),
                _ => panic!("unreachable"),
            };

            let second_param_func = get_second_param_func(node);

            let semantic_type = visit_func_use(second_param_func)?;
            let func_type = match semantic_type {
                SemanticType::Function(e) => e,
                _ => unreachable!(),
            };

            let func_ret_type = visit_infer_func_use(
                &func_type,
                Some(&first_param_uncapsuled),
                Some(&first_param_uncapsuled),
            )?;

            if !func_ret_type.eq(&SemanticType::Primitive(SemanticPrimitiveType::Integer)) {
                return Err(format!("The second parameter of sort function is must be return i32! Currently, {:?} is retuned!", func_ret_type).into());
            }

            Ok(first_param_type)
        }
        "bind" => todo!(),
        _ => Err(format!("'{}' function not found!", &node.name).into()),
    };

    if let Ok(e) = &result {
        node.semantic_type = Some(e.clone());
    }

    result
}

fn get_second_param_func(node: &FunctionExpressionNode) -> &FunctionExpressionNode {
    &node.args[1].expr_and.as_ref().unwrap().expr_ors[0].expr_cases[0]
        .func
        .as_ref()
        .unwrap()
}

fn visit_func_use(node: &FunctionExpressionNode) -> Result<SemanticType, Box<dyn Error>> {
    let result = match &node.name[..] {
        "category" => Ok(SemanticType::Function(SemanticFunctionType::Category)),
        "select_min_len" | "select_max_len" => {
            Ok(SemanticType::Function(SemanticFunctionType::Select))
        }
        "redirect" => Ok(SemanticType::Function(SemanticFunctionType::Redirect)),
        "unwrap_tuple1" => Ok(SemanticType::Function(SemanticFunctionType::UnwrapTuple1)),
        "unwrap_tuple2" => Ok(SemanticType::Function(SemanticFunctionType::UnwrapTuple2)),
        "cmp_array" => Ok(SemanticType::Function(SemanticFunctionType::CmpArray)),
        "cmp_tuple1" => Ok(SemanticType::Function(SemanticFunctionType::CmpTuple1)),
        "cmp_tuple2" => Ok(SemanticType::Function(SemanticFunctionType::CmpTuple2)),
        _ => Err(format!("'{}' function not found!", &node.name).into()),
    };

    result
}

fn visit_infer_func_use(
    func: &SemanticFunctionType,
    param1: Option<&SemanticType>,
    param2: Option<&SemanticType>,
) -> Result<SemanticType, Box<dyn Error>> {
    match func {
        SemanticFunctionType::None => unreachable!(),
        SemanticFunctionType::Category => {
            if let Some(p1) = &param1 {
                match p1 {
                    SemanticType::Primitive(e) => match e {
                        SemanticPrimitiveType::Article => Ok(SemanticType::Array(Box::new(
                            SemanticType::Primitive(SemanticPrimitiveType::Category),
                        ))),
                        _ => Err(format!(
                            "'category' function's first param type must be 'Article' instead of {:?}!",
                            p1,
                        )
                        .into()),
                    },
                    _ => Err(format!(
                        "'category' function's first param type must be 'Article' instead of {:?}!",
                        p1,
                    )
                    .into()),
                }
            } else {
                Err(format!("'category' function must have one parameter").into())
            }
        }
        SemanticFunctionType::Select => {
            if let Some(p1) = &param1 {
                match p1 {
                    SemanticType::Array(e) => Ok(*e.clone()),
                    SemanticType::Set(e) => Ok(*e.clone()),
                    _ => Err(format!(
                        "'select_*' function's first param type must be 'Article' instead of '{:?}'!",
                        p1,
                    )
                    .into()),
                }
            } else {
                Err(format!("'select_*' function must have one parameter").into())
            }
        }
        SemanticFunctionType::Redirect => {
            if let Some(p1) = &param1 {
                match p1 {
                    SemanticType::Primitive(e) => match e {
                        SemanticPrimitiveType::Article => {
                            Ok(SemanticType::Primitive(SemanticPrimitiveType::Article))
                        }
                        _ => Err(format!(
                        "'redirect' function's first param type must be 'Article' instead of {:?}!",
                        p1,
                    )
                        .into()),
                    },
                    _ => Err(format!(
                        "'redirect' function's first param type must be 'Article' instead of {:?}!",
                        p1,
                    )
                    .into()),
                }
            } else {
                Err(format!("'redirect' function must have one parameter").into())
            }
        }
        SemanticFunctionType::UnwrapTuple1 => {
            if let Some(p1) = &param1 {
                match p1 {
                    SemanticType::Tuple(e) => Ok(*e[0].clone()),
                    _ => Err(format!(
                        "'unwrap_tuple1' function's first param type must be <Tuple> instead of {:?}!",
                        p1,
                    )
                    .into()),
                }
            } else {
                Err(format!("'unwrap_tuple1' function must have one parameter").into())
            }
        }
        SemanticFunctionType::UnwrapTuple2 => {
            if let Some(p1) = &param1 {
                match p1 {
                    SemanticType::Tuple(e) => Ok(*e[1].clone()),
                    _ => Err(format!(
                        "'unwrap_tuple2' function's first param type must be <Tuple> instead of {:?}!",
                        p1,
                    )
                    .into()),
                }
            } else {
                Err(format!("'unwrap_tuple2' function must have one parameter").into())
            }
        }
        SemanticFunctionType::CmpArray => {
            if param1.is_some() && param2.is_some() {
                let p1 = param1.unwrap();
                let p2 = param2.unwrap();

                if p1.eq(&p2) {
                    Ok(SemanticType::Primitive(SemanticPrimitiveType::Integer))
                } else {
                    Err(format!(
                        "'cmp_array''s arguments must have same type. currently ({:?}, {:?})",
                        p1, p2
                    )
                    .into())
                }
            } else {
                Err(format!("'cmp_array' function must have two parameter").into())
            }
        }
        SemanticFunctionType::CmpTuple1 | SemanticFunctionType::CmpTuple2 => {
            if param1.is_some() && param2.is_some() {
                if param1.unwrap().eq(&param2.unwrap()) {
                    Ok(SemanticType::Primitive(SemanticPrimitiveType::Integer))
                } else {
                    Err(format!("'cmp_tuple1' function must have same parameters type").into())
                }
            } else {
                Err(format!("'cmp_tuple1' function must have two parameter").into())
            }
        }
    }
}

fn visit_arg(node: &mut ArgumentNode) -> Result<SemanticType, Box<dyn Error>> {
    let result = Ok(if let Some(_) = &node.value {
        SemanticType::Primitive(SemanticPrimitiveType::String)
    } else if let Some(expr_and) = &mut node.expr_and {
        visit_expr_and(expr_and)?
    } else {
        panic!("unreachable")
    });

    if let Ok(e) = &result {
        node.semantic_type = Some(e.clone());
    }

    result
}

fn param_check_lazy_1(
    node: &mut FunctionExpressionNode,
    target_type: &SemanticType,
) -> Result<(), Box<dyn Error>> {
    if node.args.len() != 1 {
        Err(format!("'{}' function must have one parameter!", &node.name).into())
    } else {
        if param_type_eq_generic(&mut node.args[0], target_type)? {
            node.args[0].semantic_type = Some(target_type.clone());
            Ok(())
        } else {
            Err(format!(
                "The first parameter of '{}' function must be '{:?}' type! Current type is '{:?}'.",
                &node.name,
                target_type,
                visit_arg(&mut node.args[0])?
            )
            .into())
        }
    }
}

fn param_check_lazy_2(
    node: &mut FunctionExpressionNode,
    first_target_type: &SemanticType,
    second_target_type: &SemanticType,
) -> Result<(), Box<dyn Error>> {
    if node.args.len() != 2 {
        Err(format!("'{}' function must have two parameter!", &node.name).into())
    } else {
        if !param_type_eq_generic(&mut node.args[0], first_target_type)? {
            Err(format!(
                "The first parameter of '{}' function must be '{:?}' type! Current type is '{:?}'.",
                &node.name,
                first_target_type,
                visit_arg(&mut node.args[0])?
            )
            .into())
        } else if !param_type_eq_generic(&mut node.args[1], second_target_type)? {
            Err(format!(
                "The second parameter of '{}' function must be '{:?}' type! Current type is '{:?}'.",
                &node.name,
                second_target_type,
                visit_arg(&mut node.args[1])?
            )
            .into())
        } else {
            node.args[1].semantic_type = Some(second_target_type.clone());
            Ok(())
        }
    }
}

fn param_type_eq_generic(
    arg: &mut ArgumentNode,
    target_type: &SemanticType,
) -> Result<bool, Box<dyn Error>> {
    Ok(if let Some(value) = &arg.value {
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
    } else if let Some(expr_and) = &mut arg.expr_and {
        let l_type = visit_expr_and(expr_and)?;

        match l_type {
            SemanticType::Primitive(e) => match target_type {
                SemanticType::Primitive(o) => e == *o,
                _ => false,
            },
            SemanticType::Array(e) => match target_type {
                SemanticType::Array(o) => e.eq(o),
                _ => false,
            },
            SemanticType::Set(e) => match target_type {
                SemanticType::Set(o) => e.eq(o),
                _ => false,
            },
            SemanticType::Function(_) => match target_type {
                SemanticType::Function(_) => true,
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
    use crate::core::{
        parser::Parser,
        semantic::{SemanticPrimitiveType, SemanticType},
    };

    use super::check_semantic;

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

    #[test]
    fn type_infer_test() {
        let mut p = Parser::from("title:startswith(\"abcd\") & title:startswith(\"abcd\")");
        let mut root = p.parse().unwrap();

        let inferred_type = check_semantic(&mut root).unwrap();

        let target_type = SemanticType::Array(Box::new(SemanticType::Primitive(
            SemanticPrimitiveType::Article,
        )));

        assert!(inferred_type.eq(&target_type));
    }

    #[test]
    fn type_infer_test_2() {
        let mut p = Parser::from("flatten(map(title:contains(\"동방\"), category))");
        let mut root = p.parse().unwrap();

        let inferred_type = check_semantic(&mut root).unwrap();

        let target_type = SemanticType::Array(Box::new(SemanticType::Primitive(
            SemanticPrimitiveType::Category,
        )));

        assert!(inferred_type.eq(&target_type));
    }

    #[test]
    fn type_infer_test_3() {
        let mut p = Parser::from("count(flatten(map(title:contains(\"동방\"), category)))");
        let mut root = p.parse().unwrap();

        let inferred_type = check_semantic(&mut root).unwrap();

        let target_type = Box::new(SemanticType::Primitive(SemanticPrimitiveType::Integer));

        assert!(inferred_type.eq(&target_type));
    }

    #[test]
    fn type_infer_test_4() {
        let mut p = Parser::from("set(flatten(map(title:contains(\"동방\"), category)))");
        let mut root = p.parse().unwrap();

        let inferred_type = check_semantic(&mut root).unwrap();

        let target_type = SemanticType::Set(Box::new(SemanticType::Primitive(
            SemanticPrimitiveType::Category,
        )));

        assert!(inferred_type.eq(&target_type));
    }

    #[test]
    fn type_infer_test_5() {
        let mut p = Parser::from("group_sum(flatten(map(title:contains(\"동방\"), category)))");
        let mut root = p.parse().unwrap();

        let inferred_type = check_semantic(&mut root).unwrap();

        let target_type = SemanticType::Array(Box::new(SemanticType::Tuple(vec![
            Box::new(SemanticType::Primitive(SemanticPrimitiveType::Category)),
            Box::new(SemanticType::Primitive(SemanticPrimitiveType::Integer)),
        ])));

        assert!(inferred_type.eq(&target_type));
    }

    #[test]
    fn type_infer_test_6() {
        let mut p =
            Parser::from("sort(flatten(map(title:contains(\"동방\"), category)), cmp_array)");
        let mut root = p.parse().unwrap();

        let inferred_type = check_semantic(&mut root).unwrap();

        let target_type = SemanticType::Array(Box::new(SemanticType::Primitive(
            SemanticPrimitiveType::Category,
        )));

        assert!(inferred_type.eq(&target_type));
    }
}

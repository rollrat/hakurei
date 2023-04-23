use std::{collections::VecDeque, error::Error};

use super::{
    parser::*,
    semantic::{check_semantic, SemanticType},
};

#[derive(Debug, PartialEq)]
pub enum InstructionType {
    FunctionCall,
    UseFunction,
    Intercross,
    Concat,
    Constant,
}

#[derive(Debug)]
pub struct Instruction {
    pub id: usize,
    pub inst_type: InstructionType,
    pub semantic_type: SemanticType,
    pub data: Option<String>,
    pub params: Option<Vec<Box<Instruction>>>,
}

impl Instruction {
    pub fn to_string(&self) -> String {
        match self.inst_type {
            InstructionType::FunctionCall => format!(
                "{}({})",
                self.data.clone().unwrap(),
                self.params_to_string()
            ),
            InstructionType::Intercross => format!("&({})", self.params_to_string()),
            InstructionType::Concat => format!("|({})", self.params_to_string()),
            _ => unreachable!(),
        }
    }

    pub fn params_to_string(&self) -> String {
        if self.params.is_none() {
            String::from("{}")
        } else {
            self.params
                .as_ref()
                .unwrap()
                .iter()
                .map(|x| match x.inst_type {
                    InstructionType::UseFunction => format!("ref(\"{}\")", x.data.clone().unwrap()),
                    InstructionType::Constant => format!("\"{}\"", x.data.clone().unwrap()),
                    _ => format!("v{}", x.id),
                })
                .collect::<Vec<String>>()
                .join(", ")
        }
    }
}

pub struct IRBuilder {
    root: CommandExpressionNode,
}

impl IRBuilder {
    pub fn from(target: &str) -> Result<Self, Box<dyn Error>> {
        let mut p = Parser::from(target);

        let mut root = p.parse()?;

        check_semantic(&mut root)?;

        Ok(IRBuilder { root: root })
    }

    pub fn build(self) -> Instruction {
        let root_node = self.root;

        let mut id_count = 0;
        let head_inst = Self::visit_root(&mut id_count, root_node);

        head_inst
    }

    pub fn ir_flatten(head_inst: &Instruction) -> Vec<&Instruction> {
        let mut insts: Vec<&Instruction> = Vec::new();
        let mut dq: VecDeque<&Instruction> = VecDeque::new();

        dq.push_back(&head_inst);
        while !dq.is_empty() {
            let p = dq.pop_front().unwrap();
            insts.insert(0, p);

            if let Some(p) = &p.params {
                p.iter()
                    .rev()
                    .filter(|x| match x.inst_type {
                        InstructionType::UseFunction => false,
                        InstructionType::Constant => false,
                        _ => true,
                    })
                    .map(|x| &*x.as_ref())
                    .for_each(|x| dq.push_back(x));
            }
        }

        insts
    }

    fn visit_root(id_count: &mut usize, node: CommandExpressionNode) -> Instruction {
        Self::visit_expr_and(id_count, *node.expr_and)
    }

    fn visit_expr_and(id_count: &mut usize, node: ExpressionAndNode) -> Instruction {
        let mut expr_ors = node.expr_ors;
        let semantic_type = node.semantic_type.unwrap();

        if expr_ors.len() == 1 {
            return Self::visit_expr_or(id_count, *expr_ors.remove(0));
        }

        let mut params: Vec<Box<Instruction>> = Vec::new();

        for expr_or in expr_ors {
            let inst = Self::visit_expr_or(id_count, *expr_or);
            params.push(Box::new(inst));
        }

        *id_count += 1;

        Instruction {
            id: *id_count,
            inst_type: InstructionType::Intercross,
            semantic_type: semantic_type,
            data: None,
            params: Some(params),
        }
    }

    fn visit_expr_or(id_count: &mut usize, node: ExpressionOrNode) -> Instruction {
        let mut expr_cases = node.expr_cases;
        let semantic_type = node.semantic_type.unwrap();

        if expr_cases.len() == 1 {
            return Self::visit_expr_case(id_count, *expr_cases.remove(0));
        }

        let mut params: Vec<Box<Instruction>> = Vec::new();

        for expr_case in expr_cases {
            let inst = Self::visit_expr_case(id_count, *expr_case);
            params.push(Box::new(inst));
        }

        *id_count += 1;

        Instruction {
            id: *id_count,
            inst_type: InstructionType::Concat,
            semantic_type: semantic_type,
            data: None,
            params: Some(params),
        }
    }

    fn visit_expr_case(id_count: &mut usize, node: ExpressionCaseNode) -> Instruction {
        if let Some(expr_and) = node.expr_and {
            return Self::visit_expr_and(id_count, *expr_and);
        }

        Self::visit_func(id_count, *node.func.unwrap())
    }

    fn visit_func(id_count: &mut usize, node: FunctionExpressionNode) -> Instruction {
        let semantic_type = node.semantic_type.unwrap();

        if node.is_use {
            *id_count += 1;

            Instruction {
                id: *id_count,
                inst_type: InstructionType::UseFunction,
                semantic_type: semantic_type,
                data: Some(node.name),
                params: None,
            }
        } else {
            let mut params: Vec<Box<Instruction>> = Vec::new();

            for arg in node.args {
                let inst = if let Some(e) = arg.value {
                    *id_count += 1;

                    Instruction {
                        id: *id_count,
                        inst_type: InstructionType::Constant,
                        semantic_type: arg.semantic_type.unwrap(),
                        data: Some(e),
                        params: None,
                    }
                } else {
                    Self::visit_expr_and(id_count, *arg.expr_and.unwrap())
                };

                params.push(Box::new(inst));
            }

            *id_count += 1;

            Instruction {
                id: *id_count,
                inst_type: InstructionType::FunctionCall,
                semantic_type,
                data: Some(node.name),
                params: Some(params),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IRBuilder;

    #[test]
    fn ir_build_test() {
        // let target = "set(reduce(title:contains(\"동방\"), category))";
        let target = "count(set(flatten(map(title:contains(\"동방\"), category))))";
        // let target =
        //     "group_sum(reduce(title:startswith(\"서든\") & title:endswith(\"어택\"), category))";
        let irb = IRBuilder::from(target).unwrap();

        let head_inst = irb.build();
        let y = IRBuilder::ir_flatten(&head_inst);

        y.iter()
            .for_each(|x| println!("v{} = {} # {:?}", x.id, x.to_string(), x.semantic_type));

        assert!(false);
    }
}

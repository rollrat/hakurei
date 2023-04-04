use std::collections::{HashMap, HashSet};

use crate::{
    index::{category::CategoryIndex, title::TitleIndex},
    model::article::Article,
};

use super::ir::Instruction;

#[derive(Clone, Debug)]
pub enum RuntimeVariableAbstractPrimitiveData<'a> {
    Article(&'a Article),
    Category(&'a str),
    Integer(i64),
    String(String),
    Function(String),
}

#[derive(Clone, Debug)]
pub enum RuntimeVariableAbstractData<'a> {
    None,
    Primitive(RuntimeVariableAbstractPrimitiveData<'a>),
    Array(Box<Vec<RuntimeVariableAbstractData<'a>>>),
    Set(Box<HashSet<RuntimeVariableAbstractData<'a>>>),
    Tuple(Vec<Box<RuntimeVariableAbstractData<'a>>>),
}

#[derive(Clone, Debug)]
pub struct RuntimeVariable<'a> {
    inst: &'a Instruction,
    pub data: RuntimeVariableAbstractData<'a>,
}

pub struct RuntimeRef<'a> {
    category_index: &'a CategoryIndex,
    title_index: &'a TitleIndex,
}

pub struct VirtualMachine<'a> {
    insts: Vec<&'a Instruction>,
}

impl VirtualMachine<'_> {
    pub fn from(insts: Vec<&Instruction>) -> VirtualMachine {
        VirtualMachine { insts: insts }
    }

    pub fn run<'a>(&'a self, reference: RuntimeRef) -> RuntimeVariable<'a> {
        let mut rt_var: HashMap<usize, RuntimeVariable<'a>> = HashMap::new();

        self.insts.iter().for_each(|&inst| {
            rt_var.insert(inst.id, self.eval_inst(&reference, inst));
        });

        // safe unwrap thank to semantic analysis
        rt_var.get(&self.insts.last().unwrap().id).unwrap().clone()
    }

    fn eval_inst(&self, reference: &RuntimeRef, inst: &Instruction) -> RuntimeVariable {
        match inst.inst_type {
            crate::core::ir::InstructionType::FunctionCall => self.eval_func(reference, inst),
            crate::core::ir::InstructionType::UseFunction => self.eval_use_func(reference, inst),
            crate::core::ir::InstructionType::Intercross => self.eval_intercross(reference, inst),
            crate::core::ir::InstructionType::Concat => self.eval_concat(reference, inst),
            crate::core::ir::InstructionType::Constant => self.eval_constant(reference, inst),
        }
    }

    fn eval_func(&self, reference: &RuntimeRef, inst: &Instruction) -> RuntimeVariable {
        todo!()
    }

    fn eval_use_func(&self, reference: &RuntimeRef, inst: &Instruction) -> RuntimeVariable {
        todo!()
    }

    fn eval_intercross(&self, reference: &RuntimeRef, inst: &Instruction) -> RuntimeVariable {
        todo!()
    }

    fn eval_concat(&self, reference: &RuntimeRef, inst: &Instruction) -> RuntimeVariable {
        todo!()
    }

    fn eval_constant(&self, reference: &RuntimeRef, inst: &Instruction) -> RuntimeVariable {
        todo!()
    }
}

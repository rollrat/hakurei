use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use crate::{
    index::{
        category::CategoryIndex,
        title::{TitleIndex, TitleIndexFindOption},
    },
    model::article::Article,
};

use super::ir::{Instruction, InstructionType};

#[derive(Clone, Debug)]
pub enum RuntimeVariableAbstractPrimitiveData<'a> {
    Article(Article),
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

pub struct RuntimeEnvironment<'a> {
    rt_ref: &'a RuntimeRef<'a>,
    rt_var: &'a HashMap<usize, RuntimeVariable<'a>>,
}

pub struct VirtualMachine<'a> {
    insts: Vec<&'a Instruction>,
}

impl VirtualMachine<'_> {
    pub fn from(insts: Vec<&Instruction>) -> VirtualMachine {
        VirtualMachine { insts: insts }
    }

    pub fn run<'a>(&'a self, reference: RuntimeRef) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let mut rt_var: HashMap<usize, RuntimeVariable<'a>> = HashMap::new();

        for inst in self.insts.iter() {
            let rt_env = RuntimeEnvironment {
                rt_ref: &reference,
                rt_var: &rt_var,
            };
            rt_var.insert(inst.id, self.eval_inst(&rt_env, inst)?);
        }

        // safe unwrap thank to semantic analysis
        Ok(rt_var.get(&self.insts.last().unwrap().id).unwrap().clone())
    }

    fn eval_inst<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        match inst.inst_type {
            InstructionType::FunctionCall => self.eval_func(env, inst),
            InstructionType::UseFunction => self.eval_use_func(env, inst),
            InstructionType::Intercross => self.eval_intercross(env, inst),
            InstructionType::Concat => self.eval_concat(env, inst),
            InstructionType::Constant => self.eval_constant(env, inst),
        }
    }

    fn eval_func<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        match &inst.data.as_ref().unwrap()[..] {
            "title" | "title:contains" | "title:statswith" | "title:endswith" => {
                self.eval_func_title(env, inst)
            }
            "count" => {
                let var = env
                    .rt_var
                    .get(&inst.params.as_ref().unwrap()[0].id)
                    .unwrap();

                let result = match &var.data {
                    RuntimeVariableAbstractData::Array(e) => e.len(),
                    RuntimeVariableAbstractData::Set(e) => e.len(),
                    _ => unreachable!(),
                };

                Ok(RuntimeVariable {
                    inst,
                    data: RuntimeVariableAbstractData::Primitive(
                        RuntimeVariableAbstractPrimitiveData::Integer(result as i64),
                    ),
                })
            }
            "set" => todo!(),
            "group_sum" => todo!(),
            "reduce" => todo!(),
            _ => unreachable!(),
        }
    }

    fn eval_use_func<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        todo!()
    }

    fn eval_intercross<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        todo!()
    }

    fn eval_concat<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        todo!()
    }

    fn eval_constant<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        todo!()
    }

    fn eval_func_title<'a>(
        &self,
        env: &RuntimeEnvironment,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let what = inst.params.as_ref().unwrap()[0].data.as_ref().unwrap();

        match &inst.data.as_ref().unwrap()[..] {
            // exact match
            "title" => {
                let title = env.rt_ref.title_index.find_one_by(what);

                if let Some(exact_title) = title {
                    let article = env.rt_ref.title_index.get(exact_title);

                    Ok(RuntimeVariable {
                        inst,
                        data: RuntimeVariableAbstractData::Primitive(
                            RuntimeVariableAbstractPrimitiveData::Article(article.unwrap()),
                        ),
                    })
                } else {
                    Err(format!("Cannot found title '{}'", what).into())
                }
            }
            "title:exact" | "title:contains" | "title:statswith" | "title:endswith" => {
                let titles = env.rt_ref.title_index.find_by(
                    what,
                    match &inst.data.as_ref().unwrap()[..] {
                        "title:exact" => TitleIndexFindOption::Extact,
                        "title:contains" => TitleIndexFindOption::Contains,
                        "title:startswith" => TitleIndexFindOption::StartsWith,
                        "title:endswith" => TitleIndexFindOption::EndsWith,
                        _ => unreachable!(),
                    },
                );

                if titles.is_empty() {
                    Err(format!("Cannot found title '{}'", what).into())
                } else {
                    let articles: Vec<RuntimeVariableAbstractData> = titles
                        .iter()
                        .map(|t| {
                            let article = env.rt_ref.title_index.get(t);
                            let article = if let Some(e) = article {
                                e
                            } else {
                                env.rt_ref.title_index.get_no_redirect(t).unwrap()
                            };

                            RuntimeVariableAbstractData::Primitive(
                                RuntimeVariableAbstractPrimitiveData::Article(article),
                            )
                        })
                        .collect();

                    Ok(RuntimeVariable {
                        inst,
                        data: RuntimeVariableAbstractData::Array(Box::new(articles)),
                    })
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        core::{
            ir::IRBuilder,
            vm::{RuntimeVariable, RuntimeVariableAbstractPrimitiveData},
        },
        index::{category::CategoryIndex, title::TitleIndex},
        DEFAULT_CATEGORY_INDEX_PATH, DEFAULT_DUMP_PATH, DEFAULT_TITLE_INDEX_PATH,
    };

    use super::{RuntimeRef, RuntimeVariableAbstractData, VirtualMachine};

    #[test]
    fn vm_title_load_test() {
        let target = "count(title:contains(\"동방\"))";
        let irb = IRBuilder::from(target).unwrap();

        let head_inst = irb.build();
        let insts = IRBuilder::ir_flatten(&head_inst);

        let vm = VirtualMachine::from(insts);

        let tindex = TitleIndex::load(DEFAULT_DUMP_PATH, DEFAULT_TITLE_INDEX_PATH).unwrap();
        let cindex = CategoryIndex::load(DEFAULT_CATEGORY_INDEX_PATH).unwrap();

        let rt_ref = RuntimeRef {
            category_index: &cindex,
            title_index: &tindex,
        };

        let result = vm.run(rt_ref).unwrap();

        assert_eq!(_uncover_integer(&result), 851);
    }

    fn _uncover_integer(rt_var: &RuntimeVariable) -> i64 {
        match &rt_var.data {
            RuntimeVariableAbstractData::Primitive(e) => match e {
                RuntimeVariableAbstractPrimitiveData::Integer(value) => *value,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

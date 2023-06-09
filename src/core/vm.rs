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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RuntimeVariableAbstractPrimitiveData<'a> {
    Article(Article),
    Category(&'a str),
    Integer(i64),
    String(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum RuntimeVariableAbstractData<'a> {
    None,
    Primitive(RuntimeVariableAbstractPrimitiveData<'a>),
    Array(Box<Vec<RuntimeVariableAbstractData<'a>>>),
    Set(Box<Vec<RuntimeVariableAbstractData<'a>>>),
    Tuple(Vec<RuntimeVariableAbstractData<'a>>),
}

macro_rules! unwrap_rv {
    ($target_enum:ident, $target:ident, $return_type:ty) => {
        ::paste::paste! {
            pub fn [<$target:lower>](&self) -> Option<&$return_type> {
                match self {
                    $target_enum::$target(e) => Some(e),
                    _ => None,
                }
            }
            pub fn [<unwrap_ $target:lower>](&self) -> &$return_type {
                match self {
                    $target_enum::$target(e) => e,
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! unwrap_rvp {
    ($target:ident, $return_type:ty) => {
        unwrap_rv!(RuntimeVariableAbstractPrimitiveData, $target, $return_type);
    };
}

macro_rules! unwrap_rvr {
    ($target:ident, $return_type:ty) => {
        unwrap_rv!(RuntimeVariableAbstractData, $target, $return_type);
    };
}

impl<'a> RuntimeVariableAbstractPrimitiveData<'a> {
    unwrap_rvp!(Article, Article);
    unwrap_rvp!(Category, str);
    unwrap_rvp!(Integer, i64);
    unwrap_rvp!(String, String);
}

impl<'a> RuntimeVariableAbstractData<'a> {
    unwrap_rvr!(Primitive, RuntimeVariableAbstractPrimitiveData<'a>);
    unwrap_rvr!(Array, Box<Vec<RuntimeVariableAbstractData<'a>>>);
    unwrap_rvr!(Set, Box<Vec<RuntimeVariableAbstractData<'a>>>);
    unwrap_rvr!(Tuple, Vec<RuntimeVariableAbstractData<'a>>);
}

#[derive(Clone, Debug)]
pub struct RuntimeVariable<'a> {
    pub inst: &'a Instruction,
    pub data: RuntimeVariableAbstractData<'a>,
}

pub struct RuntimeRef<'a> {
    pub category_index: &'a CategoryIndex,
    pub title_index: &'a TitleIndex,
    pub articles: Option<Vec<Article>>,
}

pub struct VirtualMachine<'a> {
    insts: Vec<&'a Instruction>,
    debug: bool,
}

#[macro_export]
macro_rules! vm_from {
    ($a: expr, $b: expr) => {
        VirtualMachine::from($a, $b)
    };
    ($a: expr) => {
        VirtualMachine::from($a, false)
    };
}

impl VirtualMachine<'_> {
    pub fn from(insts: Vec<&Instruction>, debug: bool) -> VirtualMachine {
        VirtualMachine {
            insts: insts,
            debug,
        }
    }

    pub fn run<'a>(
        &'a self,
        reference: &'a RuntimeRef,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let mut rt_var: HashMap<usize, RuntimeVariable<'a>> = HashMap::new();

        for inst in self.insts.iter() {
            rt_var.insert(inst.id, self.eval_inst(&rt_var, &reference, inst)?);
        }

        // safe unwrap thank to semantic analysis
        Ok(rt_var.get(&self.insts.last().unwrap().id).unwrap().clone())
    }

    fn eval_inst<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        reference: &'a RuntimeRef,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        match inst.inst_type {
            InstructionType::FunctionCall => self.eval_func(var, reference, inst),
            InstructionType::Intercross => self.eval_intercross(var, inst),
            InstructionType::Concat => self.eval_concat(var, inst),
            _ => unreachable!(),
        }
    }

    fn eval_func<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        reference: &'a RuntimeRef,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        match &inst.data.as_ref().unwrap()[..] {
            "title" | "title:contains" | "title:statswith" | "title:endswith" => {
                self.eval_func_title(reference, inst)
            }
            "body:contains" | "body:menu_exists" => {
                if reference.articles.is_none() {
                    panic!("you must load dump if you want to use body related func!")
                }

                self.eval_func_body(reference, inst)
            }
            "count" => self.eval_func_count(var, inst),
            "set" => self.eval_func_set(var, inst),
            "group_sum" => self.eval_func_group_sum(var, inst),
            "map" => self.eval_func_map(var, reference, inst),
            "flatten" => self.eval_func_flatten(var, reference, inst),
            _ => unreachable!(),
        }
    }

    fn eval_intercross<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let mut intersection_count: HashMap<&RuntimeVariableAbstractData, usize> = HashMap::new();

        inst.params.as_ref().unwrap().iter().for_each(|x| {
            let array = &var[&x.id].data.unwrap_array();

            array.iter().for_each(|x| {
                *intersection_count.entry(x).or_default() += 1;
            });
        });

        let max_count = inst.params.as_ref().unwrap().len();

        let result: Vec<RuntimeVariableAbstractData> = intersection_count
            .iter()
            .filter(|x| *x.1 == max_count)
            .map(|x| (*x.0).clone())
            .collect();

        Ok(RuntimeVariable {
            inst,
            data: RuntimeVariableAbstractData::Array(Box::new(result)),
        })
    }

    fn eval_concat<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let mut result: Vec<RuntimeVariableAbstractData> = Vec::new();

        inst.params.as_ref().unwrap().iter().for_each(|x| {
            let array = &var[&x.id].data.unwrap_array();

            array.as_ref().iter().for_each(|x| {
                result.push(x.clone());
            });
        });

        Ok(RuntimeVariable {
            inst: inst,
            data: RuntimeVariableAbstractData::Array(Box::new(result)),
        })
    }

    fn eval_func_title<'a>(
        &self,
        reference: &RuntimeRef,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let what = inst.params.as_ref().unwrap()[0].data.as_ref().unwrap();

        match &inst.data.as_ref().unwrap()[..] {
            // exact match
            "title" => {
                let title = reference.title_index.find_one_by(what);

                if let Some(exact_title) = title {
                    let article = reference.title_index.get_no_redirect(exact_title);

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
                let titles = reference.title_index.find_by(
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
                            let article = reference.title_index.get_no_redirect(t).unwrap();

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

    fn eval_func_body<'a>(
        &self,
        reference: &RuntimeRef,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let what = inst.params.as_ref().unwrap()[0].data.as_ref().unwrap();

        match &inst.data.as_ref().unwrap()[..] {
            _ => unreachable!(),
        }
    }

    fn eval_func_count<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let var = var.get(&inst.params.as_ref().unwrap()[0].id).unwrap();

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

    // transform array to set
    fn eval_func_set<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let mut set: HashSet<&RuntimeVariableAbstractData> = HashSet::new();
        let mut result: Vec<RuntimeVariableAbstractData> = Vec::new();

        let func_id = &inst.params.as_ref().unwrap()[0].id;
        let array = &var[func_id].data.unwrap_array();

        array.iter().for_each(|x| {
            if !set.contains(x) {
                set.insert(x);
                // todo: this code maybe mem copy, mem allocation overhead
                // we must consume target runtime variable, if that variable
                // unused anymore
                result.push(x.clone());
            }
        });

        Ok(RuntimeVariable {
            inst,
            data: RuntimeVariableAbstractData::Set(Box::new(result)),
        })
    }

    fn eval_func_group_sum<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let mut group_map_index: HashMap<&RuntimeVariableAbstractData, usize> = HashMap::new();
        let mut group_map_count: HashMap<usize, usize> = HashMap::new();

        let func_id = &inst.params.as_ref().unwrap()[0].id;
        let array = &var[func_id].data.unwrap_array();

        for (i, e) in array.iter().enumerate() {
            let index = group_map_index.entry(e).or_insert(i);
            *group_map_count.entry(*index).or_default() += 1;
        }

        let mut result: Vec<RuntimeVariableAbstractData> = Vec::new();

        for (i, e) in array.iter().enumerate() {
            if group_map_index[e] == i {
                let tuple_left = e.clone();
                let tuple_right = RuntimeVariableAbstractData::Primitive(
                    RuntimeVariableAbstractPrimitiveData::Integer(group_map_count[&i] as i64),
                );

                result.push(RuntimeVariableAbstractData::Tuple(vec![
                    tuple_left,
                    tuple_right,
                ]));
            }
        }

        result.sort_by(|x, y| {
            let xx = x.unwrap_tuple()[1].unwrap_primitive().unwrap_integer();
            let yy = y.unwrap_tuple()[1].unwrap_primitive().unwrap_integer();

            yy.cmp(&xx)
        });

        Ok(RuntimeVariable {
            inst,
            data: RuntimeVariableAbstractData::Array(Box::new(result)),
        })
    }

    fn eval_func_map<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        reference: &'a RuntimeRef,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        match &inst.params.as_ref().unwrap()[1]
            .as_ref()
            .data
            .as_ref()
            .unwrap()[..]
        {
            "category" => {
                let p1 = &var[&inst.params.as_ref().unwrap()[0].as_ref().id];
                let array = &p1.data.unwrap_array();

                let result: Vec<_> = array
                    .as_ref()
                    .iter()
                    .filter_map(|x| {
                        let article = x.primitive()?.article()?;
                        let category_name = &reference.category_index.get(&article.title)?;

                        Some(RuntimeVariableAbstractData::Array(Box::new(
                            category_name
                                .into_iter()
                                .map(|x| {
                                    RuntimeVariableAbstractData::Primitive(
                                        RuntimeVariableAbstractPrimitiveData::Category(x),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        )))
                    })
                    .collect();

                Ok(RuntimeVariable {
                    inst,
                    data: RuntimeVariableAbstractData::Array(Box::new(result)),
                })
            }
            "select_min_len" | "select_max_len" => todo!(),
            "redirect" => {
                let p1 = &var[&inst.params.as_ref().unwrap()[0].as_ref().id];
                let array = &p1.data.unwrap_array();

                let result: Vec<_> = array
                    .as_ref()
                    .iter()
                    .filter_map(|x| {
                        let article = x.primitive()?.article()?;
                        let article = if article.is_redirect() {
                            reference
                                .title_index
                                .get(&article.title)
                                .unwrap_or(article.clone())
                        } else {
                            article.clone()
                        };

                        Some(RuntimeVariableAbstractData::Primitive(
                            RuntimeVariableAbstractPrimitiveData::Article(article),
                        ))
                    })
                    .collect();

                Ok(RuntimeVariable {
                    inst,
                    data: RuntimeVariableAbstractData::Array(Box::new(result)),
                })
            }
            "sort" => todo!(),
            _ => unreachable!(),
        }
    }

    fn eval_func_flatten<'a>(
        &self,
        var: &HashMap<usize, RuntimeVariable<'a>>,
        reference: &'a RuntimeRef,
        inst: &'a Instruction,
    ) -> Result<RuntimeVariable<'a>, Box<dyn Error>> {
        let var = var.get(&inst.params.as_ref().unwrap()[0].id).unwrap();

        let result: Vec<RuntimeVariableAbstractData> = match &var.data {
            RuntimeVariableAbstractData::Array(e) => (*e)
                .iter()
                .map(|x| x.unwrap_array().iter().map(|y| y.clone()))
                .flatten()
                .collect(),
            _ => unreachable!(),
        };

        Ok(RuntimeVariable {
            inst,
            data: RuntimeVariableAbstractData::Array(Box::new(result)),
        })
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

    macro_rules! vm_test {
        ($target:expr, $uncover:expr, $expected:expr) => {
            let irb = IRBuilder::from($target).unwrap();

            let head_inst = irb.build();
            let insts = IRBuilder::ir_flatten(&head_inst);

            let vm = vm_from!(insts);

            let tindex = TitleIndex::load(DEFAULT_DUMP_PATH, DEFAULT_TITLE_INDEX_PATH).unwrap();
            let cindex = CategoryIndex::load(DEFAULT_CATEGORY_INDEX_PATH).unwrap();

            let rt_ref = RuntimeRef {
                category_index: &cindex,
                title_index: &tindex,
                articles: None,
            };

            let result = vm.run(&rt_ref).unwrap();

            assert_eq!($uncover(result), $expected);
        };
    }

    #[test]
    fn vm_title_load_test() {
        vm_test!(
            "count(title:contains(\"동방\"))",
            |x| _uncover_integer(&x),
            851
        );
    }

    #[test]
    fn vm_array_to_set_test() {
        vm_test!(
            "set(title:contains(\"동방\"))",
            |x: RuntimeVariable| match x.data {
                RuntimeVariableAbstractData::Set(_) => true,
                _ => false,
            },
            true
        );
    }

    #[test]
    fn vm_group_sum_test() {
        vm_test!(
            "group_sum(flatten(map(map(title:contains(\"동방\"), redirect), category)))",
            |x: RuntimeVariable| match x.data {
                RuntimeVariableAbstractData::Array(_) => true,
                _ => false,
            },
            true
        );
    }

    #[test]
    fn vm_intercross_test() {
        vm_test!(
            "count(title:contains(\"동방\") & title:contains(\"프로젝트\") )",
            |x| _uncover_integer(&x),
            213
        );
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

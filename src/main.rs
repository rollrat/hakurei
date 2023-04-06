pub mod core;
pub mod index;
pub mod loader;
pub mod model;

use std::{env, process::exit};

use crate::{
    core::{
        ir::IRBuilder,
        vm::{RuntimeRef, VirtualMachine},
    },
    index::{category::CategoryIndex, title::TitleIndex},
};

const DEFAULT_DUMP_PATH: &str = "namuwiki_20210301.json";
const DEFAULT_TITLE_INDEX_PATH: &str = "title-index.json";
const DEFAULT_CATEGORY_INDEX_PATH: &str = "article-with-categories.json";

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() == 1 {
        println!("you must write one input ");
        exit(0);
    }

    let irb = IRBuilder::from(&args[1][..]).unwrap();

    let head_inst = irb.build();
    let insts = IRBuilder::ir_flatten(&head_inst);

    let vm = VirtualMachine::from(insts);

    let tindex = TitleIndex::load(DEFAULT_DUMP_PATH, DEFAULT_TITLE_INDEX_PATH).unwrap();
    let cindex = CategoryIndex::load(DEFAULT_CATEGORY_INDEX_PATH).unwrap();

    let rt_ref = RuntimeRef {
        category_index: &cindex,
        title_index: &tindex,
    };

    let result = vm.run(&rt_ref).unwrap();

    println!("{:#?}", result);
}

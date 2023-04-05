pub mod core;
pub mod index;
pub mod loader;
pub mod model;

use crate::index::{category::CategoryIndex, title::TitleIndex};

const DEFAULT_DUMP_PATH: &str = "namuwiki_20210301.json";
const DEFAULT_TITLE_INDEX_PATH: &str = "title-index.json";
const DEFAULT_CATEGORY_INDEX_PATH: &str = "article-with-categories.json";

fn main() {
    let mut tindex = TitleIndex::load(DEFAULT_DUMP_PATH, DEFAULT_TITLE_INDEX_PATH).unwrap();
    let cindex = CategoryIndex::load(DEFAULT_CATEGORY_INDEX_PATH).unwrap();

    println!("{}", tindex.get("동방지령전").unwrap().text);
    println!(
        "{:#?}",
        cindex
            .get(&tindex.get("동방지령전").unwrap().title)
            .unwrap()
    );
}

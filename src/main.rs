pub mod index;
pub mod loader;
pub mod model;

use crate::index::{category::CategoryIndex, title::TitleIndex};

fn main() {
    let mut tindex = TitleIndex::load().unwrap();
    let cindex = CategoryIndex::load().unwrap();

    println!("{}", tindex.get("동방지령전").unwrap().text);
    println!(
        "{:#?}",
        cindex
            .get(&tindex.get("동방지령전").unwrap().title)
            .unwrap()
    );
}

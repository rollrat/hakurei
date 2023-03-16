use std::{collections::HashMap, error::Error, fs};

use crate::model::article_category::ArticleCategory;

pub struct CategoryIndex {
    map: HashMap<String, Vec<String>>,
}

impl CategoryIndex {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let mut raw = fs::read_to_string("article-with-categories.json")?;

        unsafe {
            let js: Vec<ArticleCategory> = simd_json::from_str(&mut raw)?;

            Ok(CategoryIndex {
                map: js
                    .into_iter()
                    .map(|x| {
                        (
                            x.title.to_owned(),
                            x.categories.into_iter().map(|x| x.to_owned()).collect(),
                        )
                    })
                    .collect(),
            })
        }
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        self.map.get(key)
    }
}

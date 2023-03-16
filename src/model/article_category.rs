use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleCategory<'a> {
    pub title: &'a str,
    pub categories: Vec<&'a str>,
}

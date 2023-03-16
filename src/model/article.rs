use regex::Regex;
use serde::{Deserialize, Serialize};

use super::article_category::ArticleCategory;

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub text: String,
}

impl Article {
    pub fn categories(&self) -> Vec<&str> {
        let mut result = Vec::new();
        let re = Regex::new(r"\[\[분류:(.*?)\]\]").unwrap();

        for cap in re.captures_iter(&self.text) {
            result.push(cap.get(1).map_or("", |m| m.as_str()));
        }

        result
    }

    pub fn to_article_category(&self) -> ArticleCategory {
        ArticleCategory {
            title: &self.title,
            categories: self.categories(),
        }
    }

    pub fn is_redirect(&self) -> bool {
        self.text.starts_with("#redirect")
    }

    pub fn get_redirect(&self) -> &str {
        &self.text[10..].trim()
    }
}

use std::{error::Error, fs};

use crate::model::article::Article;

pub fn load_dump(path: &str) -> Result<Vec<Article>, Box<dyn Error>> {
    let raw = fs::read_to_string(path)?;

    //
    // https://stackoverflow.com/questions/57423880/handling-serde-error-and-other-error-type
    serde_json::from_str(&raw).map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
}

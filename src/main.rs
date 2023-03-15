use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Seek, SeekFrom},
};

use regex::Regex;

#[allow(dead_code)]
fn split_json_file() {
    let fs = fs::read_to_string("namuwiki_202103012.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&fs).unwrap();
    let arr = json.as_array().unwrap();

    let length = arr.len();

    let mut r1: Vec<&serde_json::Value> = Vec::new();
    let mut r2: Vec<&serde_json::Value> = Vec::new();
    let mut r3: Vec<&serde_json::Value> = Vec::new();

    for i in 0..length {
        match i % 3 {
            0 => &mut r1,
            1 => &mut r2,
            2 => &mut r3,
            _ => panic!(),
        }
        .push(&arr[i]);
    }

    fs::write("0.json", &serde_json::to_string(&r1).unwrap()).unwrap();
    fs::write("1.json", &serde_json::to_string(&r2).unwrap()).unwrap();
    fs::write("2.json", &serde_json::to_string(&r3).unwrap()).unwrap();
}

#[allow(dead_code)]
fn save_categories_rank() {
    let js = load_dump();
    let mut hmap = HashMap::new();

    for x in &js {
        let category = x.categories();

        for c in category {
            *hmap.entry(c).or_insert(0) += 1;
        }
    }

    let mut v = hmap.into_iter().collect::<Vec<(&str, i32)>>();
    v.sort_by(|a, b| a.1.cmp(&b.1));

    fs::write("r.json", format!("{:#?}", v)).unwrap();
}

#[allow(dead_code)]
fn find_by_category_test(what: &str) {
    let js = load_dump();

    let mut r: Vec<(&str, Vec<&str>)> = Vec::new();

    for x in &js {
        let category = x.categories();

        if category.iter().any(|x| x.starts_with(what)) {
            r.push((&x.title, category));
        }
    }

    r.sort_by(|a, b| a.0.cmp(b.0));

    fs::write("t.json", format!("{:#?}", r)).unwrap();
}

#[allow(dead_code)]
fn find_by_category<'a>(js: &'a Vec<Article>, what: &str) -> Vec<&'a str> {
    let mut result: Vec<&str> = Vec::new();

    for x in js {
        let category = x.categories();

        if category.iter().any(|x| x.starts_with(what)) {
            result.push(&x.title);
        }
    }

    result.sort_by(|a, b| a.cmp(b));

    result
}

#[allow(dead_code)]
fn group_by_category_test() {
    let js = load_dump();

    let mut hmap: HashMap<&str, Vec<&str>> = HashMap::new();

    for x in &js {
        let category = x.categories();

        if category.iter().any(|x| x.starts_with("일본 애니메이션/")) {
            category.iter().for_each(|y| {
                hmap.entry(y).or_default().push(&x.title);
            });
        }
    }

    let mut result = hmap.iter().collect::<Vec<(&&str, &Vec<&str>)>>();

    result.sort_by(|a, b| a.0.cmp(b.0));

    for category in &mut result {
        println!("{}", category.0);

        let mut animations = category.1.clone();
        animations.sort();
        for animation in animations {
            println!("{}", animation);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Article {
    pub title: String,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ArticleCategory<'a> {
    pub title: &'a str,
    pub categories: Vec<&'a str>,
}

impl Article {
    fn categories(&self) -> Vec<&str> {
        let mut result = Vec::new();
        let re = Regex::new(r"\[\[분류:(.*?)\]\]").unwrap();

        for cap in re.captures_iter(&self.text) {
            result.push(cap.get(1).map_or("", |m| m.as_str()));
        }

        result
    }

    fn to_article_category(&self) -> ArticleCategory {
        ArticleCategory {
            title: &self.title,
            categories: self.categories(),
        }
    }

    fn is_redirect(&self) -> bool {
        self.text.starts_with("#redirect")
    }

    fn get_redirect(&self) -> &str {
        &self.text[10..].trim()
    }
}

#[allow(dead_code)]
fn extract_category(js: &Vec<Article>) {
    let article_categories = js
        .iter()
        .map(|x| x.to_article_category())
        .collect::<Vec<ArticleCategory>>();

    let json_result = serde_json::to_string_pretty(&article_categories).unwrap();
    fs::write("article-with-categories.json", json_result).unwrap();
}

#[allow(dead_code)]
fn create_title_index() {
    let fs = fs::read_to_string("namuwiki_202103012.json").unwrap();
    let re = Regex::new("\"title\":\"(.*?)\"").unwrap();

    let mut result: HashMap<&str, Vec<usize>> = HashMap::new();

    let mut last_key = "";

    let front_len = "{\"namespace\":.,\"title\":\"".len();

    for cap in re.captures_iter(&fs) {
        let title_range = cap.get(1).unwrap().range();
        let current_item_start = title_range.start - front_len;

        if last_key != "" {
            result
                .get_mut(last_key)
                .map(|val| val.push(current_item_start - 2));
        }

        last_key = cap.get(1).unwrap().as_str();

        result.insert(cap.get(1).unwrap().as_str(), vec![current_item_start]);
    }

    result.get_mut(last_key).map(|val| val.push(fs.len() - 1));

    let json_result = serde_json::to_string_pretty(&result).unwrap();
    fs::write("title-index.json", json_result).unwrap();
}

struct TitleIndex {
    map: HashMap<String, Vec<usize>>,
    file: fs::File,
}

impl TitleIndex {
    fn load() -> Self {
        let mut raw = fs::read_to_string("title-index.json").unwrap();

        unsafe {
            TitleIndex {
                map: simd_json::from_str(&mut raw).unwrap(),
                file: fs::File::open("namuwiki_202103012.json").unwrap(),
            }
        }
    }

    fn get_no_redirect(&mut self, key: &str) -> Option<Article> {
        if !self.map.contains_key(key) {
            return None;
        }

        let index = &self.map[key];

        let offset_start = index[0];
        let offset_end = index[1];

        self.file
            .seek(SeekFrom::Start(offset_start as u64))
            .unwrap();

        let mut buf = vec![0; offset_end - offset_start + 1];
        self.file.read(&mut buf).unwrap();

        let raw = String::from_utf8(buf).unwrap();

        Some(serde_json::from_str(&raw).unwrap())
    }

    fn get(&mut self, key: &str) -> Option<Article> {
        if !self.map.contains_key(key) {
            return None;
        }

        match self.get_no_redirect(key) {
            Some(x) => {
                if x.is_redirect() {
                    self.get(x.get_redirect())
                } else {
                    Some(x)
                }
            }
            None => None,
        }
    }
}

fn load_dump() -> Vec<Article> {
    let raw = fs::read_to_string("namuwiki_202103012.json").unwrap();

    serde_json::from_str(&raw).unwrap()
}

fn main() {
    let mut index = TitleIndex::load();

    println!("{}", index.get("동방지령전").unwrap().text);
}

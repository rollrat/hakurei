use std::{collections::HashMap, fs};

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
fn save_classes_rank() {
    let js = load_dump();

    let mut hmap = HashMap::new();

    for x in &js {
        let class = x.classes();

        for c in class {
            *hmap.entry(c).or_insert(0) += 1;
        }
    }

    let mut v = hmap.into_iter().collect::<Vec<(&str, i32)>>();
    v.sort_by(|a, b| a.1.cmp(&b.1));

    fs::write("r.json", format!("{:#?}", v)).unwrap();
}

#[derive(Debug)]
struct Article {
    pub title: String,
    text: String,
}

impl Article {
    fn from(value: &serde_json::Value) -> Self {
        Article {
            // todo optimizing
            // to_owned may be copying processing
            title: value["title"].as_str().unwrap().to_owned(),
            text: value["text"].as_str().unwrap().to_owned(),
        }
    }

    fn classes(&self) -> Vec<&str> {
        let mut result = Vec::new();
        let re = Regex::new(r"\[\[분류:(.*?)\]\]").unwrap();

        for cap in re.captures_iter(&self.text) {
            result.push(cap.get(1).map_or("", |m| m.as_str()));
        }

        result
    }
}

fn load_dump<'a>() -> Vec<Article> {
    let raw = fs::read_to_string("namuwiki_202103012.json").unwrap();
    let s: serde_json::Value = serde_json::from_str(&raw).unwrap();

    s.as_array()
        .unwrap()
        .iter()
        .map(|x| Article::from(x))
        .collect::<Vec<Article>>()
}

fn main() {
    let js = load_dump();

    let mut r: Vec<(&str, Vec<&str>)> = Vec::new();

    for x in &js {
        let class = x.classes();

        // if x["title"].as_str().unwrap().contains(&"제로부터 시작하는") {
        //     println!("{:#?}", x);
        // }

        // if class.contains(&"일본 애니메이션/목록") {
        //     println!("{}", x["title"].as_str().unwrap());
        // }

        if class.iter().any(|x| x.starts_with("일본 애니메이션/")) {
            // println!("{}", x["title"].as_str().unwrap());
            r.push((&x.title, class));
        }
    }

    r.sort_by(|a, b| a.0.cmp(b.0));

    fs::write("t.json", format!("{:#?}", r)).unwrap();
}

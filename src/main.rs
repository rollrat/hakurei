use std::fs;

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

fn get_classes(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let re = Regex::new(r"\[\[분류:(.*?)\]\]").unwrap();

    for cap in re.captures_iter(text) {
        result.push(cap.get(1).map_or("", |m| m.as_str()));
    }

    result
}

fn main() {
    let fs = fs::read_to_string("namuwiki_202103012.json").unwrap();
    let js: serde_json::Value = serde_json::from_str(&fs).unwrap();

    for x in js.as_array().unwrap() {
        if x["title"].as_str().unwrap().starts_with(&"환상향") {
            println!("{:#?}", get_classes(x["text"].as_str().unwrap()));
        }
    }
}

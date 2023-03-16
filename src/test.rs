#[cfg(test)]
mod tests {
    use loader::load::load_dump;
    use model::{article::Article, article_category::ArticleCategory};
    use std::{collections::HashMap, fs};

    use regex::Regex;

    use crate::{loader, model};

    const DUMP_PATH: &str = "namuwiki_202103012.json";

    #[test]
    fn split_json_file() {
        let fs = fs::read_to_string(DUMP_PATH).unwrap();
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

    #[test]
    fn save_categories_rank() {
        let js = load_dump(DUMP_PATH).unwrap();
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

    #[test]
    fn find_by_category_test() {
        let what = "하쿠레이 신사";
        let js = load_dump(DUMP_PATH).unwrap();

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

    #[test]
    fn find_by_category<'a>() {
        let js = load_dump(DUMP_PATH).unwrap();
        let what = "일본 애니메이션/";
        let mut result: Vec<String> = Vec::new();

        for x in js {
            let category = x.categories();

            if category.iter().any(|x| x.starts_with(what)) {
                result.push(x.title);
            }
        }

        result.sort_by(|a, b| a.cmp(b));
    }

    #[test]
    fn group_by_category_test() {
        let js = load_dump(DUMP_PATH).unwrap();

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

    #[test]
    fn extract_category() {
        let js = load_dump(DUMP_PATH).unwrap();
        let article_categories = js
            .iter()
            .map(|x| x.to_article_category())
            .collect::<Vec<ArticleCategory>>();

        let json_result = serde_json::to_string_pretty(&article_categories).unwrap();
        fs::write("article-with-categories.json", json_result).unwrap();
    }

    #[test]
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
}

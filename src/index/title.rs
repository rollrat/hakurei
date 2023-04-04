use std::{
    cmp::Ordering,
    collections::HashMap,
    error::Error,
    fs,
    io::{Read, Seek, SeekFrom},
};

use crate::model::article::Article;

pub struct TitleIndex {
    map: HashMap<String, Vec<usize>>,
    file: fs::File,
}

pub enum TitleIndexFindOption {
    Extact,
    Contains,
    StartsWith,
    EndsWith,
}

impl TitleIndex {
    pub fn load(dump_path: &str, index_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut raw = fs::read_to_string(index_path)?;

        unsafe {
            Ok(TitleIndex {
                map: simd_json::from_str(&mut raw)?,
                file: fs::File::open(dump_path)?,
            })
        }
    }

    pub fn get_no_redirect(&mut self, key: &str) -> Option<Article> {
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

    pub fn get(&mut self, key: &str) -> Option<Article> {
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

    pub fn find_by(&self, what: &str, option: TitleIndexFindOption) -> Vec<&String> {
        self.map
            .keys()
            .filter(|x| match option {
                TitleIndexFindOption::Extact => what.cmp(x) == Ordering::Equal,
                TitleIndexFindOption::Contains => x.contains(what),
                TitleIndexFindOption::StartsWith => x.starts_with(what),
                TitleIndexFindOption::EndsWith => x.ends_with(what),
            })
            .collect()
    }

    pub fn find_one_by(&self, what: &str) -> Option<&String> {
        self.map.keys().find(|x| what.cmp(x) == Ordering::Equal)
    }
}

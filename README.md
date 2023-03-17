# hakurei

Namuwiki Stream Pipeline Filter

## Namuwiki Syncronizer & Stream

## Namuwiki Search Engine

- Search

  - [ ] Title Search
  - [ ] Title Fuzzy Search
  - [ ] Body Search
  - [ ] Body Fuzzy Search (ELK Like)
  - [ ] Category Search

- Extract

  - [ ] Extract Outlink
  - [ ] Extract Category

- Stream Pipeline

  - [ ] Map
  - [ ] Filter
  - [ ] Union
  - [ ] Substract

- Etc

  - [ ] Wordcloud

### Middle-end Data Type

- Abstract

  - Article `(Title, [Category], Text, [Contributor])`
  - Category `(Name, [Article])`

- Implementaion

  - ArticleSet `({&Article})`
  - ArticleArray `([&Article])`
  - ArticleWithCountArray `([(&Article, Count)])`
  - CategorySet `({&Category})`
  - CategoryArray `([&Category])`
  - CategoryWithCountArray `([(&Category, Count)])`

### Description

```
command -> expr

expr -> func
      | expr and expr
      | expr or  expr
      | e

func -> name ( )
      | name ( args )

name -> function_name
      | function_name:sub_name

args -> const
      | expr
      | expr, args

number -> [0-9]+
string -> "([^\\"]|\\")*"
const  -> number
        | string
```

## Namuwiki Parser

At least 32gb of memory is required to use this feature.

Dump download: https://mu-star.net/wikidb

### Extract Category

Most articles have categories.

```rs
fn main() {
  let js = load_dump();
  extract_category(js);
}
```

#### (Example) Find By Category

If you want to fetch `Article` rather than `Title`, modify the code appropriately.

```rs
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
```

### Title indexing

Before using, you must create title index file.

```rs
fn main() {
  create_title_index();
}
```

```rs
fn main() {
  let mut index = TitleIndex::load();

  println!("{}", index.get("하쿠레이 신사").unwrap().text);
}
```

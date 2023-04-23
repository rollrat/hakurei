# hakurei

Namuwiki Stream Pipeline Filter

## Namuwiki Syncronizer & Stream

## Namuwiki Search Engine

- Search

  - [x] Title Search
  - [ ] Title Fuzzy Search
  - [ ] Body Search
  - [ ] Body Fuzzy Search (ELK Like)
  - [ ] Category Search

- Extract

  - [ ] Extract Outlink
  - [ ] Extract Category

- Stream Pipeline

  - [x] Map
  - [x] Filter
  - [x] Union
  - [x] Substract

- Etc

  - [ ] Wordcloud

### Description

```
command       -> expr_and

expr_and      -> expr_or (& expr_or)*
expr_or       -> expr_case (| expr_case)*
expr_case     -> ( expr_and )
               | func

func          -> name
               | name ( args )

name          -> function_name
               | function_name:sub_name

args          -> const(, args)*
               | expr_and(, args)*

function_name -> [_a-zA-Z$][_:a-zA-Z0-9$]*
number        -> [0-9]+
string        -> "([^\\"]|\\")*"
const         -> number
               | string
```

#### Examples

```js
1. group_sum(reduce(title:contains("동방"), category))
title:contains("동방") => ArticleArray
reduce(_, category)   => CategoryArray
group_sum(_)          => CategoryWithCountArray

2. count(set(reduce(title:contains("동방"), category)))
set(_)   => CategorySet
count(_) => usize

3. map(reduce(title:startswith("서든") | title:endswith("어택"), category), select_max_len)
map(_, select_max_len) => CategoryArray
```

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

#### Type Inference Example

```js
1. group_sum(reduce(title:startswith("서든") & title:endswith("어택"), category))
v2 = title:startswith("서든")     # Array(Primitive(Article))
v4 = title:endswith("어택")       # Array(Primitive(Article))
v5 = &(v2, v4)                   # Array(Primitive(Article))
v7 = reduce(v5, ref("category")) # Array(Primitive(Category))
v8 = group_sum(v7)               # Array(Tuple([Primitive(Category), Primitive(Integer)]))

2. set(reduce(title:contains("동방"), category))
v2 = title:contains("동방")       # Array(Primitive(Article))
v4 = reduce(v2, ref("category")) # Array(Primitive(Category))
v5 = set(v4)                     # Set(Primitive(Category))

3. count(set(reduce(title:contains("동방"), category)))
v2 = title:contains("동방")       # Array(Primitive(Article))
v4 = reduce(v2, ref("category")) # Array(Primitive(Category))
v5 = set(v4)                     # Set(Primitive(Category))
v6 = count(v5)                   # Primitive(Integer)

4. set(reduce(title:contains("다크소울"), category) &
       reduce(title:contains("엘든링"), category)) &
   set(reduce(title:contains("붕괴") | title:contains("원신"), category))
v2 = title:contains("다크소울")      # Array(Primitive(Article))
v6 = title:contains("엘든링")       # Array(Primitive(Article))
v12 = title:contains("붕괴")        # Array(Primitive(Article))
v14 = title:contains("원신")        # Array(Primitive(Article))
v4 = reduce(v2, ref("category"))   # Array(Primitive(Category))
v8 = reduce(v6, ref("category"))   # Array(Primitive(Category))
v15 = |(v12, v14)                  # Array(Primitive(Article))
v9 = &(v4, v8)                     # Array(Primitive(Category))
v17 = reduce(v15, ref("category")) # Array(Primitive(Category))
v10 = set(v9)                      # Set(Primitive(Category))
v18 = set(v17)                     # Set(Primitive(Category))
v19 = &(v10, v18)                  # Set(Primitive(Category))
```

### Optimizing

- [ ] Variable Consume
- [ ] Constant Folding
- [ ] Common Expression

### Available Functions

```rs
title:*(<String>) => [Article]
body:*(<String>) => [Article]
count(<Array<T> | Set<T>>) => Integer
set(<Array<T>>) => Set<T>
array(<Set<T>>) => Array<T> // not yet
group_sum(<Array<T>>) where T: Article | Category => Array<(T, Integer)>
map(<Array<T>>, (T) => F) => Array<F> // not yet
flatten(<Array<T>>) => Array<F> // not yet
filter(<Array<T>>, (T) => bool) => Array<T> // not yet
sort(<Array<T>>, (T, T) => i32) => Array<T> // not yet
bind((T) => F, (F) => G, *, (H) => K) => K // not yet
use_funcs()
   -> category := (<Article>) => Array<Category>
   -> select_max_len := (<Array<T> | Set<T>>) => T // not yet
   -> select_min_len := (<Array<T> | Set<T>>) => T // not yet
   -> redirect := (<Article>) => Article // not yet
   -> unwrap_tuple1 := ((F, *)) => F, // not yet
   -> unwrap_tuple2 := ((F, H, *)) => H, // not yet
   -> cmp_array := (T, T) => i32 // not yet
   -> cmp_tuple1 := ((F, *), (F, *)) where T: (F, *) => i32 // not yet
   -> cmp_tuple2 := ((F, *), (F, *)) where T: (F, *) => i32 // not yet
```

```rs
title
title:exact
title:contains
title:startswith
title:endswith
body:contains
body:menu_exists
body:regex
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

//! The xmark.toml file
use serde::{Deserialize, Serialize};
use toml::{self, value::Table};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiskConf {
    pub books: Vec<Location>,
    #[serde(default)]
    pub output: Table,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Location {
    Bare(String),
    Named { name: String, root_loc: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::Value;

    fn check_disc_conf(l: DiskConf, r: &str) {
        assert_eq!(toml::from_str::<DiskConf>(r).unwrap(), l);
    }

    fn l(s: &str) -> Location {
        Location::Bare(s.to_owned())
    }

    #[test]
    fn basic_disc_conf() {
        check_disc_conf(
            DiskConf {
                books: vec![l("hello"), l("w2"), l("x4")],
                output: Table::new(),
            },
            r#"
books = [
    "hello",
    "w2",
    "x4",
]
            "#,
        );
    }

    #[test]
    fn empty_output_html() {
        check_disc_conf(
            DiskConf {
                books: vec![l("just_one")],
                output: vec![("html".to_owned(), Value::Table(Table::new()))]
                    .into_iter()
                    .collect(),
            },
            r#"
books = [ "just_one" ]
[output.html]
            "#,
        )
    }

    #[test]
    fn output_html_vals() {
        let mut html = Table::new();
        html.insert(
            "output-url".to_owned(),
            Value::String("/bookshelf/".to_owned()),
        );
        html.insert("minify".to_owned(), Value::Boolean(false));
        let html = Value::Table(html);

        check_disc_conf(
            DiskConf {
                books: vec![l("the-book")],
                output: vec![("html".to_owned(), html)].into_iter().collect(),
            },
            r#"
books = [ "the-book" ]
[output.html]
output-url = "/bookshelf/"
minify = false
            "#,
        )
    }

    #[test]
    fn multi_output() {
        let mut html = Table::new();
        html.insert(
            "output-url".to_owned(),
            Value::String("/bookshelf/".to_owned()),
        );
        html.insert("minify".to_owned(), Value::Boolean(false));
        let html = Value::Table(html);

        let mut tex = Table::new();
        tex.insert("created".to_owned(), Value::Integer(1978));
        tex.insert("xml".to_owned(), Value::Boolean(false));
        tex.insert("parsing".to_owned(), Value::String("Imposible".to_owned()));
        let tex = Value::Table(tex);

        check_disc_conf(
            DiskConf {
                books: Vec::new(),
                output: vec![("html".to_owned(), html), ("tex".to_owned(), tex)]
                    .into_iter()
                    .collect(),
            },
            r#"
books = []
[output.tex]
created = 1978
xml = false
parsing = "Imposible"
# Comment
[output.html]
output-url = "/bookshelf/"
minify = false
        "#,
        )
    }

    #[test]
    fn with_loc() {
        check_disc_conf(
            DiskConf {
                books: vec![
                    l("bare"),
                    Location::Named {
                        name: "trpl".to_owned(),
                        root_loc: "book/src/".to_owned(),
                    },
                    l("another-bare"),
                    Location::Named {
                        name: "cxx".to_owned(),
                        root_loc: "cxx/guide/src".to_owned(),
                    },
                    l("xmark"),
                    l("dummy"),
                ],
                output: Table::new(),
            },
            r#"
books = [
    "bare",
    { location = "book/src/", name = "trpl" },
    "another-bare",
    { name = "cxx", location="cxx/guide/src" },
    "xmark",
    "dummy",
]
[output]
        "#,
        )
    }
}

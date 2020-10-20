use serde::{Deserialize, Serialize};

/// The Config as represented in the global xmark.toml
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub books: Vec<String>,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_gloal_config() {
        let inp = "books = [
            'book-1',
            'book-2',
            'book-3',
            ]
";
        let conf: GlobalConfig = toml::from_str(inp).unwrap();
        assert_eq!(conf.books, vec!["book-1", "book-2", "book-3"]);
    }
}

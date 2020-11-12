use crate::config::Book;

// Types that the html renderer needs.

#[derive(Clone, Debug)]
pub struct Content<'a> {
    pub books: &'a [Book],
}

impl<'a> Content<'a> {
    pub fn new(books: &'a [Book]) -> Self {
        Self { books }
    }
}

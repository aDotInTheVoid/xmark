// SPDX-License-Identifier: GPL-3.0-only
//! The "minitoc" for the page
//!
//! This is inspired by the right hand table of contents from
//! [MkDocs Material](https://squidfunk.github.io/mkdocs-material/getting-started/)
//!
//! It's different from the left hand table for contents, which is for the whole book.
//! This is just for the the page we're on
//!
//! A page with this content
//! ```markdown
//! # Controll Flow
//!
//! ## If
//! ### If else
//! ### As an expression
//!
//! ## Match
//!
//! ## Loops
//! ### for .. in ..
//! ### while ...
//! ### loop
//! ```
//!
//! Will have this as a toc
//! ```rust,notest
//! # https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=14a9939a1d620cd67a9c8763a730a6df
//! PageToc(
//!    [
//!        H2 {
//!            this: Link {
//!                pritty: "If",
//!                link: "if",
//!            },
//!            children: [
//!                H3(
//!                    Link {
//!                        pritty: "If else",
//!                        link: "if-else",
//!                    },
//!                ),
//!                H3(
//!                    Link {
//!                        pritty: "As an expression",
//!                        link: "as-as-expression",
//!                    },
//!                ),
//!            ],
//!        },
//!        H2 {
//!            this: Link {
//!                pritty: "Match",
//!                link: "match",
//!            },
//!            children: [],
//!        },
//!        H2 {
//!            this: Link {
//!                pritty: "Loops",
//!                link: "loops",
//!            },
//!            children: [
//!                H3(
//!                    Link {
//!                        pritty: "for .. in ..",
//!                        link: "for-in",
//!                    },
//!                ),
//!                H3(
//!                    Link {
//!                        pritty: "while ...",
//!                        link: "while",
//!                    },
//!                ),
//!                H3(
//!                    Link {
//!                        pritty: "loop",
//!                        link: "loop",
//!                    },
//!                ),
//!            ],
//!        },
//!    ],
//!)
//!```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct PageToc(pub(crate) Vec<H2>);

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct Link {
    // The "nice" name, eg "Creating a book"
    pub(crate) pritty: String,
    // The name of the link, eg "creating-a-book"
    pub(crate) link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct H3(pub(crate) Link);

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct H2 {
    pub(crate) this: Link,
    pub(crate) children: Vec<H3>,
}

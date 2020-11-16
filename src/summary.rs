// SPDX-License-Identifier: GPL-3.0-only
use eyre::{bail, eyre, Error, Result, WrapErr};
use log::{debug, trace, warn};
use memchr::{self, Memchr};
use pulldown_cmark::{self, Event, Tag};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

/// Parse the text from a `SUMMARY.md` file into a sort of "recipe" to be
/// used when loading a book from disk.
///
/// # Summary Format
///
/// **Title:** It's common practice to begin with a title, generally
/// "# Summary". It's not mandatory and the parser (currently) ignores it, so
/// you can too if you feel like it.
///
/// **Prefix Chapter:** Before the main numbered chapters you can add a couple
/// of elements that will not be numbered. This is useful for forewords,
/// introductions, etc. There are however some constraints. You can not nest
/// prefix chapters, they should all be on the root level. And you can not add
/// prefix chapters once you have added numbered chapters.
///
/// ```markdown
/// [Title of prefix element](relative/path/to/markdown.md)
/// ```
///
/// **Part Title:** An optional title for the next collect of numbered chapters. The numbered
/// chapters can be broken into as many parts as desired.
///
/// **Numbered Chapter:** Numbered chapters are the main content of the book,
/// they
/// will be numbered and can be nested, resulting in a nice hierarchy (chapters,
/// sub-chapters, etc.)
///
/// ```markdown
/// # Title of Part
///
/// - [Title of the Chapter](relative/path/to/markdown.md)
/// ```
///
/// You can either use - or * to indicate a numbered chapter, the parser doesn't
/// care but you'll probably want to stay consistent.
///
/// **Suffix Chapter:** After the numbered chapters you can add a couple of
/// non-numbered chapters. They are the same as prefix chapters but come after
/// the numbered chapters instead of before.
///
/// All other elements are unsupported and will be ignored at best or result in
/// an error.
pub fn parse_summary(summary: &str) -> Result<Summary> {
    let parser = SummaryParser::new(summary);
    parser.parse()
}

/// The parsed `SUMMARY.md`, specifying how the book should be laid out.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Hash)]
pub struct Summary {
    /// An optional title for the `SUMMARY.md`, currently just ignored.
    pub title: String,
    /// Chapters before the main text (e.g. an introduction).
    pub prefix_chapters: Vec<Chapter>,
    /// The main numbered chapters of the book, broken into one or more possibly named parts.
    pub numbered_chapters: Vec<Link>,
    /// Items which come after the main document (e.g. a conclusion).
    pub suffix_chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Hash)]
pub struct Chapter {
    pub name: String,
    // None => draft
    pub location: Option<PathBuf>,
}

/// A struct representing an entry in the `SUMMARY.md`, possibly with nested
/// entries.
///
/// This is roughly the equivalent of `[Some section](./path/to/file.md)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Hash)]
pub struct Link {
    pub chapter: Chapter,
    pub nested_items: Vec<Link>,
    pub section_number: Option<SectionNumber>,
}

impl Link {
    pub fn map_mut(&mut self, f: impl Fn(&mut Chapter) + Copy) {
        f(&mut self.chapter);
        for i in &mut self.nested_items {
            i.map_mut(f);
        }
    }

    pub fn try_map<E>(&self, f: impl Fn(&Chapter) -> Result<(), E> + Copy) -> Result<(), E> {
        f(&self.chapter)?;
        for i in &self.nested_items {
            i.try_map(f)?;
        }
        Ok(())
    }
}

/// A recursive descent (-ish) parser for a `SUMMARY.md`.
///
///
/// # Grammar
///
/// The `SUMMARY.md` file has a grammar which looks something like this:
///
/// ```text
/// summary           ::= title prefix_chapters numbered_chapters
///                         suffix_chapters
/// title             ::= "# " TEXT
/// prefix_chapters   ::= chapter*
/// suffix_chapters   ::= chapter*
/// numbered_chapters ::= dotted_chapter+
/// dotted_chapter    ::= INDENT* DOT_POINT item
/// chapter           ::= "[" TEXT "]" "(" TEXT ")"
/// DOT_POINT         ::= "-"
///                     | "*"
/// ```
///
/// > **Note:** the `TEXT` terminal is "normal" text, and should (roughly)
/// > match the following regex: "[^<>\n[]]+".
pub struct SummaryParser<'a> {
    src: &'a str,
    stream: pulldown_cmark::OffsetIter<'a>,
    offset: usize,

    /// We can't actually put an event back into the `OffsetIter` stream, so instead we store it
    /// here until somebody calls `next_event` again.
    back: Option<Event<'a>>,
}

/// Reads `Events` from the provided stream until the corresponding
/// `Event::End` is encountered which matches the `$delimiter` pattern.
///
/// This is the equivalent of doing
/// `$stream.take_while(|e| e != $delimeter).collect()` but it allows you to
/// use pattern matching and you won't get errors because `take_while()`
/// moves `$stream` out of self.
macro_rules! collect_events {
    ($stream:expr,start $delimiter:pat) => {
        collect_events!($stream, Event::Start($delimiter))
    };
    ($stream:expr,end $delimiter:pat) => {
        collect_events!($stream, Event::End($delimiter))
    };
    ($stream:expr, $delimiter:pat) => {{
        let mut events = Vec::new();

        loop {
            let event = $stream.next().map(|(ev, _range)| ev);
            trace!("Next event: {:?}", event);

            match event {
                Some($delimiter) => break,
                Some(other) => events.push(other),
                None => {
                    debug!(
                        "Reached end of stream without finding the closing pattern, {}",
                        stringify!($delimiter)
                    );
                    break;
                }
            }
        }

        events
    }};
}

impl<'a> SummaryParser<'a> {
    fn new(text: &str) -> SummaryParser<'_> {
        let pulldown_parser = pulldown_cmark::Parser::new(text).into_offset_iter();

        SummaryParser {
            src: text,
            stream: pulldown_parser,
            offset: 0,
            back: None,
        }
    }

    /// Get the current line and column to give the user more useful error
    /// messages.
    fn current_location(&self) -> (usize, usize) {
        let previous_text = self.src[..self.offset].as_bytes();
        let line = Memchr::new(b'\n', previous_text).count() + 1;
        let start_of_line = memchr::memrchr(b'\n', previous_text).unwrap_or(0);
        let col = self.src[start_of_line..self.offset].chars().count();

        (line, col)
    }

    /// Parse the text the `SummaryParser` was created with.
    fn parse(mut self) -> Result<Summary> {
        let title = self.parse_title()?;

        let prefix_chapters = self
            .parse_affix(true)
            .with_context(|| "There was an error parsing the prefix chapters")?;
        let numbered_chapters = self
            .parse_parts()
            .with_context(|| "There was an error parsing the numbered chapters")?;
        let suffix_chapters = self
            .parse_affix(false)
            .with_context(|| "There was an error parsing the suffix chapters")?;

        Ok(Summary {
            title,
            prefix_chapters,
            numbered_chapters,
            suffix_chapters,
        })
    }

    /// Parse the affix chapters.
    fn parse_affix(&mut self, is_prefix: bool) -> Result<Vec<Chapter>> {
        let mut items = Vec::new();
        debug!(
            "Parsing {} items",
            if is_prefix { "prefix" } else { "suffix" }
        );

        loop {
            match self.next_event() {
                Some(ev @ Event::Start(Tag::List(..)))
                | Some(ev @ Event::Start(Tag::Heading(1))) => {
                    if is_prefix {
                        // we've finished prefix chapters and are at the start
                        // of the numbered section.
                        self.back(ev);
                        break;
                    } else {
                        bail!(self.parse_error("Suffix chapters cannot be followed by a list"));
                    }
                }
                Some(Event::Start(Tag::Link(_type, href, _title))) => {
                    let link = self.parse_link(href.to_string())?;
                    if !link.nested_items.is_empty() {
                        bail!(self.parse_error("Cannot have nested chapted in prefix.postfix"));
                    }
                    assert!(link.section_number == None);
                    items.push(link.chapter);
                }
                Some(_) => {}
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_parts(&mut self) -> Result<Vec<Link>> {
        let mut parts = vec![];

        // We want the section numbers to be continues through all parts.
        let mut root_number = SectionNumber::default();
        let mut root_items = 0;

        loop {
            // Buthured remnents of when their could be titles halfway throught the summary.
            // TODO: Cleanup and merge with parse_numbered
            match self.next_event() {
                Some(ev @ Event::Start(Tag::Paragraph)) => {
                    // we're starting the suffix chapters
                    self.back(ev);
                    break;
                }

                //TODO: Figure out why I need this
                Some(Event::Start(Tag::Heading(1))) => {
                    collect_events!(self.stream, end Tag::Heading(1));
                }

                Some(ev) => {
                    self.back(ev);
                }

                None => break, // EOF, bail...
            };

            // Parse the rest of the part.
            let numbered_chapters = self
                .parse_numbered(&mut root_items, &mut root_number)
                .with_context(|| "There was an error parsing the numbered chapters")?;

            parts.extend(numbered_chapters);
        }

        Ok(parts)
    }

    /// Finishes parsing a link once the `Event::Start(Tag::Link(..))` has been opened.
    fn parse_link(&mut self, href: String) -> Result<Link> {
        let href = href.replace("%20", " ");
        let link_content = collect_events!(self.stream, end Tag::Link(..));
        let name = stringify_events(link_content);

        let location = if href.is_empty() {
            None
        } else {
            Some(PathBuf::from(href))
        };

        Ok(Link {
            chapter: Chapter { name, location },
            section_number: None,
            nested_items: Vec::new(),
        })
    }

    /// Parse the numbered chapters.
    fn parse_numbered(
        &mut self,
        root_items: &mut u32,
        root_number: &mut SectionNumber,
    ) -> Result<Vec<Link>> {
        let mut items = Vec::new();

        // For the first iteration, we want to just skip any opening paragraph tags, as that just
        // marks the start of the list. But after that, another opening paragraph indicates that we
        // have started a new part or the suffix chapters.
        let mut first = true;

        loop {
            match self.next_event() {
                Some(ev @ Event::Start(Tag::Paragraph)) => {
                    if !first {
                        // we're starting the suffix chapters
                        self.back(ev);
                        break;
                    }
                }
                // The expectation is that pulldown cmark will terminate a paragraph before a new
                // heading, so we can always count on this to return without skipping headings.
                Some(ev @ Event::Start(Tag::Heading(1))) => {
                    // we're starting a new part
                    self.back(ev);
                    break;
                }
                Some(ev @ Event::Start(Tag::List(..))) => {
                    self.back(ev);
                    let mut bunch_of_items = self.parse_nested_numbered(&root_number)?;

                    // if we've resumed after something like a rule the root sections
                    // will be numbered from 1. We need to manually go back and update
                    // them
                    update_section_numbers(&mut bunch_of_items, 0, *root_items);
                    *root_items += bunch_of_items.len() as u32;
                    items.extend(bunch_of_items);
                }
                Some(Event::Start(other_tag)) => {
                    trace!("Skipping contents of {:?}", other_tag);

                    // Skip over the contents of this tag
                    while let Some(event) = self.next_event() {
                        if event == Event::End(other_tag.clone()) {
                            break;
                        }
                    }
                }

                // something else... ignore
                Some(_) => {}

                // EOF, bail...
                None => {
                    break;
                }
            }

            // From now on, we cannot accept any new paragraph opening tags.
            first = false;
        }

        Ok(items)
    }

    /// Push an event back to the tail of the stream.
    fn back(&mut self, ev: Event<'a>) {
        assert!(self.back.is_none());
        trace!("Back: {:?}", ev);
        self.back = Some(ev);
    }

    fn next_event(&mut self) -> Option<Event<'a>> {
        let next = self.back.take().or_else(|| {
            self.stream.next().map(|(ev, range)| {
                self.offset = range.start;
                ev
            })
        });

        trace!("Next event: {:?}", next);

        next
    }

    fn parse_nested_numbered(&mut self, parent: &SectionNumber) -> Result<Vec<Link>> {
        debug!("Parsing numbered chapters at level {}", parent);
        let mut items = Vec::new();

        loop {
            match self.next_event() {
                Some(Event::Start(Tag::Item)) => {
                    let item = self.parse_nested_item(parent, items.len())?;
                    items.push(item);
                }
                Some(Event::Start(Tag::List(..))) => {
                    // Skip this tag after comment bacause it is not nested.
                    if items.is_empty() {
                        continue;
                    }
                    // recurse to parse the nested list
                    let (_, last_item) = get_last_link(&mut items)?;
                    let last_item_number = last_item
                        .section_number
                        .as_ref()
                        .expect("All numbered chapters have numbers");

                    let sub_items = self.parse_nested_numbered(last_item_number)?;

                    last_item.nested_items = sub_items;
                }
                Some(Event::End(Tag::List(..))) => break,
                Some(_) => {}
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_nested_item(
        &mut self,
        parent: &SectionNumber,
        num_existing_items: usize,
    ) -> Result<Link> {
        loop {
            match self.next_event() {
                Some(Event::Start(Tag::Paragraph)) => continue,
                Some(Event::Start(Tag::Link(_type, href, _title))) => {
                    let mut link = self.parse_link(href.to_string())?;

                    let mut number = parent.clone();
                    number.0.push(num_existing_items as u32 + 1);
                    trace!(
                        "Found chapter: {} {} ({})",
                        number,
                        link.chapter.name,
                        link.chapter
                            .location
                            .as_ref()
                            .map(|p| p.to_str().unwrap_or(""))
                            .unwrap_or("[draft]")
                    );

                    link.section_number = Some(number);

                    return Ok(link);
                }
                other => {
                    warn!("Expected a start of a link, actually got {:?}", other);
                    bail!(self.parse_error(
                        "The link items for nested chapters must only contain a hyperlink"
                    ));
                }
            }
        }
    }

    fn parse_error<D: Display>(&self, msg: D) -> Error {
        let (line, col) = self.current_location();
        eyre!(
            "failed to parse SUMMARY.md line {}, column {}: {}",
            line,
            col,
            msg
        )
    }

    /// Try to parse the title line.
    fn parse_title(&mut self) -> Result<String> {
        match self.next_event() {
            Some(Event::Start(Tag::Heading(1))) => {
                debug!("Found a h1 in the SUMMARY");

                let tags = collect_events!(self.stream, end Tag::Heading(1));
                Ok(stringify_events(tags))
            }
            _ => bail!(self.parse_error("Could not parse title")),
        }
    }
}

fn update_section_numbers(sections: &mut [Link], level: usize, by: u32) {
    for link in sections {
        if let Some(ref mut number) = link.section_number {
            number.0[level] += by;
        }

        update_section_numbers(&mut link.nested_items, level, by);
    }
}

/// Gets a pointer to the last `Link` in a list of `SummaryItem`s, and its
/// index.
fn get_last_link(links: &mut [Link]) -> Result<(usize, &mut Link)> {
    links
        .iter_mut()
        .enumerate()
        //.filter_map(|(i, item)| item.maybe_link_mut().map(|l| (i, l)))
        .rev()
        .next()
        .ok_or_else(||
            eyre!("Unable to get last link because the list of SummaryItems doesn't contain any Links")
            )
}

/// Removes the styling from a list of Markdown events and returns just the
/// plain text.
fn stringify_events(events: Vec<Event<'_>>) -> String {
    events
        .into_iter()
        .filter_map(|t| match t {
            Event::Text(text) | Event::Code(text) => Some(text.into_string()),
            Event::SoftBreak => Some(String::from(" ")),
            _ => None,
        })
        .collect()
}

/// A section number like "1.2.3", basically just a newtype'd `Vec<u32>` with
/// a pretty `Display` impl.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize, Hash)]
pub struct SectionNumber(pub Vec<u32>);

impl Display for SectionNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "0")
        } else {
            for item in &self.0 {
                write!(f, "{}.", item)?;
            }
            Ok(())
        }
    }
}

impl Deref for SectionNumber {
    type Target = Vec<u32>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SectionNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<u32> for SectionNumber {
    fn from_iter<I: IntoIterator<Item = u32>>(it: I) -> Self {
        SectionNumber(it.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;

    use super::*;

    #[test]
    fn section_number_has_correct_dotted_representation() {
        let inputs = vec![
            (vec![0], "0."),
            (vec![1, 3], "1.3."),
            (vec![1, 2, 3], "1.2.3."),
        ];

        for (input, should_be) in inputs {
            let section_number = SectionNumber(input).to_string();
            assert_eq!(section_number, should_be);
        }
    }

    #[test]
    fn parse_initial_title() {
        let src = "# Summary";
        let should_be = String::from("Summary");

        let mut parser = SummaryParser::new(src);
        let got = parser.parse_title().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_title_with_styling() {
        let src = "# My **Awesome** Summary";
        let should_be = String::from("My Awesome Summary");

        let mut parser = SummaryParser::new(src);
        let got = parser.parse_title().unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn convert_markdown_events_to_a_string() {
        let src = "Hello *World*, `this` is some text [and a link](./path/to/link)";
        let should_be = "Hello World, this is some text and a link";

        let events = pulldown_cmark::Parser::new(src).collect();
        let got = stringify_events(events);

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_some_prefix_items() {
        let src = "[First](./first.md)\n[Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let should_be = vec![
            Chapter {
                name: String::from("First"),
                location: Some(PathBuf::from("./first.md")),
                ..Default::default()
            },
            Chapter {
                name: String::from("Second"),
                location: Some(PathBuf::from("./second.md")),
                ..Default::default()
            },
        ];

        let got = parser.parse_affix(true).unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn suffix_items_cannot_be_followed_by_a_list() {
        let src = "[First](./first.md)\n- [Second](./second.md)\n";
        let mut parser = SummaryParser::new(src);

        let got = parser.parse_affix(false);

        assert!(got.is_err());
    }

    #[test]
    fn parse_a_link() {
        let src = "[First](./first.md)";
        let should_be = Link {
            chapter: Chapter {
                name: String::from("First"),
                location: Some(PathBuf::from("./first.md")),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut parser = SummaryParser::new(src);
        let _ = parser.stream.next(); // Discard opening paragraph

        let href = match parser.stream.next() {
            Some((Event::Start(Tag::Link(_type, href, _title)), _range)) => href.to_string(),
            other => panic!("Unreachable, {:?}", other),
        };

        let got = parser.parse_link(href).unwrap();
        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_a_numbered_chapter() {
        let src = "- [First](./first.md)\n";
        let link = Link {
            chapter: Chapter {
                name: String::from("First"),
                location: Some(PathBuf::from("./first.md")),
                ..Default::default()
            },

            section_number: Some(SectionNumber(vec![1])),
            ..Default::default()
        };
        let should_be = vec![link];

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_eq!(got, should_be);
    }

    #[test]
    fn parse_nested_numbered_chapters() {
        let src = "- [First](./first.md)\n  - [Nested](./nested.md)\n- [Second](./second.md)";

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_yaml_snapshot!(got);
    }

    #[test]
    fn parse_numbered_chapters_separated_by_comment() {
        let src = "- [First](./first.md)\n<!-- this is a comment -->\n- [Second](./second.md)";

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_yaml_snapshot!(got);
    }

    #[test]
    fn parse_titled_parts() {
        let src = "- [First](./first.md)\n- [Second](./second.md)\n\
                   # Title 2\n- [Third](./third.md)\n\t- [Fourth](./fourth.md)";

        let mut parser = SummaryParser::new(src);
        let got = parser.parse_parts().unwrap();
        assert_yaml_snapshot!(got);
    }

    /// This test ensures the book will continue to pass because it breaks the
    /// `SUMMARY.md` up using level 2 headers ([example]).
    ///
    /// [example]: https://github.com/rust-lang/book/blob/2c942dc094f4ddcdc7aba7564f80782801197c99/second-edition/src/SUMMARY.md#basic-rust-literacy
    #[test]
    fn can_have_a_subheader_between_nested_items() {
        let src = "- [First](./first.md)\n\n## Subheading\n\n- [Second](./second.md)\n";

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_yaml_snapshot!(got);
    }

    #[test]
    fn an_empty_link_location_is_a_draft_chapter() {
        let src = "- [Empty]()\n";
        let mut parser = SummaryParser::new(src);

        let got = parser.parse_numbered(&mut 0, &mut SectionNumber::default());

        assert!(got.is_ok());
        assert_yaml_snapshot!(got.unwrap());
    }

    /// Regression test for https://github.com/rust-lang/mdBook/issues/779
    /// Ensure section numbers are correctly incremented after a horizontal separator.
    #[test]
    fn keep_numbering_after_separator() {
        let src =
            "- [First](./first.md)\n---\n- [Second](./second.md)\n---\n- [Third](./third.md)\n";

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_yaml_snapshot!(got);
    }

    /// Regression test for https://github.com/rust-lang/mdBook/issues/1218
    /// Ensure chapter names spread across multiple lines have spaces between all the words.
    #[test]
    fn add_space_for_multi_line_chapter_names() {
        let src = "- [Chapter\ntitle](./chapter.md)";

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_yaml_snapshot!(got);
    }

    #[test]
    fn allow_space_in_link_destination() {
        let src = "- [test1](./test%20link1.md)\n- [test2](<./test link2.md>)";

        let mut parser = SummaryParser::new(src);
        let got = parser
            .parse_numbered(&mut 0, &mut SectionNumber::default())
            .unwrap();

        assert_yaml_snapshot!(got);
    }
}

// Copyright 2020 Nixon Enraght Moony. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;
use std::io::prelude::*;
use std::io::Result;

macro_rules! dbg {
    () => {
        eprintln!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { $crate::dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

const DEFAULT_WIDTH: usize = 78;

fn _escape_path(word: &str) -> String {
    //PERFSMALL: Less allocs
    word.replace("$ ", "$$ ")
        .replace(" ", "$ ")
        .replace(":", "$:")
}

fn count_dollars_before_index(s: &[u8], i: usize) -> usize {
    let mut dollar_count = 0;
    let mut dollar_index = i - 1;
    while dollar_index > 0 && s[dollar_index] == b'$' {
        dollar_count += 1;
        dollar_index -= 1;
    }
    dollar_count
}

pub struct NinjaWritter<W> {
    width: usize,
    output: W,
}

impl<W> NinjaWritter<W> {
    pub fn new(writer: W) -> Self {
        Self::with_width(writer, DEFAULT_WIDTH)
    }

    pub fn with_width(writer: W, width: usize) -> Self {
        Self {
            output: writer,
            width,
        }
    }
}

impl<W: Write> NinjaWritter<W> {
    pub fn newline(&mut self) -> Result<()> {
        self.output.write_all(b"\n")
    }

    pub(crate) fn line(&mut self, text: &[u8]) -> Result<()> {
        self.line_indent(text, 0)
    }

    pub(crate) fn line_indent(&mut self, mut text: &[u8], indent: usize) -> Result<()> {
        //TODO: Don't alloc
        let mut leading_space = b"  ".repeat(indent);
        while leading_space.len() + text.len() > self.width {
            let available_space = self.width - leading_space.len() - " $".len();
            let mut space = Some(available_space);
            loop {
                space = memchr::memrchr(b' ', &text[..space.unwrap_or(text.len() - 1)]);
                dbg!(space);
                if match space {
                    None => true,
                    Some(s) => count_dollars_before_index(text, s) % 2 == 0,
                } {
                    break;
                }
            }

            if space == None {
                space = Some(available_space - 1);
                loop {
                    // Oh dear god oh god.
                    let newspace = memchr::memchr(b' ', &text[space.expect("xkcd 2200") + 1..]);
                    match newspace {
                        None => space = None,
                        Some(s) => space = Some(space.expect("xkcd 2200") + s + 1),
                    }
                    if match space {
                        None => true,
                        Some(s) => count_dollars_before_index(text, s) % 2 == 0,
                    } {
                        break;
                    }
                }
            }

            match space {
                None => break,
                Some(s) => {
                    self.output.write_all(&leading_space)?;
                    self.output.write_all(&text[..s])?;
                    self.output.write_all(b" $\n")?;
                    leading_space = b"  ".repeat(indent + 2);
                    text = &text[s + 1..];
                }
            }
        }

        self.output.write_all(&leading_space)?;
        self.output.write_all(text)?;
        self.output.write_all(b"\n")
    }

    pub fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }

    pub fn comment(&mut self, text: &str) -> io::Result<()> {
        let mut wr = textwrap::Wrapper::with_splitter(self.width - 2, textwrap::NoHyphenation);
        wr.break_words = false;

        for i in wr.wrap_iter(text) {
            self.output.write_all(b"# ")?;
            self.output.write_all(&i.as_bytes())?;
            self.output.write_all(b"\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn line_test(inp: &str, out: &str) {
        let mut x = Vec::<u8>::new();
        let mut ninja = NinjaWritter::with_width(&mut x, 8);
        ninja.line(inp.as_bytes()).unwrap();
        assert_eq!(String::from_utf8(x).unwrap(), out);
    }

    #[test]
    fn single_long_word() {
        line_test("aaaaaaaaaa", "aaaaaaaaaa\n");
    }

    #[test]
    fn few_long_words() {
        line_test("x 0123456789 y", "\
x $
    0123456789 $
    y
");
    }

    #[test]
    fn comment_wrap() {
        let mut x = Vec::<u8>::new();
        let mut ninja = NinjaWritter::with_width(&mut x, 8);
        ninja.comment("Hello /usr/local/build-tools/bin").unwrap();
        assert_eq!(
            String::from_utf8(x).unwrap(),
            "\
# Hello
# /usr/local/build-tools/bin
"
        );
    }

    #[test]
    fn short_words_indented() {
        line_test(
            "line_one to tree",
            "\
line_one $
    to $
    tree
",
        );
    }

    #[test]
    fn short_words_indented2() {
        line_test(
            "lineone to tree",
            "\
lineone $
    to $
    tree
",
        );
    }

    #[test]
    fn escaped_spaces() {
        line_test("x aaaaa$ aaaaa y", "\
x $
    aaaaa$ aaaaa $
    y
")
    }

    #[test]
    fn fit_many_words() {
        let mut x = Vec::<u8>::new();
        let mut ninja = NinjaWritter::with_width(&mut x, 78);
        ninja.line_indent(b"command = cd ../../chrome; python ../tools/grit/grit/format/repack.py ../out/Debug/obj/chrome/chrome_dll.gen/repack/theme_resources_large.pak ../out/Debug/gen/chrome/theme_resources_large.pak", 1).unwrap();
        assert_eq!(
            String::from_utf8(x).unwrap(),
            "  \
command = cd ../../chrome; python ../tools/grit/grit/format/repack.py $
      ../out/Debug/obj/chrome/chrome_dll.gen/repack/theme_resources_large.pak $
      ../out/Debug/gen/chrome/theme_resources_large.pak
"
        )
    }
}

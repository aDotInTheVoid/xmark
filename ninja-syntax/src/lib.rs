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

//TODO: Less of a hack
const SPACES: &[u8] = &[b' '; 1024];
const DEFAULT_WIDTH: usize = 78;

fn escape_path(word: &str) -> String {
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
    writer: W,
}

impl<W> NinjaWritter<W> {
    pub fn new(writer: W) -> Self {
        Self::with_width(writer, DEFAULT_WIDTH)
    }

    pub fn with_width(writer: W, width: usize) -> Self {
        Self { writer, width }
    }
}

impl<W: Write> NinjaWritter<W> {
    pub fn newline(&mut self) -> Result<()> {
        self.writer.write_all(b"\n")
    }

    fn line(&mut self, text: &str) -> Result<()> {
        self.line_indent(text, 0)
    }

    fn line_indent(&mut self, mut text: &str, indent: usize) -> Result<()> {
        let mut leading_space = &SPACES[..indent];

        while leading_space.len() + text.len() > self.width {
            // The text is too wide; wrap if possible.

            // Find the rightmost space that would obey our width constraint and
            // that's not an escaped space.
            let available_space = self.width - leading_space.len() - b" $".len();
            let mut space = Some(available_space);
            loop {
                //TODO: Non ascii
                space = text[..space.unwrap_or(text.len())].rfind(' ');
                if space
                    .map(|x| count_dollars_before_index(text.as_bytes(), x) % 2 == 0)
                    .unwrap_or(false)
                {
                    break;
                }
            }

            if space.is_none() {
                space = Some(available_space - 1);
                loop {
                    space = text[..space.expect("Unreachable. xkcd.com/2200/") + 1].find(' ');
                    if space
                        .map(|x| count_dollars_before_index(text.as_bytes(), x) % 2 == 0)
                        .unwrap_or(false)
                    {
                        break;
                    }
                }
            }
            
            // Give up
            if space.is_none() {break;}
            match space {
                None => break,
                Some(space) => {
                    self.writer.write_all(leading_space)?;
                    self.writer.write_all(text[..space].as_bytes())?;
                    self.writer.write_all(b" $\n")?;
                    text = &text[space+1..];
                    leading_space = &SPACES[..indent*2];
                }
            }
        }

        self.writer.write_all(leading_space)?;
        self.writer.write_all(text.as_bytes())?;
        self.writer.write_all(b"\n")?;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::{Result, Write};

#[derive(Clone, Copy)]
pub enum IndentConfig {
    Tab,
    Space(usize),
}

pub struct IndentedWriter<T> {
    out: T,
    indentation: Vec<u8>,
    config: IndentConfig,
    at_begining_of_line: bool,
}

impl<T> IndentedWriter<T> {
    pub fn new(out: T, config: IndentConfig) -> Self {
        Self {
            out,
            indentation: Vec::new(),
            config,
            at_begining_of_line: true,
        }
    }

    pub fn indent(&mut self) {
        match self.config {
            IndentConfig::Tab => {
                self.indentation.push(b'\t');
            }
            IndentConfig::Space(n) => {
                self.indentation.resize(self.indentation.len() + n, b' ');
            }
        }
    }

    pub fn unindent(&mut self) {
        match self.config {
            IndentConfig::Tab => {
                self.indentation.pop();
            }
            IndentConfig::Space(n) => {
                self.indentation
                    .truncate(self.indentation.len().saturating_sub(n));
            }
        }
    }
}

impl<T: Write> Write for IndentedWriter<T> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize> {
        let mut bytes_written = 0;

        while !buf.is_empty() {
            let (before_newline, has_newline, after_newline) =
                if let Some(idx) = buf.iter().position(|&b| b == b'\n') {
                    (&buf[..idx], true, &buf[idx + 1..])
                } else {
                    (buf, false, &buf[buf.len()..])
                };

            if self.at_begining_of_line && !before_newline.is_empty() {
                self.out.write_all(&self.indentation)?;
                self.at_begining_of_line = false;
            }

            self.out.write_all(before_newline)?;
            bytes_written += before_newline.len();

            if has_newline {
                self.out.write_all(b"\n")?;
                bytes_written += 1;
                self.at_begining_of_line = true;
            }

            buf = after_newline;
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> Result<()> {
        self.out.flush()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut out = IndentedWriter::new(&mut buffer, IndentConfig::Space(2));

        writeln!(out, "foo")?;
        out.indent();
        writeln!(out, "bar")?;
        writeln!(out)?;
        writeln!(out, "bar")?;
        out.indent();
        writeln!(out, "foobar")?;
        writeln!(out)?;
        writeln!(out, "foobar")?;
        out.unindent();
        out.unindent();
        writeln!(out, "foo")?;

        let expect: &[u8] = b"\
foo
  bar

  bar
    foobar

    foobar
foo
";
        assert_eq!(buffer, expect);

        Ok(())
    }
}

use std::io::{Result, Write};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum IndentConfig {
    Tab,
    Space(usize),
}

pub struct IndentedWriter<T> {
    w: T,
    indent_level: usize,
    indent: IndentConfig,
    at_begining_of_line: bool,
}

impl<T> IndentedWriter<T> {
    pub fn new(w: T, indent: IndentConfig) -> Self {
        Self {
            w,
            indent_level: 0,
            indent,
            at_begining_of_line: true,
        }
    }

    pub fn inc_level(&mut self) {
        self.inc_level_by(1);
    }

    pub fn dec_level(&mut self) {
        self.dec_level_by(1);
    }

    pub fn inc_level_by(&mut self, n: usize) {
        self.indent_level = self.indent_level.saturating_add(n);
    }

    pub fn dec_level_by(&mut self, n: usize) {
        self.indent_level = self.indent_level.saturating_sub(n);
    }
}

impl<T> AsRef<T> for IndentedWriter<T> {
    fn as_ref(&self) -> &T {
        &self.w
    }
}

impl<T> AsMut<T> for IndentedWriter<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.w
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
                for _ in 0..self.indent_level {
                    match self.indent {
                        IndentConfig::Tab => self.w.write_all(b"\t")?,
                        IndentConfig::Space(n) => {
                            for _ in 0..n {
                                self.w.write_all(b" ")?;
                            }
                        }
                    }
                }
                self.at_begining_of_line = false;
            }

            self.w.write_all(before_newline)?;
            bytes_written += before_newline.len();

            if has_newline {
                self.w.write_all(b"\n")?;
                bytes_written += 1;
                self.at_begining_of_line = true;
            }

            buf = after_newline;
        }

        Ok(bytes_written)
    }

    fn flush(&mut self) -> Result<()> {
        self.w.flush()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let mut out: Vec<u8> = Vec::new();

        let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(2));

        writeln!(w, "foo")?;
        w.inc_level();
        writeln!(w, "bar")?;
        writeln!(w)?;
        writeln!(w, "bar")?;
        w.inc_level();
        writeln!(w, "foobar")?;
        writeln!(w)?;
        writeln!(w, "foobar")?;
        w.dec_level_by(2);
        writeln!(w, "foo")?;

        let expect: &[u8] = b"\
foo
  bar

  bar
    foobar

    foobar
foo
";
        assert_eq!(out, expect);

        Ok(())
    }
}

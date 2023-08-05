use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct TokenStream<'a> {
  source: &'a str,
  line: u16,
  col: u16,
}

fn indentation(s: &str) -> usize {
  if let Some((i, _)) = s.char_indices().find(|(_, c)| *c != ' ') {
    i
  } else {
    0
  }
}

impl<'a> TokenStream<'a> {
  pub fn new(s: &'a str) -> TokenStream<'a> {
    TokenStream {
      source: s,
      line: 0,
      col: 0,
    }
  }
  /// Precondition: other is a suffix of self.
  fn endat(&mut self, suffix: &Self) {
    let offset = self.source.len() - suffix.source.len();
    self.source = &self.source[0..offset];
  }
  pub fn tail(&mut self) -> (&str, Range) {
    let line1 = self.line + self.source.chars().filter(|c| *c == '\n').count() as u16;
    let col1 = if let Some(l) = self.source.lines().last() {
      l.len() as u16
    } else {
      0
    };
    (
      self.source,
      Range {
        line0: self.line,
        col0: self.col,
        line1,
        col1,
      },
    )
  }
  /// Consume and return a token stream over the next block defined by the offsides rule. A block
  /// defined by the offsides rule ends before the first line after the first line whose indentation
  /// is _equal_ to that of the first line; therefore, the offsides block of a nonempty stream must
  /// have at least one line.
  ///
  /// Precondition: There are no newlines or empty lines before the start of the block.
  pub fn block(&mut self) -> Result<Self, (String, Range)> {
    let mut og = *self;
    let thresh = indentation(self.source);
    self.line()?;
    let mut backup = *self;
    while let Ok(line) = self.line() {
      if indentation(line.source) <= thresh {
        break;
      }
      backup = *self;
    }
    *self = backup;
    og.endat(self);
    Ok(og)
  }

  pub fn section(&mut self) -> Self {
    let mut og = *self;
    let mut backup = *self;
    while let Ok(mut block) = self.block() {
      if let Ok(t) = block.token() {
        if t.s == "---" {
          break;
        }
      }
      backup = *self;
    }
    og.endat(&backup);
    og
  }
  /// Consume and return the token stream consisting only of the first unconsumed line of self.
  pub fn line(&mut self) -> Result<Self, (String, Range)> {
    let split_at = self
      .source
      .char_indices()
      .find_map(|(i, c)| if c == '\n' { Some(i) } else { None });
    let mut ret = *self;
    match (split_at, self.source.len()) {
      (None, 0) => Err((
        "token stream starts with a newline".to_string(),
        self.tail().1,
      )),
      (Some(0), _) => Err(("token stream is empty".to_string(), self.tail().1)),
      (None, _) => {
        self.source = &self.source[self.source.len()..];
        self.line += 1;
        Ok(ret)
      }
      (Some(split_at), _) => {
        self.source = &self.source[split_at + 1..];
        self.line += 1;
        self.col = 0;
        ret.endat(self);
        Ok(ret)
      }
    }
  }

  /// Skip Unix-style lines that are entirely empty (i.e., without evenn whitespace).
  #[cfg(test)]
  pub fn skip_blank_lines(&mut self) {
    let start = *self.source.find(|it| it != '\n').get_or_insert(0);
    self.source = &self.source[start..];
    self.line += start as u16;
  }
  /// Skip all whitespace.
  pub fn skip_whitespace(&mut self) {
    while let Some(c) = self.source.chars().next() {
      match c {
        '\n' => {
          self.line += 1;
          self.col = 0;
        }
        ' ' => {
          self.col += 1;
        }
        _ => {
          break;
        }
      }
      self.source = &self.source[1..];
    }
  }
  /// Consume a token and all whitespace before it, and produce the token.
  pub fn token(&mut self) -> Result<Token<'a>, (String, Range)> {
    self.skip_whitespace();
    let length = if self.source.starts_with('.') {
      1
    } else {
      *self
        .source
        .find(|c: char| c.is_whitespace() || c == '.')
        .get_or_insert(self.source.len())
    };
    if length == 0 {
      return Err(("expected token, not nothing".to_string(), self.tail().1));
    }
    let ret = Token {
      s: &self.source[..length],
      r: Range {
        line0: self.line,
        col0: self.col,
        line1: self.line,
        col1: self.col + length as u16,
      },
    };
    self.col += length as u16;
    self.source = &self.source[length..];
    Ok(ret)
  }
  pub fn is_empty(&self) -> bool {
    self.source.is_empty()
  }
}

macro_rules! makeIterator {
  ($structname:ident, $name: ident, $iterator_getter: ident, $result: ident) => {
    impl<'a> TokenStream<'a> {
      pub fn $iterator_getter<'b>(&'b mut self) -> $structname<'b, 'a> {
        $structname { ts: self }
      }
    }

    pub struct $structname<'b, 'a> {
      ts: &'b mut TokenStream<'a>,
    }

    impl<'b, 'a> Iterator for $structname<'b, 'a> {
      type Item = $result<'a>;

      fn next(&mut self) -> Option<Self::Item> {
        if let Ok(line) = self.ts.$name() {
          Some(line)
        } else {
          None
        }
      }
    }
  };
}

makeIterator!(LineIterator, line, lines, TokenStream);
makeIterator!(BlockIterator, block, blocks, TokenStream);

/// An opaque but serializable and deserializable Range object.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
  line0: u16,
  col0: u16,
  line1: u16,
  col1: u16,
}
/// A token and the range from which it came.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
  pub r: Range,
  pub s: &'a str,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_token() {
    let mut ts = TokenStream::new("this.is");
    assert_eq!(
      ts.token(),
      Ok(Token {
        s: "this",
        r: Range {
          line0: 0,
          col0: 0,
          line1: 0,
          col1: 4,
        },
      }),
    );
    assert_eq!(
      ts.token(),
      Ok(Token {
        s: ".",
        r: Range {
          line0: 0,
          col0: 4,
          line1: 0,
          col1: 5
        }
      })
    );
    assert_eq!(
      ts.token(),
      Ok(Token {
        s: "is",
        r: Range {
          line0: 0,
          col0: 5,
          line1: 0,
          col1: 7
        }
      })
    );
    assert!(ts.token().is_err());
  }
  #[test]
  fn test_offsides() {
    let mut ts = TokenStream::new(
      "
    the most
      exciting test
    imaginable",
    );
    ts.skip_blank_lines();
    let mut block = ts.block().unwrap();
    let mut line = block.line().unwrap();
    assert_eq!(
      line.token(),
      Ok(Token {
        s: "the",
        r: Range {
          line0: 1,
          col0: 4,
          line1: 1,
          col1: 7
        }
      })
    );
    line.token().unwrap();
    assert!(line.token().is_err());
    line = block.line().unwrap();
    assert_eq!(
      line.token(),
      Ok(Token {
        s: "exciting",
        r: Range {
          line0: 2,
          col0: 6,
          line1: 2,
          col1: 14
        }
      })
    );
    line.token().unwrap();
    assert!(line.token().is_err());
    let next_line_in_block = block.line();
    assert!(
      next_line_in_block.is_err(),
      "{}",
      next_line_in_block.unwrap().source
    );
    line = ts.line().unwrap();
    assert_eq!(
      line.token(),
      Ok(Token {
        s: "imaginable",
        r: Range {
          line0: 3,
          col0: 4,
          line1: 3,
          col1: 14
        }
      })
    );
  }
  #[test]
  fn test_section() {
    let mut ts = TokenStream::new(
      "section0
---
---
section2
section2'
---",
    );
    let mut section0 = ts.section();
    assert_eq!(section0.token().unwrap().s, "section0");
    assert!(ts.section().token().is_err());
    let mut section2 = ts.section();
    assert_eq!(section2.token().unwrap().s, "section2");
    assert_eq!(section2.token().unwrap().s, "section2'");
    assert!(section2.token().is_err());
    assert!(ts.section().token().is_err());
  }
}

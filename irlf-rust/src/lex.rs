use serde::{Deserialize, Serialize};

pub struct TokenStream<'a> {
  source: &'a str,
  line: u16,
  col: u16,
}

impl<'a> Clone for TokenStream<'a> {
  fn clone(&self) -> Self {
    Self {
      source: self.source,
      line: self.line,
      col: self.col,
    }
  }
}

impl<'a> Copy for TokenStream<'a> {}

fn indentation(s: &str) -> usize {
  if let Some((i, _)) = s.char_indices().find(|(i, c)| *c != ' ') {
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
  /// Consume and return a token stream over the next block defined by the offsides rule. A block
  /// defined by the offsides rule ends before the first line after the first line whose indentation
  /// is _equal_ to that of the first line; therefore, the offsides block of a nonempty stream must
  /// have at least one line.
  ///
  /// Precondition: There are no newlines or empty lines before the start of the block.
  pub fn offsides(&mut self) -> Option<Self> {
    let mut og = *self;
    let thresh = indentation(self.source);
    self.line()?;
    let mut backup = *self;
    while let Some(line) = self.line() {
      if indentation(line.source) <= thresh {
        break;
      }
      backup = *self;
    }
    *self = backup;
    og.endat(self);
    Some(og)
  }
  /// Consume and return the token stream consisting only of the first unconsumed line of self.
  pub fn line(&mut self) -> Option<Self> {
    let split_at = self
      .source
      .char_indices()
      .find_map(|(i, c)| if c == '\n' { Some(i) } else { None });
    let mut ret = *self;
    match (split_at, self.source.len()) {
      (None, 0) | (Some(0), _) => None,
      (None, _) => {
        self.source = &self.source[self.source.len()..];
        self.line += 1;
        Some(ret)
      }
      (Some(split_at), _) => {
        println!("{split_at}");
        self.source = &self.source[split_at + 1..];
        self.line += 1;
        self.col = 0;
        ret.endat(self);
        Some(ret)
      }
    }
  }
  pub fn skip_blank_lines(&mut self) {
    let start = *self.source.find(|it| it != '\n').get_or_insert(0);
    self.source = &self.source[start..];
    self.line += start as u16;
  }
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
  pub fn token(&mut self) -> Option<Token<'a>> {
    self.skip_whitespace();
    let mut length = self.source.find(char::is_whitespace);
    let length = *length.get_or_insert(self.source.len());
    if length == 0 {
      return None;
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
    Some(ret)
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
  line0: u16,
  col0: u16,
  line1: u16,
  col1: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
  pub r: Range,
  pub s: &'a str,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::lex::TokenStream;

  #[test]
  fn test_token() {
    let mut ts = TokenStream::new("this is");
    assert_eq!(
      ts.token(),
      Some(Token {
        s: "this",
        r: Range {
          line0: 0,
          col0: 0,
          line1: 0,
          col1: 4
        }
      })
    );
    assert_eq!(
      ts.token(),
      Some(Token {
        s: "is",
        r: Range {
          line0: 0,
          col0: 5,
          line1: 0,
          col1: 7
        }
      })
    );
    assert_eq!(ts.token(), None);
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
    let mut block = ts.offsides().unwrap();
    println!("\"{}\"", block.source);
    println!("\"{}\"", ts.source);
    let mut line = block.line().unwrap();
    assert_eq!(
      line.token(),
      Some(Token {
        s: "the",
        r: Range {
          line0: 1,
          col0: 4,
          line1: 1,
          col1: 7
        }
      })
    );
    line.token();
    assert_eq!(line.token(), None);
    line = block.line().unwrap();
    assert_eq!(
      line.token(),
      Some(Token {
        s: "exciting",
        r: Range {
          line0: 2,
          col0: 6,
          line1: 2,
          col1: 14
        }
      })
    );
    line.token();
    assert_eq!(line.token(), None);
    println!("hm?");
    let next_line_in_block = block.line();
    assert!(
      next_line_in_block.is_none(),
      "{}",
      next_line_in_block.unwrap().source
    );
    line = ts.line().unwrap();
    assert_eq!(
      line.token(),
      Some(Token {
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
}

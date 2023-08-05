use std::collections::HashMap;
use std::path::PathBuf;

use crate::ir::{
  BinaryCtor, Connection, Ctor, CtorCall, IfaceElt, InstRef, LibCtor, Program, StructlikeCtor, Sym,
};
use crate::lex::{Range, Token, TokenStream};
use lf_types::{CtorId, DebugOnlyId, IfaceNode, InstId, Side};

/// Extracts a program from its pretty-printed format.
///
/// # Errors
/// Returns an error result if the program does not parse.
pub fn unpretty(s: &str) -> Result<Program, (String, Range)> {
  let mut toks = TokenStream::new(s);
  Program::unpretty(&mut toks)
}

trait Unpretty<'a>: Sized {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)>;
}

fn parse_id<Id, F: Fn(u64) -> Id>(toks: &mut TokenStream, ctor: F) -> Result<Id, (String, Range)> {
  let tok = toks.token()?;
  let parsed = if tok.s.starts_with("0x") {
    u64::from_str_radix(&tok.s[2..], 16)
  } else {
    tok.s.parse::<u64>()
  };
  if let Ok(id) = parsed {
    Ok(ctor(id))
  } else {
    Err(("expected numeric id".to_string(), tok.r))
  }
}

impl<'a> Unpretty<'a> for CtorId {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    parse_id(toks, CtorId)
  }
}

impl<'a> Unpretty<'a> for InstId {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    parse_id(toks, InstId)
  }
}

impl<'a> Unpretty<'a> for Side {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    let tok = toks.token()?;
    match tok.s {
      "L" => Ok(Side::Left),
      "R" => Ok(Side::Right),
      _ => Err(("expected L or R".to_string(), tok.r)),
    }
  }
}

impl<'a> Unpretty<'a> for IfaceNode<IfaceElt> {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    Ok(IfaceNode(Side::unpretty(toks)?, InstId::unpretty(toks)?))
  }
}

impl<'a> Unpretty<'a> for CtorCall {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    Ok(CtorCall {
      ctor: CtorId::unpretty(toks)?,
    })
  }
}

impl<'a> Unpretty<'a> for InstRef {
  fn unpretty(toks: &mut TokenStream) -> Result<InstRef, (String, Range)> {
    let mut insts = Vec::new();
    insts.push(InstId::unpretty(toks)?);
    let mut backup = *toks;
    while let Ok(Token { s: ".", .. }) = toks.token() {
      insts.push(InstId::unpretty(toks)?);
      backup = *toks;
    }
    *toks = backup;
    Ok(InstRef(insts))
  }
}

impl<'a> Unpretty<'a> for Connection {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    let id = parse_id(toks, DebugOnlyId)?;
    let left = InstRef::unpretty(toks)?;
    let right = InstRef::unpretty(toks)?;
    Ok(Connection { id, left, right })
  }
}

impl<'a> Unpretty<'a> for StructlikeCtor {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    let mut inst2sym = HashMap::new();
    let mut insts = HashMap::new();
    let mut connections = Vec::new();
    let mut iface = Vec::new();
    let mut instantiations_section = toks.section();
    let mut iface_section = toks.section();
    let mut connections_section = toks.section();
    for mut line in instantiations_section.lines() {
      let sym = line.token()?.s.to_string();
      let id = InstId::unpretty(&mut line)?;
      inst2sym.insert(id, sym);
      let Token { r, s: equals } = &line.token()?;
      match *equals {
        "=" => {
          insts.insert(id, CtorCall::unpretty(&mut line)?);
        }
        _ => {
          return Err(("expected =".to_string(), *r));
        }
      }
    }
    while !{
      iface_section.skip_whitespace();
      iface_section.is_empty()
    } {
      iface.push(IfaceNode::unpretty(&mut iface_section)?);
    }
    for mut line in connections_section.lines() {
      connections.push(Connection::unpretty(&mut line)?);
    }
    Ok(StructlikeCtor {
      inst2sym,
      insts,
      iface,
      connections,
    })
  }
}

impl<'a> Unpretty<'a> for BinaryCtor {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    let path = PathBuf::from(toks.line()?.tail().0.trim());
    Ok(BinaryCtor { path })
  }
}

impl<'a> Unpretty<'a> for LibCtor {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    let name = toks.token()?;
    Ok(LibCtor {
      name: name.s.to_string(),
    })
  }
}

fn unpretty_ctortyp<'a, CtorTyp: Unpretty<'a>, F>(
  ctor_ctor: F,
  mut section: TokenStream<'a>,
  ctor2sym: &mut HashMap<CtorId, Sym>,
  ctors: &mut HashMap<CtorId, Ctor>,
  big: bool,
) -> Result<(), (String, Range)>
where
  F: Fn(CtorTyp) -> crate::ir::Ctor,
{
  for mut block in section.blocks() {
    let mut header = block.line()?;
    let sym = header.token()?.s;
    let cid = CtorId::unpretty(&mut header)?;
    let ctor = CtorTyp::unpretty(if big { &mut block } else { &mut header })?;
    ctor2sym.insert(cid, sym.to_string());
    ctors.insert(cid, ctor_ctor(ctor));
  }
  Ok(())
}

impl<'a> Unpretty<'a> for Program {
  fn unpretty(toks: &mut TokenStream<'a>) -> Result<Self, (String, Range)> {
    let mut ctor2sym = HashMap::new();
    let mut ctors = HashMap::new();
    let lib_ctors = toks.section();
    let binary_ctors = toks.section();
    let structlike_ctors = toks.section();
    unpretty_ctortyp(Ctor::LibCtor, lib_ctors, &mut ctor2sym, &mut ctors, false)?;
    unpretty_ctortyp(
      Ctor::BinaryCtor,
      binary_ctors,
      &mut ctor2sym,
      &mut ctors,
      false,
    )?;
    unpretty_ctortyp(
      Ctor::StructlikeCtor,
      structlike_ctors,
      &mut ctor2sym,
      &mut ctors,
      true,
    )?;
    let main = CtorId::unpretty(toks)?;
    Ok(Program {
      ctorid2sym: ctor2sym,
      ctors,
      main,
    })
  }
}

#[cfg(test)]
mod tests {
  use std::fmt::Display;

  use super::*;

  fn round_trip<'a, T: Unpretty<'a> + Display>(s: &'a str) {
    let mut toks = TokenStream::new(s);
    let tripped = T::unpretty(&mut toks).unwrap().to_string();
    pretty_assertions::assert_eq!(s, tripped);
    assert!(toks.token().is_err());
  }

  #[test]
  fn test_ctorid() {
    let mut toks = TokenStream::new("120  0x999ab");
    assert_eq!(CtorId::unpretty(&mut toks), Ok(CtorId(120)));
    assert_eq!(CtorId::unpretty(&mut toks), Ok(CtorId(0x999ab)));
  }

  #[test]
  fn test_ctorcall() {
    let mut toks = TokenStream::new("0x999ab");
    assert_eq!(
      CtorCall::unpretty(&mut toks),
      Ok(CtorCall {
        ctor: CtorId(0x999ab)
      })
    );
  }

  #[test]
  fn test_instref() {
    let mut toks = TokenStream::new("1.2.3.2 3.9.9.4");
    assert_eq!(
      InstRef::unpretty(&mut toks),
      Ok(InstRef(vec![InstId(1), InstId(2), InstId(3), InstId(2)]))
    );
  }

  #[test]
  fn test_connection() {
    round_trip::<Connection>("99 1.2.3.2 3");
  }

  #[test]
  fn test_reactorctor() {
    round_trip::<StructlikeCtor>(
      "  foo 1 = 0x99
  bar 6 = 0x2a
  ---
  R 1 R 6
  ---
  10 1 6
  11 6 1
",
    );
  }

  #[test]
  fn test_program() {
    round_trip::<Program>(
      "c 0x7 add1
---
a 0x1 /this/is/a/path
b 0x2 /this/is/another/path
---
rtor0 0x3
  foo 89 = 0x4
  ---
  L 89
  ---
  90 89 89
rtor1 0x4
  baz 87 = 0x3
  bar 88 = 0x4
  ---
  L 87 R 88
  ---
  91 88 87
  92 87 87
---
0x3
",
    );
  }
}

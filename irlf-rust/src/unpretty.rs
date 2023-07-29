use crate::lex::TokenStream;

trait Unpretty<'a> {
  fn unpretty(toks: TokenStream<'a>) -> Self;
}

// impl Unpretty for  {

// }

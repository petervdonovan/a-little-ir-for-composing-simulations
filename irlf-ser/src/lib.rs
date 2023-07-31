#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
pub mod ir;
mod lex;
pub mod pretty;
pub mod unpretty;
pub mod visitor;

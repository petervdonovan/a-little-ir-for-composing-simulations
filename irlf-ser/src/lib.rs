#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
pub mod ir_serializable;
mod lex;
pub mod pretty;
pub mod unpretty;
pub mod visitor;

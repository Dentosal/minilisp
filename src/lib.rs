//! MiniLisp interpreter

// Lints
#![deny(missing_docs)]
#![deny(unused_must_use)]
// Nightly features
#![feature(bind_by_move_pattern_guards)]
#![feature(box_syntax)]
#![feature(box_patterns)]

mod corelib;
mod interpreter;
pub mod parser;
mod value;

pub use self::interpreter::Interpreter;
pub use self::value::Value;

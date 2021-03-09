//! Errors from this crate
//!

use std::error::Error;
use std::fmt::Formatter;

pub type MyLibResult<T> = Result<T, MyError>;

#[derive(Debug)]
pub enum MyError {
    FailedToRegister { dest: String },
}

impl Error for MyError {}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::FailedToRegister { dest } => {
                write!(f, "Failed to register destination '{}'", dest)
            }
        }
    }
}

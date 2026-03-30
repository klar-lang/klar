mod types;
mod checker;
mod env;

pub use types::Type;
pub use checker::{TypeChecker, TypeError};

#[cfg(test)]
mod tests;

use std::fmt::Display;

use colored::Colorize;

pub trait PrettyError {
    type Item;
    fn pretty_unwrap(self) -> Self::Item;
}

impl<T, E> PrettyError for Result<T, E>
where
    E: Display,
{
    type Item = T;
    fn pretty_unwrap(self) -> Self::Item {
        match self {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{} : {err}", "error".red().bold());
                todo!();
            }
        }
    }
}

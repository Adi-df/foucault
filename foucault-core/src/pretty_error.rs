use std::{fmt::Display, panic};

use colored::Colorize;

#[macro_export]
macro_rules! pretty_error {
    () => {
        $crate::pretty_error::pretty_error("An error occured");
    };
    ($($arg:tt)*) => {{
        $crate::pretty_error::pretty_error(&format!($($arg)*));
    }};
}

#[doc(hidden)]
pub fn pretty_error(err: &str) {
    eprintln!("{} : {err}", "error".red().bold());
}

pub trait PrettyError {
    type Item;
    fn pretty_unwrap(self) -> Self::Item;
}

impl<T, E> PrettyError for Result<T, E>
where
    E: Display + Send + 'static,
{
    type Item = T;
    fn pretty_unwrap(self) -> Self::Item {
        match self {
            Ok(val) => val,
            Err(err) => {
                pretty_error(&format!("{err}"));
                panic::resume_unwind(Box::new(err));
            }
        }
    }
}

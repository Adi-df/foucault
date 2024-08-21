use std::{fmt::Display, sync::LazyLock};

use colored::{ColoredString, Colorize};

static ERROR_PREFIX: LazyLock<ColoredString> = LazyLock::new(|| "error".red().bold());

macro_rules! pretty_error {
    () => {
        eprintln!("{} : An error occured", *ERROR_PREFIX);
    };
    ($($arg:tt)*) => {{
        eprintln!("{} : {}", *ERROR_PREFIX, format!($($arg)*));
    }};
}

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
                pretty_error!("{err}");
                todo!();
            }
        }
    }
}
